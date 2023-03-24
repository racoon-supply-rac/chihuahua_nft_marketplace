use crate::error::ContractError;
use crate::error::GenericError::{ContractDisabled, Unauthorized};

pub fn if_admin(admin: &str, sender: &str) -> Result<(), ContractError> {
    if sender == admin {
        Ok(())
    } else {
        Err(ContractError::Generic(Unauthorized {}))
    }
}

pub fn if_enabled(contract_enabled: bool) -> Result<bool, ContractError> {
    if contract_enabled {
        Ok(true)
    } else {
        Err(ContractError::Generic(ContractDisabled {}))
    }
}