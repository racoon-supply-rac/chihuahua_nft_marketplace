use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use price_oracle_utils::config::Config;
use price_oracle_utils::oracle::OraclePrices;
use price_oracle_utils::response_handler::ResponseHandler;
use general_utils::error::ContractError;
use general_utils::error::GenericError::InvalidDenominationReceived;
use general_utils::error::PriceOracleError::{InvalidTimeForPrice, SomeDenomsAreMissingInYourUpdate};
use crate::msg::{InstantiateMsg, UpdateConfigEnum};
use crate::state::{CONFIG, HISTORICAL_PRICES};

pub fn instantiate_contract(
    deps: DepsMut,
    _info: MessageInfo,
    mut init_msg: InstantiateMsg
) -> Result<Response, ContractError> {

    init_msg.contract_owner = deps.api.addr_validate(&init_msg.contract_owner)?.to_string();
    init_msg.prices_feeder = deps.api.addr_validate(&init_msg.prices_feeder)?.to_string();

    CONFIG.save(
        deps.storage,
        &Config::new(
            init_msg.contract_owner,
            init_msg.prices_feeder,
            init_msg.accepted_ibc_denoms.list_of_denoms,
            init_msg.max_history_length),
    )?;

    Ok(ResponseHandler::init_response().response)
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    list_of_updates: Vec<UpdateConfigEnum>
) -> Result<Response, ContractError> {

    let mut config = CONFIG.load(deps.storage)?;
    for update in list_of_updates {
        match update {
            UpdateConfigEnum::UpdateOwner { new_owner } => {
                config.contract_owner = deps.api.addr_validate(&*new_owner)?.to_string();
            }
            UpdateConfigEnum::ChangePriceFeeder { new_feeder } => {
                config.prices_feeder =  deps.api.addr_validate(&*new_feeder)?.to_string();
            }
            UpdateConfigEnum::ChangeMaxLength { length } => {
                config.max_history_length = length;
            },
            UpdateConfigEnum::AddDenoms { denoms } => {
                config.accepted_ibc_denoms.add_many(denoms);
            }
            UpdateConfigEnum::RemoveDenoms { denoms } => {
                config.accepted_ibc_denoms.remove_many(denoms);
            },
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(ResponseHandler::update_config().response)
}

pub fn execute_add_new_prices_info(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    prices: OraclePrices,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Validate: Check if all received prices are accepted denoms + if all denoms are included
    for price in &prices.prices {
        if !config.accepted_ibc_denoms.list_of_denoms.contains(&price.ibc_denom) {
            return Err(ContractError::Generic(InvalidDenominationReceived {}));
        }
    }
    if config.accepted_ibc_denoms.list_of_denoms.len() != prices.prices.len() {
        return Err(ContractError::PriceOracle(SomeDenomsAreMissingInYourUpdate {}));
    }

    // Validate: Check if new prices are received in chronological order
    if prices.at_time <= config.current_prices.at_time {
        return Err(ContractError::PriceOracle(InvalidTimeForPrice {}));
    }

    // Save new historical prices and remove old ones
    let history_id = config.next_history_id;
    let mut oldest_history_id = config.oldest_history_id;
    let max_history_length = config.max_history_length;
    HISTORICAL_PRICES.save(deps.storage, history_id, &prices)?;
    while history_id - oldest_history_id > max_history_length as u64 {
        HISTORICAL_PRICES.remove(deps.storage, oldest_history_id);
        oldest_history_id += 1;
    }

    // Update the config state
    config.next_history_id += 1;
    config.oldest_history_id = oldest_history_id;
    config.current_prices = prices.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(ResponseHandler::log_add_new_oracle_prices(&prices.clone().at_time).response)
}