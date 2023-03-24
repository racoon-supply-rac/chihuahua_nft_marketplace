use std::str::FromStr;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Api, Decimal, Uint128};
use general_utils::denominations::{AcceptedDenominations, Denomination};
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{InvalidAcceptedDenoms, InvalidMarketplaceFee};
use crate::marketplace_statistics::GeneralStats;
use crate::reward_system::RewardSystem;

#[cw_serde]
pub struct Config {
    pub contract_enabled: bool,
    pub contract_owner: String,
    pub accepted_ibc_denominations: AcceptedDenominations,
    pub marketplace_pct_fees: Decimal,
    pub marketplace_listing_fee_value: Uint128,
    pub marketplace_listing_fee_denom: Denomination,
    pub oracle_contract_address: String
}

#[cw_serde]
pub struct ConfigRewardGenStatsMsg {
    pub contract_enabled: bool,
    pub contract_owner: String,
    pub accepted_ibc_denominations: AcceptedDenominations,
    pub marketplace_pct_fees: Decimal,
    pub marketplace_listing_fee_value: Uint128,
    pub marketplace_listing_fee_denom: Denomination,
    pub oracle_contract_address: String,
    pub reward_system: RewardSystem,
    pub general_stats: GeneralStats
}

impl Config {
    pub fn new_checked(
        api: &dyn Api,
        contract_enabled: bool,
        contract_owner: String,
        accepted_ibc_denominations: AcceptedDenominations,
        marketplace_pct_fees: Decimal,
        marketplace_listing_fee_value: Uint128,
        marketplace_listing_fee_denom: Denomination,
        oracle_contract_address: String
    ) -> Result<Self, ContractError> {
        if accepted_ibc_denominations.list_of_denoms.len() < 1 {
            return Err(ContractError::NftMarketplace(InvalidAcceptedDenoms {}))
        }
        if !accepted_ibc_denominations.list_of_denoms.contains(&marketplace_listing_fee_denom) {
            return Err(ContractError::NftMarketplace(InvalidAcceptedDenoms {}))
        }
        if marketplace_listing_fee_value < Uint128::zero() {
            return Err(ContractError::NftMarketplace(InvalidMarketplaceFee {}))
        }
        Ok(Config {
            contract_enabled,
            contract_owner: api.addr_validate(&contract_owner)?.to_string(),
            accepted_ibc_denominations,
            marketplace_pct_fees: Config::checked_marketplace_fees(marketplace_pct_fees)?,
            marketplace_listing_fee_value,
            marketplace_listing_fee_denom,
            oracle_contract_address: api.addr_validate(&oracle_contract_address)?.to_string(),
        })
    }
    pub fn checked_marketplace_fees(marketplace_pct_fees: Decimal) -> Result<Decimal, ContractError> {
        if marketplace_pct_fees >= Decimal::from_str("0.1").unwrap() || marketplace_pct_fees <= Decimal::from_str("0.01").unwrap() {
            return Err(ContractError::NftMarketplace(InvalidMarketplaceFee {}))
        } else {
            Ok(marketplace_pct_fees)
        }
    }
}