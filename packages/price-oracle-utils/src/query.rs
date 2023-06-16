use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Uint128;
use general_utils::denominations::Denomination;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::config::Config)]
    GetConfigAndCurrentPrices {},
    #[returns(cosmwasm_std::Uint128)]
    GetUsdcPriceFromAmountAndDenom {
        amount: Uint128,
        denom: Denomination,
    },
    #[returns(Vec<(u64, crate::oracle::OraclePrices)>)]
    GetLatestHistoricalPrices { length: Option<u32> },
}
