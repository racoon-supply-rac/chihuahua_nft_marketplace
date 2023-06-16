use cosmwasm_std::{
    ensure, to_binary, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult, Timestamp,
    Uint128, WasmQuery,
};

use general_utils::error::ContractError;
use general_utils::error::GenericError::DivisionError;
use general_utils::error::NftMarketplaceError::{
    AdditionalInfoNeedsToBeFilled, CantUseAdditionalInfoIfNotContract,
};
use nft_marketplace_utils::inputs::Buyer;
use nft_marketplace_utils::marketplace_statistics::CollectionVolume;
use nft_marketplace_utils::nft_collection::{
    define_unique_collection_by_denom_id, nft_collection_denoms, NftCollectionAddress, TokenId,
};
use nft_marketplace_utils::nft_sale::{
    compute_floor_collection_and_denom, define_unique_collection_nft_id, nfts_for_sale,
    TokenSaleHistory,
};
use nft_marketplace_utils::profile::{Profile, TradeInfo};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::constants::{MARKETPLACE_USDC_INDICATOR, MAX_PRICE};
use crate::helpers::royalties::compute_royalty;
use crate::state::{
    CONFIG, GENERAL_STATS, LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL, MARKETPLACE_STATS_BY_DENOM,
    PROFILES, REWARD_SYSTEM, TOKEN_SALE_HISTORY,
};

