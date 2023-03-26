use std::marker::PhantomData;
use std::str::FromStr;
use cosmwasm_std::{Addr, Decimal, DepsMut, Empty, Env, from_binary, MessageInfo, QueryRequest, Response, StdResult, Timestamp, to_binary, Uint128, WasmMsg, WasmQuery};
use cw20::Cw20ReceiveMsg;
use cw721::{Cw721QueryMsg, TokensResponse};
use cw721_base::helpers::Cw721Contract;
use cw2981_multiroyalties::msg::Cw2981QueryMsg;
use nft_marketplace_utils::config::Config;
use general_utils::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, UpdateConfigEnum};
use crate::state::{CONFIG, GENERAL_STATS, MARKETPLACE_STATS_BY_DENOM, NFT_COLLECTION_VOLUME_USDC, PROFILES, REWARD_SYSTEM, TOKEN_SALE_HISTORY};
use general_utils::denominations::{DenominationValue};
use general_utils::error::GenericError::{DivisionError, InvalidDenominationReceived};
use general_utils::error::NftMarketplaceError::{AdditionalInfoNeedsToBeFilled, CantUseAdditionalInfoIfNotContract, InvalidSenderOrYouCantChangeProfileAddress, NftCollectionAlreadyExists, NftCollectionNotListed, RevokeYourApprovalBeforeCancellingSale, SaleAlreadyExists, ThisProfileDoesNotExist, YourProfileAlreadyExists};
use nft_marketplace_utils::inputs::{Buyer};
use nft_marketplace_utils::marketplace_statistics::{CollectionVolume, GeneralStats, MarketplaceStatsByDenom};
use nft_marketplace_utils::nft_collection::{define_unique_collection_by_denom_id, nft_collection_denoms, NftCollectionInfoByDenom, NftCollectionAddress, TokenId};
use nft_marketplace_utils::nft_sale::{compute_floor_collection_and_denom, define_unique_collection_nft_id, NftSale, nfts_for_sale, TokenSaleHistory};
use nft_marketplace_utils::profile::{Profile, TradeInfo};
use nft_marketplace_utils::response_handler::{ResponseHandler, RoyaltiesInfoResponse};
use nft_marketplace_utils::reward_system::{RewardSystem, VipLevel};
use crate::constants::{MARKETPLACE_USDC_INDICATOR, MAX_EXPIRATION_SECONDS, MAX_PRICE, MIN_EXPIRATION_SECONDS, MIN_PRICE};
use crate::{msg, query};

