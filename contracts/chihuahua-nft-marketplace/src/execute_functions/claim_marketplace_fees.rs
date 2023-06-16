use cosmwasm_std::{DepsMut, Response, Uint128};

use general_utils::denominations::DenominationValue;
use general_utils::error::ContractError;
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::state::{CONFIG, MARKETPLACE_STATS_BY_DENOM};

pub fn execute_claim_marketplace_fees(deps: DepsMut) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // States Update: Gather the fees into a vec and reset to 0
    let all_accepted_denoms_value: Vec<DenominationValue> = config
        .accepted_ibc_denominations
        .list_of_denoms
        .iter()
        .map(|denom| {
            let mut mp_fees: Uint128 = Uint128::zero();
            MARKETPLACE_STATS_BY_DENOM.update(
                deps.storage,
                denom,
                |mp_info| -> Result<_, ContractError> {
                    let mut mp_info_u = mp_info.unwrap();
                    mp_fees = mp_info_u.marketplace_fees_to_claim;
                    mp_info_u.marketplace_fees_to_claim = Uint128::zero();
                    Ok(mp_info_u)
                },
            )?;
            Ok::<DenominationValue, ContractError>(DenominationValue {
                denom: denom.clone(),
                value: mp_fees,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Remove the Zero fees before sending the message
    let non_zero_denoms: Vec<DenominationValue> = all_accepted_denoms_value
        .into_iter()
        .filter(|dv| dv.value != Uint128::zero())
        .collect();

    Ok(ResponseHandler::claim_marketplace_fees(config.contract_owner, non_zero_denoms).response)
}
