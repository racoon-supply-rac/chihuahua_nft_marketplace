use cosmwasm_std::{ensure, DepsMut, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    AdditionalInfoNeedsToBeFilled, CantUseAdditionalInfoIfNotContract, YourProfileAlreadyExists,
};
use nft_marketplace_utils::profile::Profile;
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::state::PROFILES;

pub fn execute_create_profile(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    additional_info: Option<String>,
) -> Result<Response, ContractError> {
    // Validate: who is the sender
    if let Some(addr) = additional_info {
        ensure!(
            info.sender == env.contract.address,
            ContractError::NftMarketplaceError(CantUseAdditionalInfoIfNotContract {})
        );
        info.sender = deps.api.addr_validate(&addr)?;
    } else {
        ensure!(
            info.sender != env.contract.address,
            ContractError::NftMarketplaceError(AdditionalInfoNeedsToBeFilled {})
        );
    }

    let sender_addr = deps.api.addr_validate(info.sender.as_ref())?.to_string();

    // Validate: if profile already exists
    ensure!(
        PROFILES.may_load(deps.storage, &sender_addr)?.is_none(),
        ContractError::NftMarketplaceError(YourProfileAlreadyExists {})
    );
    let new_profile = Profile::new(info.sender.to_string());
    PROFILES.save(deps.storage, &sender_addr, &new_profile)?;

    Ok(ResponseHandler::create_or_update_profile(new_profile)?.response)
}
