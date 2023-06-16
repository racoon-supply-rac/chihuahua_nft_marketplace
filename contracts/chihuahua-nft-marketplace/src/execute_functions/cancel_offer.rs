use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    AdditionalInfoNeedsToBeFilled, CantUseAdditionalInfoIfNotContract,
};
use nft_marketplace_utils::nft_collection::{NftCollectionAddress, TokenId};
use nft_marketplace_utils::nft_offer::{define_unique_offer, nft_offers};
use nft_marketplace_utils::response_handler::ResponseHandler;

pub fn execute_cancel_offer(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    additional_info: Option<String>,
) -> Result<Response, ContractError> {
    // If the sender is the contract, can be cancelled
    let sender = match (additional_info.clone(), info.sender.clone()) {
        (Some(_), sender) if sender != env.contract.address => {
            return Err(ContractError::NftMarketplaceError(
                CantUseAdditionalInfoIfNotContract {},
            ));
        }
        (None, sender) if sender == env.contract.address => {
            return Err(ContractError::NftMarketplaceError(
                AdditionalInfoNeedsToBeFilled {},
            ));
        }
        (Some(addr), _) => addr,
        (None, sender) => sender.into(),
    };
    let unique_offer = define_unique_offer(&nft_collection_address, &token_id, &sender);

    // Validate: Does the offer exists
    let nft_offer_loaded = nft_offers().load(deps.storage, unique_offer.clone())?;

    // Validate: If the offer exists; sender and offerer should be the same
    if nft_offer_loaded.offerer_address != info.sender && info.sender == env.contract.address {
        info.sender = deps.api.addr_validate(&additional_info.unwrap())?;
    }

    // Update: Remove the offer
    nft_offers().remove(deps.storage, unique_offer)?;

    Ok(ResponseHandler::nft_cancel_offer_response(nft_offer_loaded).response)
}