pub fn instantiate_contract(
    deps: DepsMut,
    _info: MessageInfo,
    init_msg: InstantiateMsg
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        // Validates the input -> error if invalid
        &Config::new_checked(
            deps.api,
            false,
                init_msg.contract_owner,
            init_msg.accepted_ibc_denominations.clone(),
            Decimal::from_str(&init_msg.marketplace_pct_fees_decimal_string).unwrap(),
            init_msg.marketplace_listing_fee_value,
            init_msg.marketplace_listing_fee_denom,
            init_msg.oracle_contract_address
        )?
    )?;

    REWARD_SYSTEM.save(
        deps.storage,
        &RewardSystem::new_checked(
            deps.api,
            init_msg.reward_system.reward_token_address,
            init_msg.reward_system.reward_token_per_1usdc_volume,
            init_msg.reward_system.total_reward_tokens_distributed,
            init_msg.reward_system.vip_perks,
        )?
    )?;

    // Contract states update: Marketplace statistics are for each accepted denomination
    init_msg.accepted_ibc_denominations.list_of_denoms.clone().iter()
        .try_for_each(|accepted_denom| MARKETPLACE_STATS_BY_DENOM.save(
            deps.storage,
            accepted_denom,
            &MarketplaceStatsByDenom::new(accepted_denom.clone()),
        ))?;

    NFT_COLLECTION_VOLUME_USDC.save(deps.storage, &MARKETPLACE_USDC_INDICATOR, &Uint128::zero())?;

    GENERAL_STATS.save(deps.storage,
                       &GeneralStats {
                           last_collection_added: "".to_string(),
                           last_collections_traded: vec![],
                           top_10_volume_usdc: vec![],
                           lowest_volume_usdc: Uint128::zero()
                       }
    )?;

    Ok(ResponseHandler::init_response().response)
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    list_of_updates: Vec<UpdateConfigEnum>
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    for update in list_of_updates {
        match update {
            UpdateConfigEnum::EnableDisable {} => {
                config.contract_enabled = !config.contract_enabled
            }
            UpdateConfigEnum::AddDenoms { denoms } => {
                config.accepted_ibc_denominations.add_many(denoms);
            }
            UpdateConfigEnum::RemoveDenoms { denoms } => {
                config.accepted_ibc_denominations.remove_many(denoms);
            },
            UpdateConfigEnum::UpdateOwner { address } => {
                config.contract_owner = deps.api.addr_validate(&address.to_string())?.to_string();
            },
            UpdateConfigEnum::UpdateRewardSystem { reward_system } => {
                REWARD_SYSTEM.save(deps.storage, &reward_system)?;
            },
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(ResponseHandler::update_config().response)
}

pub fn execute_add_new_nft_collection(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    nft_collection_address: NftCollectionAddress
) -> Result<Response, ContractError> {
    if NFT_COLLECTION_VOLUME_USDC.has(
        deps.storage,
        &deps.api.addr_validate(&nft_collection_address)?.to_string()
    ) {
        return Err(ContractError::NftMarketplace(NftCollectionAlreadyExists {}))
    }

    // Add all the currently accepted denoms
    let config = CONFIG.load(deps.storage)?;
    let nft_collection_address = deps.api.addr_validate(&nft_collection_address)?.to_string();
    config.accepted_ibc_denominations.list_of_denoms.iter()
        .map(|accepted_denom| {
            NftCollectionInfoByDenom::new_checked(
                deps.querier,
                nft_collection_address.clone(),
                accepted_denom.clone()
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .and_then(|collections| {
            Ok(collections.into_iter().try_for_each(|collection| {
                let collection_denom_unique = define_unique_collection_by_denom_id(
                    &nft_collection_address,
                    &collection.denom,
                );
                nft_collection_denoms().save(
                    deps.storage,
                    collection_denom_unique,
                    &collection
                )
            }))
        })?.expect("Invalid collection addition");

    NFT_COLLECTION_VOLUME_USDC.save(deps.storage, &nft_collection_address, &Uint128::zero())?;

    // Update last added collections
    GENERAL_STATS.update(deps.storage, |mut gen_stats| -> Result<_, ContractError> {
        gen_stats.last_collection_added = nft_collection_address.clone();
        Ok(gen_stats)
    })?;

    // Check if there are royalties
    let nft_tokens: TokensResponse =
        deps.querier
            .query::<TokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: nft_collection_address.clone().to_string(),
                msg: to_binary(&Cw721QueryMsg::AllTokens { start_after: None, limit: Some(1) })?,
            }))?;

    let _nft_royalties: Vec<RoyaltiesInfoResponse> = deps.querier.query::<Vec<RoyaltiesInfoResponse>>(
        &QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: nft_collection_address.clone().to_string(),
            msg: to_binary(&cw2981_multiroyalties::QueryMsg::Extension {
                msg: Cw2981QueryMsg::RoyaltyInfo {
                    token_id: nft_tokens.tokens[0].clone(),
                    sale_price: Uint128::new(100_000u128) } })? })
    )?;

    Ok(ResponseHandler::add_nft_collection(&nft_collection_address).response)
}

