use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;

use general_utils::error::ContractError;
use general_utils::error::GenericError::InvalidDenominationReceived;
use general_utils::validations::validate_address;

use crate::execute_functions::level_up_profile;
use crate::msg;
use crate::state::REWARD_SYSTEM;

pub fn execute_receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
    env: Env,
) -> Result<Response, ContractError> {
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    let cw20_msg: msg::ReceiveMsg = from_binary(&wrapper.msg)?;
    let cw20_denom = deps.api.addr_validate(info.sender.as_ref())?;
    if cw20_denom != reward_system.reward_token_address {
        return Err(ContractError::Generic(InvalidDenominationReceived {}));
    }
    let user_sender = wrapper.sender.clone();
    let cw20_amount = wrapper.amount;
    validate_address(
        env.contract.address.to_string(),
        deps.as_ref(),
        user_sender.to_string(),
    )?;
    match cw20_msg {
        msg::ReceiveMsg::LevelUpProfile {} => {
            level_up_profile::execute_level_up_profile(deps, env, info, user_sender, cw20_amount)
        }
    }
}
