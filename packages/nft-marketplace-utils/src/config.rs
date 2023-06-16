use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Api, Decimal, Uint128};

use general_utils::denominations::{AcceptedDenominations, Denomination};
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{InvalidAcceptedDenoms, InvalidMarketplaceFee};

use crate::marketplace_statistics::GeneralStats;
use crate::nft_collection::NftContractInfo;
use crate::reward_system::RewardSystem;

#[cw_serde]
pub struct Config {
    pub contract_enabled: bool,
    pub contract_owner: String,
    pub accepted_ibc_denominations: AcceptedDenominations,
    pub marketplace_pct_fees: Decimal,
    pub marketplace_listing_fee_value: Uint128,
    pub marketplace_listing_fee_denom: Denomination,
    pub oracle_contract_address: String,
    pub accepted_nft_code_ids: Vec<NftContractInfo>,
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
    pub general_stats: GeneralStats,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new_checked(
        api: &dyn Api,
        contract_enabled: bool,
        contract_owner: String,
        accepted_ibc_denominations: AcceptedDenominations,
        marketplace_pct_fees: Decimal,
        marketplace_listing_fee_value: Uint128,
        marketplace_listing_fee_denom: Denomination,
        oracle_contract_address: String,
        accepted_nft_code_ids: Vec<NftContractInfo>,
    ) -> Result<Self, ContractError> {
        ensure!(
            !accepted_ibc_denominations.list_of_denoms.is_empty(),
            ContractError::NftMarketplaceError(InvalidAcceptedDenoms {})
        );
        ensure!(
            accepted_ibc_denominations
                .list_of_denoms
                .contains(&marketplace_listing_fee_denom),
            ContractError::NftMarketplaceError(InvalidAcceptedDenoms {})
        );
        ensure!(
            marketplace_listing_fee_value >= Uint128::new(1u128),
            ContractError::NftMarketplaceError(InvalidMarketplaceFee {})
        );
        Ok(Config {
            contract_enabled,
            contract_owner: api.addr_validate(&contract_owner)?.to_string(),
            accepted_ibc_denominations,
            marketplace_pct_fees: Config::checked_marketplace_fees(marketplace_pct_fees)?,
            marketplace_listing_fee_value,
            marketplace_listing_fee_denom,
            oracle_contract_address: api.addr_validate(&oracle_contract_address)?.to_string(),
            accepted_nft_code_ids,
        })
    }
    pub fn checked_marketplace_fees(
        marketplace_pct_fees: Decimal,
    ) -> Result<Decimal, ContractError> {
        ensure!(
            marketplace_pct_fees > Decimal::from_str("0.01").unwrap()
                && marketplace_pct_fees < Decimal::from_str("0.1").unwrap(),
            ContractError::NftMarketplaceError(InvalidMarketplaceFee {})
        );
        Ok(marketplace_pct_fees)
    }
}
