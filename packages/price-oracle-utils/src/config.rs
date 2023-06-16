use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;

use general_utils::denominations::{AcceptedDenominations, Denomination};

use crate::oracle::OraclePrices;

#[cw_serde]
pub struct Config {
    pub contract_owner: String,
    pub prices_feeder: String,
    pub current_prices: OraclePrices,
    pub accepted_ibc_denoms: AcceptedDenominations,
    pub max_history_length: u32,
    pub next_history_id: u64,
    pub oldest_history_id: u64,
}

impl Config {
    pub fn new(
        contract_owner: String,
        prices_feeder: String,
        accepted_ibc_denoms: Vec<Denomination>,
        max_history_length: u32,
    ) -> Self {
        Config {
            contract_owner,
            prices_feeder,
            current_prices: OraclePrices {
                prices: vec![],
                at_time: Timestamp::from_seconds(0u64),
            },
            accepted_ibc_denoms: AcceptedDenominations::new(accepted_ibc_denoms),
            max_history_length,
            next_history_id: 1,
            oldest_history_id: 0,
        }
    }
}
