use cosmwasm_std::{ensure, DepsMut, Env, MessageInfo, Response, WasmMsg};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    CantCancelASaleYouDontOwn, NftCollectionNotListed,
};
use nft_marketplace_utils::nft_collection::{NftCollectionAddress, TokenId};
use nft_marketplace_utils::nft_sale::{
    check_if_sender_is_owner_token_id_exists_and_can_transfer, define_unique_collection_nft_id,
    nfts_for_sale,
};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::msg::ExecuteMsg;
use crate::state::{LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL, PROFILES};

pub fn execute_transfer_my_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    mut recipient: String,
) -> Result<Response, ContractError> {
    recipient = deps.api.addr_validate(&recipient)?.to_string();

    let mut create_profile_msg: Option<WasmMsg> = None;
    if !PROFILES.has(deps.storage, info.sender.as_ref()) {
        create_profile_msg = ExecuteMsg::wasm_execute_message_create_profile(
            env.contract.address.to_string(),
            Some(info.sender.to_string()),
        )?;
    }

    // Validation: If the collection is listed
    ensure!(
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.has(deps.storage, &nft_collection_address),
        ContractError::NftMarketplaceError(NftCollectionNotListed {})
    );

    check_if_sender_is_owner_token_id_exists_and_can_transfer(
        deps.as_ref(),
        &nft_collection_address,
        token_id.clone(),
        info.sender.to_string(),
        env.contract.address.to_string(),
    )?;

    // Validate: is it for sale? If yes, cancel the sale
    let collection_token_id_unique: String =
        define_unique_collection_nft_id(&nft_collection_address, &token_id);
    let nft_for_sale_info = nfts_for_sale().load(deps.storage, collection_token_id_unique);
    let exec_cancel_sale_msg_inner = match nft_for_sale_info {
        Ok(sale) => {
            ensure!(
                sale.seller == info.sender,
                ContractError::NftMarketplaceError(CantCancelASaleYouDontOwn {})
            );
            Some(
                ExecuteMsg::wasm_execute_message_cancel_sale(
                    nft_collection_address.to_string(),
                    env.contract.address.to_string(),
                    token_id.clone(),
                    info.sender.to_string(),
                )?
                .unwrap(),
            )
        }
        Err(_) => None,
    };

    // Sender profile to be updated
    if PROFILES.has(deps.storage, info.sender.as_ref()) {
        PROFILES.update(
            deps.storage,
            info.sender.as_ref(),
            |profile| -> Result<_, ContractError> {
                let mut profile_u = profile.unwrap();
                profile_u = profile_u.nft_used_in_profile_check_and_reset(
                    token_id.clone(),
                    nft_collection_address.clone(),
                )?;
                Ok(profile_u)
            },
        )?;
    }

    Ok(ResponseHandler::transfer_my_nft(
        token_id,
        nft_collection_address,
        recipient,
        create_profile_msg,
        exec_cancel_sale_msg_inner,
    )?
    .response)
}
