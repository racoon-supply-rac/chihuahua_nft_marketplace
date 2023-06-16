use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::UsernameUnexpectedError;
use nft_marketplace_utils::profile::{Profile, ProfileUpdateAction};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::state::{PROFILES, REWARD_SYSTEM, USERNAMES};

pub fn execute_update_profile(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_profile: Profile,
    profile_update_action: ProfileUpdateAction,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.to_string();
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    let current_profile = PROFILES.load(deps.storage, &sender_addr)?;

    if new_profile.username.is_some() && profile_update_action == ProfileUpdateAction::Add {
        // Means the user wants to change or add a username
        if USERNAMES.has(deps.storage, &new_profile.username.clone().unwrap()) {
            // Username is already taken -> cannot be changed so set to None
            return Err(ContractError::NftMarketplaceError(UsernameUnexpectedError {}));
        }
        // Otherwise -> continue and can be changed below if it respects the username constraints
    }

    let updated_profile = current_profile.clone().user_update_profile(
        deps.as_ref(),
        new_profile,
        reward_system,
        profile_update_action,
    )?;

    // Now need to check if the profile already had a username or not. If yes, need to change it and remove the old one
    if updated_profile.username != current_profile.username {
        // Already checked if the current profile had a username and if the new username was taken
        if updated_profile.username.is_some() {
            if current_profile.username.is_some() {
                USERNAMES.remove(deps.storage, &current_profile.username.unwrap());
            }
            USERNAMES.save(
                deps.storage,
                &updated_profile.username.clone().unwrap(),
                &sender_addr,
            )?;
        }
    }
    PROFILES.save(deps.storage, &sender_addr, &updated_profile)?;
    Ok(ResponseHandler::create_or_update_profile(updated_profile)?.response)
}
