#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ensure, to_binary, Binary, StdResult};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw2;

use general_utils::error::ContractError;
use general_utils::error::GenericError::Unauthorized;

use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::CONFIG;

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
        ExecuteMsg::UpdateConfig { list_of_updates } => {
            ensure!(
                info.sender.as_ref() == config.contract_owner,
                ContractError::Generic(Unauthorized {})
            );
            execute::execute_update_config(deps, env, info, list_of_updates)
        }

        ExecuteMsg::FeedPrices { prices } => {
            ensure!(
                info.sender.as_ref() == config.prices_feeder,
                ContractError::Generic(Unauthorized {})
            );
            execute::execute_add_new_prices_info(deps, env, info, prices)
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
        QueryMsg::GetConfigAndCurrentPrices {} => {
            to_binary(&crate::query::query_config_and_prices(deps)?)
        }

        QueryMsg::GetUsdcPriceFromAmountAndDenom { amount, denom } => to_binary(
            &crate::query::query_latest_value_usdc_from_amount_denom(deps, amount, denom)?,
        ),

        QueryMsg::GetLatestHistoricalPrices { length } => {
            to_binary(&crate::query::query_historical_prices(deps, length)?)
        }
    }
}