pub fn execute_buy_nft(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    additional_info: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Validate the given NFT address
    nft_collection_address = deps.api.addr_validate(&nft_collection_address)?.to_string();
    let collection_token_id_unique: String =
        define_unique_collection_nft_id(&nft_collection_address, &token_id);
    let nft_for_sale_info = nfts_for_sale().load(deps.storage, collection_token_id_unique)?;

    // Additional info can only be used internally by the contract
    if let Some(additional_info) = additional_info {
        ensure!(
            info.sender == env.contract.address,
            ContractError::NftMarketplaceError(CantUseAdditionalInfoIfNotContract {})
        );
        info = MessageInfo {
            sender: deps.api.addr_validate(&additional_info)?,
            funds: info.funds.clone(),
        };
    } else {
        ensure!(
            info.sender != env.contract.address,
            ContractError::NftMarketplaceError(AdditionalInfoNeedsToBeFilled {})
        );
    }

    // Create the buyer info & check if it matches the requirements from the sale
    let buyer = Buyer::new_checked(deps.api.addr_validate(info.sender.as_ref())?, info.funds)?;
    let collection_token_id_unique =
        define_unique_collection_nft_id(&nft_collection_address, &token_id);
    nft_for_sale_info
        .clone()
        .validate_buying_information(&buyer)?;

    // If buyer is valid, can remove the sale
    nfts_for_sale().remove(deps.storage, collection_token_id_unique.clone())?;

    // Update the states and if it was expired -> refund the buyer and cancel the sale
    let new_floor = compute_floor_collection_and_denom(
        deps.storage,
        nft_for_sale_info.clone().sale_price_denom,
        nft_for_sale_info.clone().nft_collection_address,
        MAX_PRICE,
    )?;
    let collection_denom_unique: String = define_unique_collection_by_denom_id(
        &nft_for_sale_info.nft_collection_address,
        &nft_for_sale_info.sale_price_denom,
    );
    let is_expired = nft_for_sale_info.sale_expiration.seconds() <= env.block.time.seconds();
    MARKETPLACE_STATS_BY_DENOM.update(
        deps.storage,
        &nft_for_sale_info.clone().sale_price_denom,
        |mp_info| -> Result<_, ContractError> {
            let mut mp_info = mp_info.unwrap();
            if is_expired {
                mp_info.remove_nft_for_sale();
            } else {
                mp_info.execute_sale(
                    nft_for_sale_info.clone().sale_price_value,
                    config.marketplace_pct_fees,
                );
            }
            Ok(mp_info)
        },
    )?;
    nft_collection_denoms().update(
        deps.storage,
        collection_denom_unique,
        |nft_coll_denom| -> Result<_, ContractError> {
            let mut nft_coll_denom = nft_coll_denom.unwrap();
            if is_expired {
                nft_coll_denom = nft_coll_denom.expired_sale(new_floor);
            } else {
                nft_coll_denom = nft_coll_denom
                    .execute_sale(nft_for_sale_info.clone().sale_price_value, new_floor);
            }
            Ok(nft_coll_denom)
        },
    )?;
    if is_expired {
        return Ok(ResponseHandler::expired_nft_sale_response(buyer).response);
    }

    // Update the token's sale history
    let maybe_history: Option<Vec<TokenSaleHistory>> =
        TOKEN_SALE_HISTORY.may_load(deps.storage, &collection_token_id_unique)?;
    let transaction_info = TokenSaleHistory {
        seller: nft_for_sale_info.seller.clone(),
        buyer: buyer.sender.clone(),
        nft_collection_address: nft_for_sale_info.nft_collection_address.clone(),
        token_id: nft_for_sale_info.token_id.clone(),
        sale_price_value: nft_for_sale_info.sale_price_value,
        sale_price_denom: nft_for_sale_info.sale_price_denom.clone(),
        sale_time: Timestamp::from_seconds(env.block.time.seconds()),
    };
    let mut new_vector_of_transactions = maybe_history.unwrap_or_default();
    new_vector_of_transactions.push(transaction_info);
    TOKEN_SALE_HISTORY.save(
        deps.storage,
        &collection_token_id_unique,
        &new_vector_of_transactions,
    )?;

    // Calculation of royalties and marketplace revenues
    let nft_collection_info =
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.load(deps.storage, &nft_collection_address)?;
    let nft_royalties = compute_royalty(
        nft_for_sale_info.clone(),
        nft_collection_info,
        deps.as_ref(),
    )?;

    // Volume USDC from the oracle
    let nft_price_usdc: Uint128 =
        deps.querier
            .query::<Uint128>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.oracle_contract_address.clone(),
                msg: to_binary(&oracle::msg::QueryMsg::GetUsdcPriceFromAmountAndDenom {
                    amount: nft_for_sale_info.sale_price_value,
                    denom: nft_for_sale_info.sale_price_denom.clone(),
                })?,
            }))?;

    for nft_col in vec![
        nft_for_sale_info.nft_collection_address.clone(),
        MARKETPLACE_USDC_INDICATOR.to_string(),
    ]
    .iter()
    {
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.update(
            deps.storage,
            nft_col,
            |mp_info| -> StdResult<_> {
                let mut mp_info_u = mp_info.unwrap();
                mp_info_u.usdc_volume += nft_price_usdc;
                Ok(mp_info_u)
            },
        )?;
    }
    let mut gen_stats = GENERAL_STATS.load(deps.storage)?;
    let current_collection_usdc_volume = LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL
        .load(deps.storage, &nft_for_sale_info.nft_collection_address)?
        .usdc_volume;
    // Means we can make an update
    if current_collection_usdc_volume > gen_stats.lowest_volume_usdc
        || gen_stats.top_10_volume_usdc.len() < 10
    {
        gen_stats.compute_new_top_10_and_latest_collection_traded(CollectionVolume {
            nft_collection_address: nft_collection_address.clone(),
            usdc_volume: current_collection_usdc_volume,
        });
        GENERAL_STATS.save(deps.storage, &gen_stats)?;
    }

    // BUYER profile (sender is the buyer)
    let mut buyer_profile: Profile = Profile::new(buyer.sender.clone());
    if !PROFILES.has(deps.storage, &buyer.sender) && buyer.sender != env.contract.address {
        PROFILES.save(deps.storage, &buyer.sender, &buyer_profile)?;
    }
    // SELLER profile (nft_sale_info.seller is the seller)
    let mut seller_profile: Profile = Profile::new(nft_for_sale_info.seller.clone());
    if !PROFILES.has(deps.storage, &nft_for_sale_info.seller) && info.sender != env.contract.address
    {
        PROFILES.save(deps.storage, &nft_for_sale_info.seller, &seller_profile)?;
    }
    // Update the profiles
    let sale_price_trade_info = TradeInfo {
        denom: nft_for_sale_info.sale_price_denom.clone(),
        volume_value: nft_for_sale_info.sale_price_value,
    };

    // Seller update
    PROFILES.update(
        deps.storage,
        &nft_for_sale_info.seller,
        |profile| -> Result<_, ContractError> {
            let mut profile_u = profile.unwrap();
            seller_profile = profile_u.clone();
            profile_u = profile_u
                .realise_transaction(None, Some(sale_price_trade_info.clone()))
                .unwrap();
            profile_u = profile_u.nft_used_in_profile_check_and_reset(
                token_id.clone(),
                nft_collection_address.clone(),
            )?;
            Ok(profile_u)
        },
    )?;
    // Buyer update
    PROFILES.update(
        deps.storage,
        info.sender.as_ref(),
        |profile| -> Result<_, ContractError> {
            let mut profile_u = profile.unwrap();
            buyer_profile = profile_u.clone();
            profile_u = profile_u
                .realise_transaction(Some(sale_price_trade_info.clone()), None)
                .unwrap();
            Ok(profile_u)
        },
    )?;
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    REWARD_SYSTEM.update(
        deps.storage,
        |mut updated_reward_system| -> Result<_, ContractError> {
            let current_distribution: Uint128 = nft_price_usdc
                .checked_div(updated_reward_system.reward_token_per_1usdc_volume)
                .map_err(|_| ContractError::Generic(DivisionError {}))?
                .checked_mul(Uint128::new(2_000_000u128))
                .map_err(|_| ContractError::Generic(DivisionError {}))?;
            updated_reward_system.total_reward_tokens_distributed += current_distribution;
            Ok(updated_reward_system)
        },
    )?;
    Ok(ResponseHandler::execute_succes_nft_sale_response(
        deps.as_ref(),
        buyer,
        nft_for_sale_info,
        config.marketplace_pct_fees,
        nft_royalties,
        seller_profile,
        buyer_profile,
        reward_system,
        nft_price_usdc,
    )?
    .response)
}
