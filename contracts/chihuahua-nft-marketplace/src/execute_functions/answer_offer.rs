use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, WasmMsg};

use general_utils::error::ContractError;
use nft_marketplace_utils::nft_collection::{NftCollectionAddress, TokenId};
use nft_marketplace_utils::nft_offer::{define_unique_offer, nft_offers};
use nft_marketplace_utils::nft_sale::{define_unique_collection_nft_id, nfts_for_sale, NftSale};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::msg::ExecuteMsg;
use crate::state::CONFIG;

#[allow(clippy::too_many_arguments)]
pub fn execute_answer_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    from: String,
    if_accepted: bool,
    answer_msg: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let unique_offer = define_unique_offer(&nft_collection_address, &token_id, &from);

    // Validate: Offer needs to exist
    let nft_offer_loaded = nft_offers().load(deps.storage, unique_offer.clone())?;

    // If offer expired OR rejected -> cancel the offer + reimburse
    if !if_accepted
        || nft_offer_loaded.offer_expiration.clone().seconds() < env.block.time.seconds()
    {
        // If expired or rejected, outcome should be the same as if the offer is cancelled
        // Cancel an offer refunds the offerr
        let exec_cancel_sale_msg_inner = ExecuteMsg::wasm_execute_cancel_offer(
            nft_collection_address,
            env.contract.address.to_string(),
            token_id,
            nft_offer_loaded.offerer_address.clone(),
        )?;

        return Ok(ResponseHandler::execute_rejected_or_expired_offer(
            exec_cancel_sale_msg_inner.unwrap(),
            nft_offer_loaded,
            if_accepted,
            match answer_msg {
                Some(string) => string,
                None => "None".to_string(),
            },
        )
        .unwrap()
        .response);
    }

    // Execute: Remove if it exists
    nft_offers().remove(deps.storage, unique_offer)?;

    // Below: If sale exists, cancel
    // If sale does not exists ok
    // Then: create a new sale with the offer info
    // Then: execute the sale

    // Validate + Execute: If NFT was for sale, need to cancel the sale
    let collection_token_id_unique: String =
        define_unique_collection_nft_id(&nft_collection_address, &token_id);
    let not_for_sale = nfts_for_sale()
        .load(deps.storage, collection_token_id_unique)
        .is_err();
    let execute_cancel_message: Option<WasmMsg> = if !not_for_sale {
        ExecuteMsg::wasm_execute_message_cancel_sale(
            nft_collection_address.clone(),
            env.contract.address.to_string(),
            token_id.clone(),
            info.sender.to_string(),
        )?
    } else {
        None
    };

    // Create new sale with offer info
    let execute_sell_nft_msg_inner = ExecuteMsg::wasm_execute_message_sell(
        config.marketplace_listing_fee_value,
        env.contract.address.to_string(),
        NftSale {
            seller: info.sender.to_string(),
            nft_collection_address: nft_collection_address.clone(),
            token_id: token_id.clone(),
            sale_price_value: nft_offer_loaded.offer_price_value,
            sale_price_denom: nft_offer_loaded.offer_price_denom.clone(),
            sale_expiration: nft_offer_loaded.offer_expiration,
        },
        config.marketplace_listing_fee_denom,
    )?;

    // Accept the sale / Buy NFT from offerer
    let execute_buy_nft_msg_inner = ExecuteMsg::wasm_execute_buy_nft(
        nft_collection_address,
        env.contract.address.to_string(),
        token_id,
        nft_offer_loaded.offerer_address.clone(),
        nft_offer_loaded.offer_price_value,
        nft_offer_loaded.offer_price_denom.clone(),
    )?;

    Ok(ResponseHandler::execute_accept_offer(
        info.sender.to_string(),
        nft_offer_loaded.offerer_address,
        execute_cancel_message,
        execute_sell_nft_msg_inner.unwrap(),
        execute_buy_nft_msg_inner.unwrap(),
        match answer_msg {
            Some(string) => string,
            None => "None".to_string(),
        },
    )
    .response)
}
