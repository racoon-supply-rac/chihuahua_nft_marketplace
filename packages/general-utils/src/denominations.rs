use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

use crate::error::ContractError;
use crate::error::GenericError::InvalidDenominationReceived;

pub type Denomination = String;

#[cw_serde]
pub struct DenominationValue {
    pub denom: Denomination,
    pub value: Uint128,
}

#[cw_serde]
pub struct AcceptedDenominations {
    pub list_of_denoms: Vec<Denomination>,
}

impl AcceptedDenominations {
    pub fn new(list_of_denoms: Vec<Denomination>) -> Self {
        AcceptedDenominations { list_of_denoms }
    }
    pub fn add_many(&mut self, denoms: Vec<Denomination>) -> &mut Self {
        for denom in &denoms {
            if !self.list_of_denoms.contains(&Denomination::from(denom)) {
                self.add(denom.to_string());
            }
        }

        self
    }
    pub fn remove_many(&mut self, denoms: Vec<Denomination>) -> &mut Self {
        for denom in denoms {
            self.remove(denom)
        }
        self
    }
    pub fn add(&mut self, denom: Denomination) {
        self.list_of_denoms.push(denom);
    }

    pub fn remove(&mut self, denom: Denomination) {
        if let Some(remove_location) = self.list_of_denoms.iter().position(|x| *x == denom) {
            self.list_of_denoms.remove(remove_location);
        }
    }

    pub fn check_if_denom_is_accepted(self, denom: &Denomination) -> Result<bool, ContractError> {
        if !self.list_of_denoms.contains(denom) {
            return Err(ContractError::Generic(InvalidDenominationReceived {}));
        }
        Ok(true)
    }
}
