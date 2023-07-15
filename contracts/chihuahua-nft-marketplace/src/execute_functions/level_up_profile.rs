use cosmwasm_std::{DepsMut, ensure, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use general_utils::error::GenericError::{InvalidDenominationReceived, InvalidFundsReceived};
use general_utils::validations::validate_address;
use nft_marketplace_utils::profile::Profile;
use nft_marketplace_utils::response_handler::ResponseHandler;
use nft_marketplace_utils::reward_system::{VipLevel};

use crate::state::{PROFILES, REWARD_SYSTEM};

pub fn execute_level_up_profile(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;

    ensure!(info.funds.len() == 1, ContractError::Generic(InvalidFundsReceived {}));
    ensure!(info.funds[0].denom == reward_system.reward_token_address, ContractError::Generic(InvalidDenominationReceived {}));
    validate_address(
        env.contract.address.to_string(),
        deps.as_ref(),
        info.sender.to_string(),
    )?;

    let mut loaded_profile: Profile = PROFILES.load(deps.storage, info.sender.as_str())?;
    let previous_level = loaded_profile.vip_level.clone();
    if VipLevel::level_up_if_possible(
        loaded_profile.vip_level.clone().unwrap(),
        reward_system.vip_perks,
        info.funds[0].amount,
    )? {
        loaded_profile = loaded_profile.level_up();
        PROFILES.save(deps.storage, info.sender.as_str(), &loaded_profile)?;
    }
    Ok(ResponseHandler::level_up_profile(
        previous_level.unwrap(),
        loaded_profile.vip_level.unwrap(),
    )?
    .response)
}
