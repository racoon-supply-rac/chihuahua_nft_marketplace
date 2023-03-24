#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Deps};
use cosmwasm_std::{StdResult, Binary, to_binary};
use general_utils::validations::{if_admin, if_enabled};

use crate::msg::{InstantiateMsg, ExecuteMsg, MigrateMsg, QueryMsg};
use crate::state::{CONFIG};
use general_utils::error::{ContractError};
use general_utils::error::GenericError::NotImplementedYet;
use crate::execute;

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
    execute::instantiate_contract(deps, info, init_msg)
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
        ExecuteMsg::UpdateConfig { list_of_updates } => {
            if_admin(&config.contract_owner, &info.sender.to_string())?;
            execute::execute_update_config(deps, env, info, list_of_updates)
        },
        ExecuteMsg::ClaimMarketplaceFees {} => {
            if_admin(&config.contract_owner, &info.sender.to_string())?;
            execute::execute_claim_marketplace_fees(deps)
        },
        // AddNewCollection is only used by admins for now and a PR on GitHub will be used to add a
        // collection on the marketplace
        ExecuteMsg::AddNewCollection { nft_collection_address } => {
            if_admin(&config.contract_owner, &info.sender.to_string())?;
            execute::execute_add_new_nft_collection(deps, env, info, nft_collection_address)
        },
        // Any users entry points
        ExecuteMsg::SellNft { sale_info } => {
            if_enabled(config.contract_enabled)?;
            execute::execute_sell_nft(deps, env, info, sale_info)
        },
        ExecuteMsg::UpdateSale { sale_info } => {
            if_enabled(config.contract_enabled)?;
            execute::execute_update_nft_sale(deps, env, info, sale_info)
        },
        ExecuteMsg::CancelSale { nft_collection_address, token_id, additional_info } => {
            if_enabled(config.contract_enabled)?;
            execute::execute_cancel_nft_sale(deps, env, info, nft_collection_address, token_id, additional_info)
        },
        ExecuteMsg::BuyNft { nft_collection_address, token_id, additional_info } => {
            if_enabled(config.contract_enabled)?;
            execute::execute_buy_nft(deps, env, info, nft_collection_address, token_id, additional_info)
        },
        ExecuteMsg::Offer { offer: _ } => {
            return Err(ContractError::Generic(NotImplementedYet {}))
        },
        ExecuteMsg::CancelOffer { nft_collection_address: _, token_id: _, additional_info: _ } => {
            return Err(ContractError::Generic(NotImplementedYet {}))
        },
        ExecuteMsg::AcceptOffer { nft_collection_address: _, token_id: _, from: _ } => {
            return Err(ContractError::Generic(NotImplementedYet {}))
        },
        ExecuteMsg::CreateMyProfile { profile, additional_info } => {
            if_enabled(config.contract_enabled)?;
            execute::execute_create_profile(deps, env, info, profile, additional_info)
        },
        ExecuteMsg::UpdateMyProfile { profile } => {
            if_enabled(config.contract_enabled)?;
            execute::execute_update_profile(deps, env, info, profile)
        }
        ExecuteMsg::Receive(msg) => {
            if_enabled(config.contract_enabled)?;
            execute::execute_receive_cw20(deps, info, msg, env)
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
        QueryMsg::GetConfig {} => {
            to_binary(&crate::query::query_config(deps)?)
        },
        QueryMsg::GetNftCollectionInfo { nft_collection_address } => {
            to_binary(&crate::query::query_nft_collection(deps, nft_collection_address)?)
        },
        QueryMsg::GetMarketplaceVolume {} => {
            to_binary(&crate::query::query_marketplace_total_volume(deps)?)
        },
        QueryMsg::GetNftCollectionVolume { nft_collection_address } => {
            to_binary(&crate::query::query_nft_collection_total_volume(deps, nft_collection_address)?)
        },
        QueryMsg::GetNftForSaleInfo { nft_collection_address, token_id } => {
            to_binary(&crate::query::query_nft_for_sale(deps, nft_collection_address, token_id)?)
        },
        QueryMsg::GetSellerAllNftsForSale { seller_address, start_after_collection_token_id: start_after_token_id, output_length } => {
            to_binary(&crate::query::query_nfts_for_sale_from_seller(
                deps,
                seller_address,
                start_after_token_id,
                output_length)?)
        }
        QueryMsg::GetCollectionAllNftsForSale { nft_collection_address, start_after_token_id, output_length } =>
            {
                to_binary(&crate::query::query_nfts_for_sale_from_collection(
                    deps,
                    nft_collection_address,
                    start_after_token_id,
                    output_length)?)
            },
        QueryMsg::GetMarketplaceInfo {} => {
            to_binary(&crate::query::query_marketplace_info(deps)?)
        },
        QueryMsg::GetTokenIdSaleHistory { nft_collection_address, token_id } => {
            to_binary(&crate::query::query_nft_trade_history(deps, nft_collection_address, token_id)?)
        },
        QueryMsg::GetProfileInfo { address } => {
            to_binary(&crate::query::query_profile_info(deps, address)?)
        },
        QueryMsg::GetAllOffersTokenId { token_id, nft_collection_address, start_after, output_length } => {
            to_binary(&crate::query::query_nft_offers_by_token_id(deps, nft_collection_address, token_id, start_after, output_length)?)
        },
        QueryMsg::GetAllOffersAddress { address, start_after, output_length } => {
            to_binary(&crate::query::query_nft_offers_from_offerer(deps, address, start_after, output_length)?)
        },
        QueryMsg::GetTokenIdsByCollection { address, list_of_collections } => {
            to_binary(&crate::query::query_token_ids_by_collection(deps, address, list_of_collections)?)
        },
    }
}
