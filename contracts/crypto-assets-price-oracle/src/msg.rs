use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128};
use general_utils::denominations::{AcceptedDenominations, Denomination};
use price_oracle_utils::oracle::{OraclePrices};

#[cw_serde]
pub struct InstantiateMsg {
    pub contract_owner: String,
    pub prices_feeder: String,
    pub max_history_length: u32,
    pub accepted_ibc_denoms: AcceptedDenominations
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { list_of_updates: Vec<UpdateConfigEnum> },
    FeedPrices { prices: OraclePrices },
}

#[cw_serde]
pub enum UpdateConfigEnum {
    ChangeMaxLength { length: u32 },
    UpdateOwner { new_owner: String },
    ChangePriceFeeder { new_feeder: String },
    AddDenoms { denoms: Vec<Denomination> },
    RemoveDenoms { denoms: Vec<Denomination> },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(price_oracle_utils::config::Config)]
    GetConfigAndCurrentPrices {},
    #[returns(cosmwasm_std::Uint128)]
    GetUsdcPriceFromAmountAndDenom { amount: Uint128, denom: Denomination },
    #[returns(Vec<(u64, price_oracle_utils::oracle::OraclePrices)>)]
    GetLatestHistoricalPrices { length: Option<u32> },
}