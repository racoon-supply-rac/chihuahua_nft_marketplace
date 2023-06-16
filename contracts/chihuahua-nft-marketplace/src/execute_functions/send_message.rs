use cosmwasm_std::{ensure, Addr, DepsMut, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{InvalidMessage, ReceiverDoesNotExist};
use nft_marketplace_utils::profile::Profile;
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::state::{PROFILES, USERNAMES};

pub fn execute_send_message(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    to: String,
    message: String,
) -> Result<Response, ContractError> {
    ensure!(
        message.is_ascii() && message.len() <= 120,
        ContractError::NftMarketplaceError(InvalidMessage {})
    );

    let sender_username: Option<String> = if !PROFILES.has(deps.storage, info.sender.as_ref()) {
        let profile = Profile::new(info.sender.to_string());
        PROFILES.save(deps.storage, info.sender.as_ref(), &profile)?;
        None
    } else {
        let profile = PROFILES.load(deps.storage, info.sender.as_ref())?;
        profile.username
    };

    // To can be a username or an address
    let mut receiver_username = Some("Does not exist".to_string());
    let mut validated_receiver = deps.api.addr_validate(&to);
    if validated_receiver.is_err() {
        // Error could mean its a username - need to check -> if nothing -> profile does not exist
        if !USERNAMES.has(deps.storage, &to) {
            return Err(ContractError::NftMarketplaceError(ReceiverDoesNotExist {}));
        } else {
            validated_receiver = Ok(Addr::unchecked(USERNAMES.load(deps.storage, &to)?));
            receiver_username = Some(to);
        }
    }
    // If reached here, receiver exists -> need to check if the sender exists and create it
    let receiver = validated_receiver?.to_string();
    let receiver_profile = PROFILES.load(deps.storage, &receiver)?;
    let updated_receiver_profile =
        receiver_profile.receive_message(info.sender.to_string(), sender_username.clone(), message);
    PROFILES.save(deps.storage, &receiver, &updated_receiver_profile)?;

    Ok(ResponseHandler::send_message(
        info.sender.to_string(),
        sender_username,
        receiver,
        receiver_username,
    )
    .response)
}