pub fn execute_claim_marketplace_fees(deps: DepsMut) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // States Update: Gather the fees into a vec and reset to 0
    let all_accepted_denoms_value: Vec<DenominationValue> = config
        .accepted_ibc_denominations
        .list_of_denoms
        .iter()
        .map(|denom| {
            let mut mp_fees: Uint128 = Uint128::zero();
            MARKETPLACE_STATS_BY_DENOM.update(
                deps.storage,
                &denom,
                |mp_info| -> Result<_, ContractError> {
                    let mut mp_info_u = mp_info.unwrap();
                    mp_fees = mp_info_u.marketplace_fees_to_claim.clone();
                    mp_info_u.marketplace_fees_to_claim = Uint128::zero();
                    Ok(mp_info_u)
                })?;
            Ok::<DenominationValue, ContractError>(DenominationValue {
                denom: denom.clone(),
                value: mp_fees.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Remove the Zero fees before sending the message
    let non_zero_denoms: Vec<DenominationValue> = all_accepted_denoms_value
        .into_iter()
        .filter(|dv| dv.value != Uint128::zero())
        .collect();

    Ok(ResponseHandler::claim_marketplace_fees(
        config.contract_owner.to_string(),
        non_zero_denoms,
    ).response)
}

pub fn execute_sell_nft(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut sale_info: NftSale
) -> Result<Response, ContractError> {
    sale_info.seller = deps.api.addr_validate(&sale_info.seller)?.to_string();
    sale_info.nft_collection_address = deps.api.addr_validate(&sale_info.nft_collection_address)?.to_string();

    let mut create_profile_msg: Option<WasmMsg> = None;
    if !PROFILES.has(deps.storage, &info.sender.to_string()) && info.sender != env.contract.address {
        create_profile_msg = ExecuteMsg::wasm_execute_message_create_profile(
            env.contract.address.to_string(),
            Some(info.sender.to_string())
        )?;
    }
    let config = CONFIG.load(deps.storage)?;

    // Validation: If the collection is listed
    if !NFT_COLLECTION_VOLUME_USDC.has(
        deps.storage,
        &sale_info.nft_collection_address,
    ) {
        return Err(ContractError::NftMarketplace(NftCollectionNotListed {}))
    };

    // If sender is the contract it means:
    // A. It is for updating a sale or
    // B. We create a sale for an "accept offer" -> will be possible in the next version
    if info.sender == env.contract.address {
        info.sender = deps.api.addr_validate(&sale_info.seller)?;
    }

    // Validation: Validate all the info from the given NftSale information given by the sender
    let nft_for_sale_validated: NftSale = NftSale::new_checked(
        deps.as_ref(),
        &env.block.time.seconds(),
        &info,
        &sale_info,
        config.clone(),
        env.contract.address.to_string(),
        MAX_EXPIRATION_SECONDS,
        MIN_EXPIRATION_SECONDS,
        MAX_PRICE,
        MIN_PRICE
    )?;

    // Contract states update: NFT Collection stats
    MARKETPLACE_STATS_BY_DENOM.update(
        deps.storage,
        &*nft_for_sale_validated.sale_price_denom,
        |mp_info| -> Result<_, ContractError> {
            Ok(mp_info.unwrap().list_nft_for_sale().clone())
    })?;

    // Validation: Check if the sale already exists
    let collection_token_id_unique: String = define_unique_collection_nft_id(
        &nft_for_sale_validated.nft_collection_address,
        &nft_for_sale_validated.token_id,
    );
    if nfts_for_sale().has(deps.storage, collection_token_id_unique.clone()) {
        return Err(ContractError::NftMarketplace(SaleAlreadyExists {}))
    }

    // Contract states update: NFT Collection stats
    let collection_denom_unique: String = define_unique_collection_by_denom_id(
        &nft_for_sale_validated.nft_collection_address,
        &nft_for_sale_validated.sale_price_denom,
    );
    if !(nft_collection_denoms().has(deps.storage, collection_denom_unique.clone())) {
        // The denom did not exist at the time of adding the collection - add it
        nft_collection_denoms().save(
            deps.storage,
            collection_denom_unique.clone(),
            &NftCollectionInfoByDenom::new_checked(
                deps.querier.clone(),
                nft_for_sale_validated.nft_collection_address.clone(),
                nft_for_sale_validated.sale_price_denom.clone()
            )?)?;
    }
    nft_collection_denoms().update(
        deps.storage,
        collection_denom_unique.clone(),
        |nft_coll_denom| -> Result<_, ContractError> {
            Ok(nft_coll_denom.unwrap().register_sale(nft_for_sale_validated.clone()))
        })?;

    // Contract states update: Add the Sale info to all the other sales
    nfts_for_sale().save(
        deps.storage,
        collection_token_id_unique,
        &nft_for_sale_validated.clone()
    )?;

    Ok(ResponseHandler::register_nft_sale_response(
        nft_for_sale_validated,
        create_profile_msg).response)
}

pub fn execute_update_nft_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut sale_info: NftSale
) -> Result<Response, ContractError> {
    sale_info.nft_collection_address = deps.api.addr_validate(&sale_info.nft_collection_address)?.to_string();
    sale_info.seller = deps.api.addr_validate(&sale_info.seller)?.to_string();

    let config = CONFIG.load(deps.storage)?;

    // Validation: If the Sale exists
    let collection_token_id_unique = define_unique_collection_nft_id(
        &sale_info.nft_collection_address.clone(),
        &sale_info.token_id.clone(),
    );
    let mut nft_for_sale_info = nfts_for_sale().load(deps.storage, collection_token_id_unique.clone())?;

    // Validation: Sender == Owner
    // Validate: Sender is the owner
    let owner_response = Cw721Contract::<Empty, Empty>(
        deps.api.addr_validate(&sale_info.nft_collection_address.clone())?,
        PhantomData,
        PhantomData)
        .owner_of(
            &deps.querier,
            sale_info.token_id.clone().to_string(),
            false
        )?;
    nft_for_sale_info = nft_for_sale_info.validate_sender_is_token_owner(
        &info.sender.to_string(),
        &env.contract.address.to_string(),
        &owner_response.owner.to_string()
    )?;

    // Updating a sale = Cancelling + Adding a new sale with waived fees
    let cancel_sale_msg = ExecuteMsg::wasm_execute_message_cancel_sale(
        nft_for_sale_info.nft_collection_address.clone(),
        env.contract.address.to_string(),
        nft_for_sale_info.token_id.clone(),
        nft_for_sale_info.seller.clone()
    )?;

    // Then list it back with new info
    let execute_sale_msg = ExecuteMsg::wasm_execute_message_sell(
        config.marketplace_listing_fee_value.clone(),
        env.contract.address.to_string(),
        sale_info.clone(),
        config.marketplace_listing_fee_denom.clone()
    )?;

    return Ok(ResponseHandler::execute_update_sale(
        cancel_sale_msg.unwrap(),
        execute_sale_msg.unwrap())
        .response
    )
}

pub fn execute_cancel_nft_sale(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    additional_info: Option<String>
) -> Result<Response, ContractError> {
    nft_collection_address = deps.api.addr_validate(&nft_collection_address)?.to_string();
    // Validation: additional_info can only be used by contract
    // When the contract uses this entry point it is for a cancellation for:
    // A. an update of a sale
    // B. Offer acceptation
    if let Some(addr) = &additional_info {
        if info.sender != env.contract.address {
            return Err(ContractError::NftMarketplace(CantUseAdditionalInfoIfNotContract {}));
        }
        info = MessageInfo {
            sender: deps.api.addr_validate(&addr)?,
            funds: info.funds.clone(),
        };
    } else if info.sender == env.contract.address {
        return Err(ContractError::NftMarketplace(AdditionalInfoNeedsToBeFilled {}));
    }

    // Validate: Only the NFT owner can cancel a sale (or the contract for an offer change)
    // Validate: Sender is the owner
    let owner_response = Cw721Contract::<Empty, Empty>(
        deps.api.addr_validate(&nft_collection_address)?,
        PhantomData,
        PhantomData)
        .owner_of(
            &deps.querier,
            token_id.clone().to_string(),
            false
        )?;
    let collection_token_id_unique = define_unique_collection_nft_id(&nft_collection_address, &token_id);

    let mut nft_for_sale_info = nfts_for_sale().load(deps.storage, collection_token_id_unique.clone())?;
    nft_for_sale_info = nft_for_sale_info.validate_sender_is_token_owner(
        &info.sender.to_string(),
        &env.contract.address.to_string(),
        &owner_response.owner.to_string()
    )?;

    // Validate: Need to revoke an approval before cancelling a Sale (otherwise a Transfer auto-cancels)
    if !additional_info.is_some() {
        let cw721_contract = Cw721Contract::<Empty, Empty>(
            Addr::unchecked(nft_collection_address.to_string()),
            PhantomData,
            PhantomData,
        );
        if cw721_contract.approval(
            &deps.querier,
            token_id.clone(),
            env.contract.address.to_string(),
            None,
        ).is_ok() {
            return Err(ContractError::NftMarketplace(RevokeYourApprovalBeforeCancellingSale {}));
        }
    }

    // Update: Marketplace Statistics
    MARKETPLACE_STATS_BY_DENOM.update(
        deps.storage,
        &*nft_for_sale_info.sale_price_denom.clone().to_string(),
        |mp_info| -> Result<_, ContractError> {
            Ok(mp_info.unwrap().remove_nft_for_sale().clone())
        })?;

    // Update: Remove the Sale
    let collection_denom_unique = define_unique_collection_by_denom_id(
        &nft_for_sale_info.nft_collection_address,
        &nft_for_sale_info.sale_price_denom,
    );

    nfts_for_sale().remove(deps.storage, collection_token_id_unique)?;

    // Execute: Compute new floor; if needed and update collection's stats (floor f.e.)
    let new_floor = compute_floor_collection_and_denom(
        deps.storage,
        nft_for_sale_info.sale_price_denom.clone(),
        nft_for_sale_info.nft_collection_address.clone(),
        MAX_PRICE
    )?;

    nft_collection_denoms().update(
        deps.storage,
        collection_denom_unique,
        |nft_coll_denom| -> Result<_, ContractError> {
            Ok(nft_coll_denom.unwrap().remove_sale(new_floor.clone()))
        })?;
    Ok(ResponseHandler::cancel_nft_sale_response(nft_for_sale_info.clone()).response)
}

pub fn execute_buy_nft(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    additional_info: Option<String>
) -> Result<Response, ContractError> {
    // Validate addresses
    nft_collection_address = deps.api.addr_validate(&nft_collection_address)?.to_string();

    if let Some(additional_info) = additional_info {
        // additional_info is not None
        if info.sender != env.contract.address {
            return Err(ContractError::NftMarketplace(CantUseAdditionalInfoIfNotContract {}))
        }
        info = MessageInfo {
            sender: deps.api.addr_validate(&additional_info.to_string())?,
            funds: info.funds.clone()
        };
    } else {
        // additional_info is None
        if info.sender == env.contract.address {
            return Err(ContractError::NftMarketplace(AdditionalInfoNeedsToBeFilled {}))
        }
    }

    let config = CONFIG.load(deps.storage)?;

    let buyer = Buyer::new_checked(
        deps.api.addr_validate(&*info.sender.to_string())?,
        info.funds
    )?;

    let nft_for_sale_info = query::query_nft_for_sale(
        deps.as_ref(),
        nft_collection_address.clone(),
        token_id.clone()
    )?;

    let collection_token_id_unique = define_unique_collection_nft_id(
        &nft_collection_address,
        &token_id,
    );

    nft_for_sale_info.clone().validate_buying_information(&buyer)?;

    // If valid, can remove
    nfts_for_sale().remove(deps.storage, collection_token_id_unique.clone())?;

    if nft_for_sale_info.clone().sale_expiration.seconds() <= env.block.time.seconds() {
        MARKETPLACE_STATS_BY_DENOM
            .update(deps.storage, &nft_for_sale_info.clone().sale_price_denom.to_string(), |mp_info| -> Result<_, ContractError> {
                let mut mp_info = mp_info.unwrap();
                mp_info.remove_nft_for_sale();
                Ok(mp_info)
            })?;
        return Ok(ResponseHandler::expired_nft_sale_response(buyer.clone()).response)
    }

    MARKETPLACE_STATS_BY_DENOM.update(deps.storage, &nft_for_sale_info.clone().sale_price_denom.to_string(), |mp_info| -> Result<_, ContractError> {
        let mut mp_info = mp_info.unwrap();
        mp_info.execute_sale(
            nft_for_sale_info.clone().sale_price_value,
            config.marketplace_pct_fees.clone()
        );
        Ok(mp_info)
    })?;

    let collection_denom_unique: String = define_unique_collection_by_denom_id(
        &nft_for_sale_info.nft_collection_address,
        &nft_for_sale_info.sale_price_denom,
    );

    let new_floor = compute_floor_collection_and_denom(deps.storage,
                                                       nft_for_sale_info.clone().sale_price_denom,
                                                       nft_for_sale_info.clone().nft_collection_address,
                                                       MAX_PRICE)?;
    nft_collection_denoms().update(deps.storage, collection_denom_unique, |nft_coll_denom| -> Result<_, ContractError> {
        let mut nft_coll_denom = nft_coll_denom.unwrap();
        nft_coll_denom = nft_coll_denom.execute_sale(nft_for_sale_info.clone().sale_price_value, new_floor);
        Ok(nft_coll_denom)
    })?;

    let maybe_history: Option<Vec<TokenSaleHistory>> = TOKEN_SALE_HISTORY.may_load(deps.storage, &collection_token_id_unique.clone())?;
    let transaction_info = TokenSaleHistory {
        seller: nft_for_sale_info.seller.clone(),
        buyer: buyer.sender.clone(),
        nft_collection_address: nft_for_sale_info.nft_collection_address.clone(),
        token_id: nft_for_sale_info.token_id.clone(),
        sale_price_value: nft_for_sale_info.sale_price_value.clone(),
        sale_price_denom: nft_for_sale_info.sale_price_denom.clone(),
        sale_time: Timestamp::from_seconds(env.block.time.seconds())
    };
    let mut new_vector_of_transactions = maybe_history.unwrap_or_else(|| vec![]);
    new_vector_of_transactions.push(transaction_info.clone());
    TOKEN_SALE_HISTORY.save(deps.storage, &collection_token_id_unique, &new_vector_of_transactions)?;

    // Calculation of royalties and marketplace revenues
    let nft_royalties: Vec<RoyaltiesInfoResponse> = deps.querier.query::<Vec<RoyaltiesInfoResponse>>(
        &QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: nft_for_sale_info.nft_collection_address.clone(),
            msg: to_binary(&cw2981_multiroyalties::QueryMsg::Extension {
                msg: Cw2981QueryMsg::RoyaltyInfo {
                    token_id: nft_for_sale_info.token_id.clone(),
                    sale_price: nft_for_sale_info.sale_price_value.clone() } })? })
    )?;

    // Volume USDC from the oracle
    let oracle_price: Uint128 =
        deps.querier
            .query::<Uint128>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.oracle_contract_address.clone(),
                msg: to_binary(&crypto_assets_price_oracle::msg::QueryMsg::GetUsdcPriceFromAmountAndDenom {
                    amount: nft_for_sale_info.sale_price_value.clone(),
                    denom: nft_for_sale_info.sale_price_denom.clone()
                })?, }))?;

    let nft_coll_to_update = vec![
        nft_for_sale_info.nft_collection_address.clone(),
        MARKETPLACE_USDC_INDICATOR.to_string(),
    ];

    for nft_col in nft_coll_to_update.iter() {
        NFT_COLLECTION_VOLUME_USDC.update(
            deps.storage,
            nft_col,
            |mp_info| -> StdResult<_> {
                let mut mp_info_u = mp_info.unwrap();
                mp_info_u += oracle_price.clone();
                Ok(mp_info_u)
            },
        )?;
    }
    let mut gen_stats = GENERAL_STATS.load(deps.storage)?;
    let current_collection_usdc_volume = NFT_COLLECTION_VOLUME_USDC.load(
        deps.storage, &nft_for_sale_info.nft_collection_address)?;
    // Means we can make an update
    if &current_collection_usdc_volume > &gen_stats.lowest_volume_usdc || gen_stats.top_10_volume_usdc.len() < 10 {
        gen_stats.compute_new_top_10_and_latest_collection_traded(CollectionVolume {
            nft_collection_address: nft_collection_address.clone(),
            usdc_volume: current_collection_usdc_volume.clone(),
        });
        GENERAL_STATS.save(deps.storage, &gen_stats)?;
    }

    // Update the profiles
    if !PROFILES.has(deps.storage, info.sender.as_ref()) && info.sender != env.contract.address {
        let new_profile = Profile::new(info.sender.to_string());
        PROFILES.save(deps.storage, info.sender.as_ref(), &new_profile)?;
    }

    // Update the profiles
    let sale_price_trade_info = TradeInfo {
        denom: nft_for_sale_info.sale_price_denom.clone(),
        volume_value: nft_for_sale_info.sale_price_value.clone(),
    };
    if !PROFILES.has(deps.storage, info.sender.clone().as_ref()) && info.sender != env.contract.address {
        let new_profile = Profile::new(info.sender.to_string());
        PROFILES.save(deps.storage, &info.sender.to_string(), &new_profile)?;
    }
    let mut seller_profile: Profile = Profile::new(info.sender.to_string());
    PROFILES.update(
        deps.storage,
        &nft_for_sale_info.seller.clone(),
        |profile| -> Result<_, ContractError> {
            let mut profile_u = profile.unwrap();
            seller_profile = profile_u.clone();
            profile_u = profile_u.realise_transaction(
                None,
                Some(sale_price_trade_info.clone())).unwrap();
            Ok(profile_u)
        })?;
    let mut buyer_profile: Profile = Profile::new(buyer.sender.clone());
    PROFILES.update(
        deps.storage,
        &info.sender.to_string(),
        |profile| -> Result<_, ContractError> {
            let mut profile_u = profile.unwrap();
            buyer_profile = profile_u.clone();
            profile_u = profile_u.realise_transaction(
                Some(sale_price_trade_info.clone()),
                None).unwrap();
            Ok(profile_u)
        })?;
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    REWARD_SYSTEM.update(deps.storage, |mut updated_reward_system| -> Result<_, ContractError> {
        let current_distribution: Uint128 = oracle_price.clone()
            .checked_div(updated_reward_system.reward_token_per_1usdc_volume)
            .map_err(|_| ContractError::Generic(DivisionError {}))?
            .checked_mul(Uint128::new(2_000_000u128))
            .map_err(|_| ContractError::Generic(DivisionError {}))?;
        updated_reward_system.total_reward_tokens_distributed += current_distribution;
        Ok(updated_reward_system)
    })?;
    Ok(ResponseHandler::execute_succes_nft_sale_response(
        deps.as_ref(),
        buyer.clone(),
        nft_for_sale_info.clone(),
        config.marketplace_pct_fees,
        nft_royalties.into(),
        seller_profile,
        buyer_profile,
        reward_system,
        oracle_price.clone()
    ).unwrap().response)
}

