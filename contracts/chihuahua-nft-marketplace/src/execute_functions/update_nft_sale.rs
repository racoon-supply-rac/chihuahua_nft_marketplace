use std::marker::PhantomData;

use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};
use cw721_base::helpers::Cw721Contract;

use general_utils::error::ContractError;
use nft_marketplace_utils::nft_sale::{define_unique_collection_nft_id, nfts_for_sale, NftSale};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::msg::ExecuteMsg;
use crate::state::CONFIG;

pub fn execute_update_nft_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut sale_info: NftSale,
) -> Result<Response, ContractError> {
    sale_info.nft_collection_address = deps
        .api
        .addr_validate(&sale_info.nft_collection_address)?
        .to_string();
    sale_info.seller = deps.api.addr_validate(&sale_info.seller)?.to_string();

    let config = CONFIG.load(deps.storage)?;

    // Validation: If the Sale exists
    let collection_token_id_unique =
        define_unique_collection_nft_id(&sale_info.nft_collection_address, &sale_info.token_id);
    let mut nft_for_sale_info = nfts_for_sale().load(deps.storage, collection_token_id_unique)?;

    // Validate: Sender is the owner
    let owner_response = Cw721Contract::<Empty, Empty>(
        deps.api.addr_validate(&sale_info.nft_collection_address)?,
        PhantomData,
        PhantomData,
    )
    .owner_of(&deps.querier, sale_info.token_id.to_string(), false)?;
    nft_for_sale_info = nft_for_sale_info.validate_sender_is_token_owner(
        info.sender.as_ref(),
        env.contract.address.as_ref(),
        &owner_response.owner,
    )?;

    // Updating a sale = Cancelling + Adding a new sale with waived fees
    let cancel_sale_msg = ExecuteMsg::wasm_execute_message_cancel_sale(
        nft_for_sale_info.nft_collection_address.clone(),
        env.contract.address.to_string(),
        nft_for_sale_info.token_id.clone(),
        nft_for_sale_info.seller,
    )?;

    // Then list it back with new info
    let execute_sale_msg = ExecuteMsg::wasm_execute_message_sell(
        config.marketplace_listing_fee_value,
        env.contract.address.to_string(),
        sale_info,
        config.marketplace_listing_fee_denom,
    )?;

    Ok(
        ResponseHandler::execute_update_sale(cancel_sale_msg.unwrap(), execute_sale_msg.unwrap())
            .response,
    )
}
