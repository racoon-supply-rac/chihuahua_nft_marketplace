use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use general_utils::error::ContractError;
use nft_marketplace_utils::profile::Profile;
use nft_marketplace_utils::response_handler::ResponseHandler;
use nft_marketplace_utils::reward_system::{RewardSystem, VipLevel};

use crate::state::{PROFILES, REWARD_SYSTEM};

pub fn execute_level_up_profile(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    mut user_sender: String,
    cw20_amount: Uint128,
) -> Result<Response, ContractError> {
    user_sender = deps.api.addr_validate(&user_sender)?.to_string();
    let reward_system: RewardSystem = REWARD_SYSTEM.load(deps.storage)?;
    let mut loaded_profile: Profile = PROFILES.load(deps.storage, user_sender.as_ref())?;
    let previous_level = loaded_profile.vip_level.clone();
    if VipLevel::level_up_if_possible(
        loaded_profile.vip_level.clone().unwrap(),
        reward_system.vip_perks,
        cw20_amount,
    )? {
        loaded_profile = loaded_profile.level_up();
        PROFILES.save(deps.storage, user_sender.as_ref(), &loaded_profile)?;
    }
    Ok(ResponseHandler::level_up_profile(
        previous_level.unwrap(),
        loaded_profile.vip_level.unwrap(),
    )?
    .response)
}
