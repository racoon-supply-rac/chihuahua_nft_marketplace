use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

pub type IbcDenom = String;

#[cw_serde]
pub struct OraclePrice {
    pub ticker: String,
    pub name: String,
    pub ibc_denom: IbcDenom,
    pub value_usdc_6_decimals: Uint128,
}

#[cw_serde]
pub struct OraclePrices {
    pub prices: Vec<OraclePrice>,
    pub at_time: Timestamp,
}
