#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, StdResult};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw2;
use general_utils::error::ContractError;
use general_utils::validations::{if_admin, if_enabled, validate_address};

use crate::execute_functions::{
    add_new_nft_collection, answer_offer, buy_nft, cancel_nft_sale, cancel_offer,
    claim_marketplace_fees, create_profile, instantiate, offer, receive_cw20, remove_expired_sales,
    sell_nft, send_message, transfer_my_nft, update_config, update_nft_sale, update_profile,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{CONFIG};

// Name & Version
const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_CRATE_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    init_msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    instantiate::instantiate_contract(deps, info, init_msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        // Admin entry points
        ExecuteMsg::RemoveExpiredSale {} => {
            if_admin(&config.contract_owner, info.sender.as_ref())?;
            remove_expired_sales::remove_expired_sales_function(deps, env, info)
        }
        ExecuteMsg::UpdateConfig { list_of_updates } => {
            if_admin(&config.contract_owner, info.sender.as_ref())?;
            update_config::execute_update_config(deps, env, info, list_of_updates)
        }
        ExecuteMsg::ClaimMarketplaceFees {} => {
            if_admin(&config.contract_owner, info.sender.as_ref())?;
            claim_marketplace_fees::execute_claim_marketplace_fees(deps)
        }
        // AddNewCollection is only used by admins for now and a PR on GitHub will be used to add a
        // collection on the marketplace
        ExecuteMsg::AddNewCollection {
            nft_collection_address,
            nft_contract_info: nft_contract_type,
        } => {
            if_admin(&config.contract_owner, info.sender.as_ref())?;
            add_new_nft_collection::execute_add_new_nft_collection(
                deps,
                env,
                info,
                nft_collection_address,
                nft_contract_type,
            )
        }
        // Any users entry points
        ExecuteMsg::TransferMyNft {
            nft_collection_address,
            token_id,
            recipient,
        } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            transfer_my_nft::execute_transfer_my_nft(
                deps,
                env,
                info,
                nft_collection_address,
                token_id,
                recipient,
            )
        }
        ExecuteMsg::SellNft { sale_info } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            sell_nft::execute_sell_nft(deps, env, info, sale_info)
        }
        ExecuteMsg::UpdateSale { sale_info } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            update_nft_sale::execute_update_nft_sale(deps, env, info, sale_info)
        }
        ExecuteMsg::CancelSale {
            nft_collection_address,
            token_id,
            additional_info,
        } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            cancel_nft_sale::execute_cancel_nft_sale(
                deps,
                env,
                info,
                nft_collection_address,
                token_id,
                additional_info,
            )
        }
        ExecuteMsg::BuyNft {
            nft_collection_address,
            token_id,
            additional_info,
        } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            buy_nft::execute_buy_nft(
                deps,
                env,
                info,
                nft_collection_address,
                token_id,
                additional_info,
            )
        }
        ExecuteMsg::Offer { offer } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            offer::execute_offer(deps, env, info, offer)
        }
        ExecuteMsg::CancelOffer {
            nft_collection_address,
            token_id,
            additional_info,
        } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            cancel_offer::execute_cancel_offer(
                deps,
                env,
                info,
                nft_collection_address,
                token_id,
                additional_info,
            )
        }
        ExecuteMsg::AnswerOffer {
            nft_collection_address,
            token_id,
            from,
            if_accepted,
            answer_msg,
        } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            answer_offer::execute_answer_offer(
                deps,
                env,
                info,
                nft_collection_address,
                token_id,
                from,
                if_accepted,
                answer_msg,
            )
        }
        ExecuteMsg::SendMessage { to, message } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            send_message::execute_send_message(deps, env, info, to, message)
        }
        ExecuteMsg::CreateMyProfile { additional_info } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            create_profile::execute_create_profile(deps, env, info, additional_info)
        }
        ExecuteMsg::UpdateMyProfile {
            profile,
            profile_update_action,
        } => {
            validate_address(
                env.contract.address.to_string(),
                deps.as_ref(),
                info.sender.to_string(),
            )?;
            if_enabled(config.contract_enabled)?;
            update_profile::execute_update_profile(deps, env, info, profile, profile_update_action)
        }
        ExecuteMsg::Receive(msg) => {
            if_enabled(config.contract_enabled)?;
            receive_cw20::execute_receive_cw20(deps, info, msg, env)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&crate::query::query_config(deps)?),
        QueryMsg::GetNftCollectionInfo {
            nft_collection_address,
        } => to_binary(&crate::query::query_nft_collection(
            deps,
            nft_collection_address,
        )?),
        QueryMsg::GetMarketplaceVolume {} => {
            to_binary(&crate::query::query_marketplace_total_volume(deps)?)
        }
        QueryMsg::GetNftCollectionVolume {
            nft_collection_address,
        } => to_binary(&crate::query::query_nft_collection_total_volume(
            deps,
            nft_collection_address,
        )?),
        QueryMsg::GetNftCollectionType {
            nft_collection_address,
        } => to_binary(&crate::query::query_nft_collection_type(
            deps,
            nft_collection_address,
        )?),
        QueryMsg::GetNftForSaleInfo {
            nft_collection_address,
            token_id,
        } => to_binary(&crate::query::query_nft_for_sale(
            deps,
            nft_collection_address,
            token_id,
        )?),
        QueryMsg::GetSellerAllNftsForSale {
            seller_address,
            start_after_collection_token_id,
            output_length,
        } => to_binary(&crate::query::query_nfts_for_sale_from_seller(
            deps,
            seller_address,
            start_after_collection_token_id,
            output_length,
        )?),
        QueryMsg::GetAllTokensByCollAndIfForSale {
            address,
            nft_collection_address,
            output_length,
        } => to_binary(&crate::query::query_all_tokens_by_coll_and_if_sale(
            deps,
            address,
            nft_collection_address,
            output_length,
        )?),
        QueryMsg::GetCollectionAllNftsForSale {
            nft_collection_address,
            start_after_token_id,
            output_length,
        } => to_binary(&crate::query::query_nfts_for_sale_from_collection(
            deps,
            nft_collection_address,
            start_after_token_id,
            output_length,
        )?),
        QueryMsg::GetMarketplaceInfo {} => to_binary(&crate::query::query_marketplace_info(deps)?),
        QueryMsg::GetTokenIdSaleHistory {
            nft_collection_address,
            token_id,
        } => to_binary(&crate::query::query_nft_trade_history(
            deps,
            nft_collection_address,
            token_id,
        )?),
        QueryMsg::GetProfileInfo {
            address_or_username,
        } => to_binary(&crate::query::query_profile_info(
            deps,
            address_or_username,
        )?),
        QueryMsg::GetAllOffersTokenId {
            token_id,
            nft_collection_address,
            start_after,
            output_length,
        } => to_binary(&crate::query::query_nft_offers_by_token_id(
            deps,
            nft_collection_address,
            token_id,
            start_after,
            output_length,
        )?),
        QueryMsg::GetAllOffersAddress {
            address,
            start_after,
            output_length,
        } => to_binary(&crate::query::query_nft_offers_from_offerer(
            deps,
            address,
            start_after,
            output_length,
        )?),
        QueryMsg::GetTokenIdsByCollection {
            address,
            list_of_collections,
        } => to_binary(&crate::query::query_token_ids_by_collection(
            deps,
            address,
            list_of_collections,
        )?),
    }
}
