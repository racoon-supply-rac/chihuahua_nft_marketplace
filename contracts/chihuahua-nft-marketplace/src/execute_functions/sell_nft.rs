use cosmwasm_std::{ensure, DepsMut, Env, MessageInfo, Response, WasmMsg};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{NftCollectionNotListed, SaleAlreadyExists};
use nft_marketplace_utils::nft_collection::{
    define_unique_collection_by_denom_id, nft_collection_denoms, NftCollectionInfoByDenom,
};
use nft_marketplace_utils::nft_sale::{define_unique_collection_nft_id, nfts_for_sale, NftSale};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::constants::{MAX_EXPIRATION_SECONDS, MAX_PRICE, MIN_EXPIRATION_SECONDS, MIN_PRICE};
use crate::msg::ExecuteMsg;
use crate::state::{
    CONFIG, LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL, MARKETPLACE_STATS_BY_DENOM, PROFILES,
};

pub fn execute_sell_nft(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut sale_info: NftSale,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Input validation
    sale_info.seller = deps.api.addr_validate(&sale_info.seller)?.to_string();
    sale_info.nft_collection_address = deps
        .api
        .addr_validate(&sale_info.nft_collection_address)?
        .to_string();

    // If the profile does not exist -> create it
    let mut create_profile_msg: Option<WasmMsg> = None;
    if !PROFILES.has(deps.storage, info.sender.as_ref()) && info.sender != env.contract.address {
        create_profile_msg = ExecuteMsg::wasm_execute_message_create_profile(
            env.contract.address.to_string(),
            Some(info.sender.to_string()),
        )?;
    }

    // Validation: If the collection is listed
    ensure!(
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL
            .has(deps.storage, &sale_info.nft_collection_address),
        ContractError::NftMarketplaceError(NftCollectionNotListed {})
    );

    // If sender is the contract it means:
    // A. It is for updating a sale or
    // B. We create a sale for an "accept offer"
    if info.sender == env.contract.address {
        info.sender = deps.api.addr_validate(&sale_info.seller)?;
    } else {
        // Means we can add the marketplace listing fees (so we dont duplicate when its the contract)
        MARKETPLACE_STATS_BY_DENOM.update(
            deps.storage,
            &*config.marketplace_listing_fee_denom,
            |mp_info| -> Result<_, ContractError> {
                Ok(mp_info
                    .unwrap()
                    .add_listing_fees(&config.marketplace_listing_fee_value)
                    .clone())
            },
        )?;
    }

    // Validation: Validate all the info from the given NftSale information given by the sender
    let nft_for_sale_validated: NftSale = NftSale::new_checked(
        deps.as_ref(),
        &env.block.time.seconds(),
        &info,
        &sale_info,
        config,
        env.contract.address.to_string(),
        MAX_EXPIRATION_SECONDS,
        MIN_EXPIRATION_SECONDS,
        MAX_PRICE,
        MIN_PRICE,
    )?;

    // Contract states update: NFT Collection stats
    MARKETPLACE_STATS_BY_DENOM.update(
        deps.storage,
        &*nft_for_sale_validated.sale_price_denom,
        |mp_info| -> Result<_, ContractError> { Ok(mp_info.unwrap().list_nft_for_sale().clone()) },
    )?;

    // Validation: Check if the sale already exists
    let collection_token_id_unique: String = define_unique_collection_nft_id(
        &nft_for_sale_validated.nft_collection_address,
        &nft_for_sale_validated.token_id,
    );
    if nfts_for_sale().has(deps.storage, collection_token_id_unique.clone()) {
        return Err(ContractError::NftMarketplaceError(SaleAlreadyExists {}));
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
                deps.querier,
                nft_for_sale_validated.nft_collection_address.clone(),
                nft_for_sale_validated.sale_price_denom.clone(),
            )?,
        )?;
    }
    nft_collection_denoms().update(
        deps.storage,
        collection_denom_unique,
        |nft_coll_denom| -> Result<_, ContractError> {
            Ok(nft_coll_denom
                .unwrap()
                .register_sale(nft_for_sale_validated.clone()))
        },
    )?;

    // Contract states update: Add the Sale info to all the other sales
    nfts_for_sale().save(
        deps.storage,
        collection_token_id_unique,
        &nft_for_sale_validated,
    )?;

    Ok(
        ResponseHandler::register_nft_sale_response(nft_for_sale_validated, create_profile_msg)
            .response,
    )
}
