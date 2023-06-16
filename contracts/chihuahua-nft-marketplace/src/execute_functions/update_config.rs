use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use nft_marketplace_utils::marketplace_statistics::MarketplaceStatsByDenom;
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::msg::UpdateConfigEnum;
use crate::state::{CONFIG, MARKETPLACE_STATS_BY_DENOM, REWARD_SYSTEM};

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    list_of_updates: Vec<UpdateConfigEnum>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    for update in list_of_updates {
        match update {
            UpdateConfigEnum::EnableDisable {} => {
                config.contract_enabled = !config.contract_enabled;
            }
            UpdateConfigEnum::AddDenoms { denoms } => {
                config.accepted_ibc_denominations.add_many(denoms.clone());
                denoms.iter().try_for_each(|denom| {
                    MARKETPLACE_STATS_BY_DENOM.save(
                        deps.storage,
                        denom,
                        &MarketplaceStatsByDenom::new(denom.clone()),
                    )
                })?;
            }
            UpdateConfigEnum::RemoveDenoms { denoms } => {
                config.accepted_ibc_denominations.remove_many(denoms);
            }
            UpdateConfigEnum::UpdateOwner { address } => {
                config.contract_owner = deps.api.addr_validate(&address.to_string())?.to_string();
            }
            UpdateConfigEnum::UpdateRewardSystem { reward_system } => {
                REWARD_SYSTEM.save(deps.storage, &reward_system)?;
            }
            UpdateConfigEnum::UpdateAcceptedNftContracts { contracts } => {
                for nft_contract_info in contracts {
                    let contains_contract_info = config
                        .accepted_nft_code_ids
                        .iter()
                        .any(|contract_info| contract_info.equal(&nft_contract_info));
                    if contains_contract_info {
                        config
                            .accepted_nft_code_ids
                            .retain(|contract_info| !contract_info.equal(&nft_contract_info));
                    } else {
                        config.accepted_nft_code_ids.push(nft_contract_info.clone());
                    }
                }
            }
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(ResponseHandler::update_config().response)
}
