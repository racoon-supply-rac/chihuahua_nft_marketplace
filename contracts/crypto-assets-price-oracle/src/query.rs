use cosmwasm_std::{Deps, Order, StdError, StdResult, Uint128};
use general_utils::denominations::Denomination;
use price_oracle_utils::config::Config;
use price_oracle_utils::oracle::OraclePrices;
use crate::state::{CONFIG, HISTORICAL_PRICES};

pub fn query_config_and_prices(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_latest_value_usdc_from_amount_denom(
    deps: Deps,
    requested_amount: Uint128,
    requested_denom: Denomination,
) -> StdResult<Uint128> {
    let config_and_prices = CONFIG.load(deps.storage)?;
    let result = config_and_prices.current_prices.prices
        .iter()
        .find(|p| p.ibc_denom == requested_denom)
        .map(|denom| {
            requested_amount.checked_mul(denom.value_usdc_6_decimals)
                .and_then(|value| Ok(value.checked_div(Uint128::new(1_000_000u128)).unwrap()))
        })
        .ok_or_else(|| StdError::generic_err("DenominationNotFound"))?
        .unwrap();
    Ok(result)
}

pub fn query_historical_prices(
    deps: Deps,
    length: Option<u32>,
) -> StdResult<Vec<(u64, OraclePrices)>> {
    let config = CONFIG.load(deps.storage)?;
    let max_history_length = config.max_history_length;
    let historical_prices: Vec<(u64, OraclePrices)> = HISTORICAL_PRICES
        .range(deps.storage, None, None, Order::Descending)
        .take(length.unwrap_or(max_history_length) as usize)
        .map(|res| res.map_err(|_| StdError::generic_err("Cannot get historical price")))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(historical_prices)
}
