use cosmwasm_std::{ensure, DepsMut, Env, MessageInfo, Response, WasmMsg};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{NftCollectionNotListed, OfferAlreadyExists};
use nft_marketplace_utils::config::Config;
use nft_marketplace_utils::nft_offer::{define_unique_offer, nft_offers, NftOffer};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::constants::{MAX_EXPIRATION_SECONDS, MAX_PRICE, MIN_EXPIRATION_SECONDS, MIN_PRICE};
use crate::msg::ExecuteMsg;
use crate::state::{CONFIG, LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL, PROFILES};

pub fn execute_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut offer: NftOffer,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // Validate addresses
    offer.nft_collection_address = deps
        .api
        .addr_validate(&offer.nft_collection_address)?
        .to_string();
    offer.offerer_address = deps.api.addr_validate(&offer.offerer_address)?.to_string();

    // Validation: If the collection is listed
    ensure!(
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.has(deps.storage, &offer.nft_collection_address),
        ContractError::NftMarketplaceError(NftCollectionNotListed {})
    );

    // If the profile of the sender does not exist -> create it
    let mut create_profile_msg: Option<WasmMsg> = None;
    if !PROFILES.has(deps.storage, info.sender.as_ref()) && info.sender != env.contract.address {
        create_profile_msg = ExecuteMsg::wasm_execute_message_create_profile(
            env.contract.address.to_string(),
            Some(info.sender.to_string()),
        )?;
    }

    let unique_offer_id = define_unique_offer(
        &offer.nft_collection_address,
        &offer.token_id,
        &offer.offerer_address.to_string(),
    );

    // Validate: Cant create an existing offer (collection + token id + offerer)
    if nft_offers().has(deps.storage, unique_offer_id.clone()) {
        return Err(ContractError::NftMarketplaceError(OfferAlreadyExists {}));
    }

    // Validate: Price, denom, sender, owner
    let nft_offer_validated: NftOffer = NftOffer::new_checked(
        deps.as_ref(),
        offer,
        &info,
        env.block.time.seconds(),
        config.accepted_ibc_denominations,
        MAX_EXPIRATION_SECONDS,
        MIN_EXPIRATION_SECONDS,
        MAX_PRICE,
        MIN_PRICE,
    )?;

    // Update: Save the offer if offer is valid
    nft_offers().save(deps.storage, unique_offer_id, &nft_offer_validated)?;

    Ok(ResponseHandler::nft_offer_response(nft_offer_validated, create_profile_msg).response)
}
