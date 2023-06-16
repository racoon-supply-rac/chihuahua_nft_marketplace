use crate::error::ContractError;
use crate::error::GenericError::{ContractDisabled, Unauthorized};
use cosmwasm_std::{ensure, Deps};

pub fn if_admin(admin: &str, sender: &str) -> Result<(), ContractError> {
    ensure!(sender == admin, ContractError::Generic(Unauthorized {}));
    Ok(())
}

pub fn if_enabled(contract_enabled: bool) -> Result<(), ContractError> {
    ensure!(
        contract_enabled,
        ContractError::Generic(ContractDisabled {})
    );
    Ok(())
}

pub fn validate_address(
    contract_address: String,
    deps: Deps,
    sender: String,
) -> Result<(), ContractError> {
    if sender != contract_address && deps.querier.query_wasm_contract_info(sender).is_ok() {
        return Err(ContractError::Generic(Unauthorized {}));
    }
    Ok(())
}