pub fn execute_create_profile(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    profile: Option<Profile>,
    additional_info: Option<String>
) -> Result<Response, ContractError> {
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    // Validate: who is the sender
    if let Some(addr) = &additional_info {
        if info.sender != env.contract.address {
            return Err(ContractError::NftMarketplace(CantUseAdditionalInfoIfNotContract {}));
        }
        info = MessageInfo {
            sender: deps.api.addr_validate(&addr.to_string())?,
            funds: info.funds.clone(),
        };
    } else if info.sender == env.contract.address {
        return Err(ContractError::NftMarketplace(AdditionalInfoNeedsToBeFilled {}));
    }
    let sender_addr = deps.api.addr_validate(&info.sender.to_string())?.to_string();
    if PROFILES.has(deps.storage, &sender_addr) {
        return Err(ContractError::NftMarketplace(YourProfileAlreadyExists {}))
    }
    let new_profile = Profile::new(info.sender.to_string()).user_update_profile(profile, reward_system);
    PROFILES.save(deps.storage, &sender_addr, &new_profile)?;
    Ok(ResponseHandler::create_or_update_profile(new_profile).unwrap().response)
}

pub fn execute_update_profile(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    profile: Profile,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_validate(&info.sender.to_string())?.to_string();
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;

    match PROFILES.load(deps.storage, &sender_addr) {
        Ok(loaded_profile) => {
            if sender_addr != profile.address {
                return Err(ContractError::NftMarketplace(InvalidSenderOrYouCantChangeProfileAddress {}));
            }
            let updated_profile = loaded_profile.clone().user_update_profile(Some(profile), reward_system);
            PROFILES.save(deps.storage, &sender_addr, &updated_profile)?;
            Ok(ResponseHandler::create_or_update_profile(updated_profile)?.response)
        }
        Err(_) => Err(ContractError::NftMarketplace(ThisProfileDoesNotExist {})),
    }
}

