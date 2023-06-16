use std::str::FromStr;

use cosmwasm_std::{
    to_binary, Decimal, DepsMut, MessageInfo, QueryRequest, Response, Uint128, WasmQuery,
};

use general_utils::error::ContractError;
use nft_marketplace_utils::config::Config;
use nft_marketplace_utils::marketplace_statistics::{GeneralStats, MarketplaceStatsByDenom};
use nft_marketplace_utils::nft_collection::{
    NftCollectionInfoAndUsdcVol, NftContractInfo, NftContractType,
};
use nft_marketplace_utils::response_handler::ResponseHandler;
use nft_marketplace_utils::reward_system::RewardSystem;

use crate::constants::MARKETPLACE_USDC_INDICATOR;
use crate::msg::InstantiateMsg;
use crate::state::{
    CONFIG, GENERAL_STATS, LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL, MARKETPLACE_STATS_BY_DENOM,
    REWARD_SYSTEM,
};

pub fn instantiate_contract(
    deps: DepsMut,
    _info: MessageInfo,
    init_msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Check if the given price oracle accepts the listing fee denom
    deps.querier
        .query::<Uint128>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: init_msg.oracle_contract_address.clone(),
            msg: to_binary(&oracle::msg::QueryMsg::GetUsdcPriceFromAmountAndDenom {
                amount: Uint128::new(100_000_000_000u128),
                denom: init_msg.marketplace_listing_fee_denom.clone(),
            })?,
        }))?;

    // Check if the given reward address is a contract
    deps.querier
        .query_wasm_contract_info(init_msg.reward_system.reward_token_address.clone())?;

    CONFIG.save(
        deps.storage,
        &Config::new_checked(
            deps.api,
            false,
            init_msg.contract_owner,
            init_msg.accepted_ibc_denominations.clone(),
            Decimal::from_str(&init_msg.marketplace_pct_fees_decimal_string).unwrap(),
            init_msg.marketplace_listing_fee_value,
            init_msg.marketplace_listing_fee_denom,
            init_msg.oracle_contract_address,
            init_msg.accepted_nft_code_ids,
        )?,
    )?;

    REWARD_SYSTEM.save(
        deps.storage,
        &RewardSystem::new_checked(
            deps.api,
            init_msg.reward_system.reward_token_address,
            init_msg.reward_system.reward_token_per_1usdc_volume,
            init_msg.reward_system.total_reward_tokens_distributed,
            init_msg.reward_system.vip_perks,
        )?,
    )?;

    // Contract states update: Marketplace statistics are for each accepted denomination
    init_msg
        .accepted_ibc_denominations
        .list_of_denoms
        .iter()
        .try_for_each(|accepted_denom| {
            MARKETPLACE_STATS_BY_DENOM.save(
                deps.storage,
                accepted_denom,
                &MarketplaceStatsByDenom::new(accepted_denom.clone()),
            )
        })?;

    // For the whole marketplace
    LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.save(
        deps.storage,
        MARKETPLACE_USDC_INDICATOR,
        &NftCollectionInfoAndUsdcVol::new(
            MARKETPLACE_USDC_INDICATOR.to_string(),
            NftContractInfo {
                code_id: 0,
                nft_contract_type: NftContractType::MarketplaceInfo,
            },
        ),
    )?;

    GENERAL_STATS.save(deps.storage, &GeneralStats::new())?;

    Ok(ResponseHandler::init_response().response)
}
