use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use general_utils::error::ContractError;
use general_utils::error::GenericError::InvalidDenominationReceived;

#[cw_serde]
pub struct Buyer {
    pub sender: String,
    pub amount: Uint128,
    pub denom: String
}

impl Buyer {
    pub fn new_checked(
        sender: Addr,
        funds: Vec<Coin>
    ) -> Result<Self, ContractError> {
        if funds.len() != 1 {
            return Err(ContractError::Generic(InvalidDenominationReceived {}))
        }
        Ok(Buyer {
            sender: sender.to_string(),
            amount: funds[0].clone().amount,
            denom: funds[0].clone().denom
        })
    }
}