fn execute_level_up_profile(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    mut user_sender: String,
    cw20_amount: Uint128
) -> Result<Response, ContractError> {
    user_sender = deps.api.addr_validate(&user_sender)?.to_string();
    let reward_system: RewardSystem = REWARD_SYSTEM.load(deps.storage)?;
    let mut loaded_profile: Profile = PROFILES.load(deps.storage, &user_sender.to_string())?;
    let previous_level = loaded_profile.vip_level.clone();
    if VipLevel::level_up_if_possible(
        loaded_profile.vip_level.clone().unwrap(),
        reward_system.vip_perks,
        cw20_amount
    )? {
        loaded_profile = loaded_profile.level_up();
        PROFILES.save(deps.storage, &user_sender.to_string(), &loaded_profile)?;
    }
    Ok(ResponseHandler::level_up_profile(previous_level.unwrap(), loaded_profile.vip_level.unwrap())?.response)
}

pub fn execute_receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
    _env: Env,
) -> Result<Response, ContractError> {
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    let cw20_msg: msg::ReceiveMsg = from_binary(&wrapper.msg)?;
    let cw20_denom = deps.api.addr_validate(&info.sender.to_string())?;
    if cw20_denom != reward_system.reward_token_address {
        return Err(ContractError::Generic(InvalidDenominationReceived {}))
    }
    let user_sender = wrapper.sender.clone();
    let cw20_amount = wrapper.amount.clone();
    match cw20_msg {
        msg::ReceiveMsg::LevelUpProfile {} => {
            execute_level_up_profile(deps, _env, info, user_sender, cw20_amount)
        }
    }
}