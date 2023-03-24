use std::fmt;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Decimal, Api};
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{AlreadyLevel3, InvalidAmountReceivedForLevelUp, InvalidLevelUp, InvalidRewards, NeedToFillAllThePerks};

#[cw_serde]
pub struct RewardSystem {
    pub reward_token_address: String,
    pub reward_token_per_1usdc_volume: Uint128,
    pub total_reward_tokens_distributed: Uint128,
    pub vip_perks: Vec<VipPerk>
}

impl RewardSystem {
    pub fn new_checked(
        api: &dyn Api,
        reward_token_address: String,
        reward_token_per_1usdc_volume: Uint128,
        total_reward_tokens_distributed: Uint128,
        vip_perks: Vec<VipPerk>
    ) -> Result<Self, ContractError> {
        if reward_token_per_1usdc_volume < Uint128::new(1u128) {
            return Err(ContractError::NftMarketplace(InvalidRewards {}))
        }
        if total_reward_tokens_distributed != Uint128::zero() {
            return Err(ContractError::NftMarketplace(InvalidRewards {}))
        }
        let mut seen = [false; 3];
        for perk in &vip_perks {
            match perk.vip_level {
                VipLevel::Level0 => {},
                VipLevel::Level1 => seen[0] = true,
                VipLevel::Level2 => seen[1] = true,
                VipLevel::Level3 => seen[2] = true,
            }
        }
        if !(seen.iter().all(|&p| p)) {
            return Err(ContractError::NftMarketplace(NeedToFillAllThePerks {}))
        }
        Ok(
            RewardSystem {
                reward_token_address: api.addr_validate(&reward_token_address)?.to_string(),
                reward_token_per_1usdc_volume,
                total_reward_tokens_distributed,
                vip_perks,
            }
        )
    }
}

#[cw_serde]
pub enum VipLevel {
    Level0,
    Level1,
    Level2,
    Level3,
}

impl fmt::Display for VipLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VipLevel::Level0 => write!(f, "Level0"),
            VipLevel::Level1 => write!(f, "Level1"),
            VipLevel::Level2 => write!(f, "Level2"),
            VipLevel::Level3 => write!(f, "Level3"),
        }
    }
}

impl VipLevel {
    pub fn level_up_if_possible(
        current_level: VipLevel,
        vip_perks: Vec<VipPerk>,
        amount_received: Uint128,
    ) -> Result<bool, ContractError> {
        let next_level = current_level.clone().next_level();
        if current_level == next_level {
            return Err(ContractError::NftMarketplace(AlreadyLevel3 {}));
        }
        let next_perk = vip_perks
            .iter()
            .find(|perk| perk.vip_level == next_level.clone())
            .ok_or_else(|| ContractError::NftMarketplace(InvalidLevelUp {}))?;
        if amount_received != next_perk.price_in_reward_tokens {
            return Err(ContractError::NftMarketplace(InvalidAmountReceivedForLevelUp {}));
        }
        Ok(true)
    }

    pub fn next_level(self) -> VipLevel {
        match self {
            VipLevel::Level0 => VipLevel::Level1,
            VipLevel::Level1 => VipLevel::Level2,
            VipLevel::Level2 => VipLevel::Level3,
            VipLevel::Level3 => VipLevel::Level3,
        }
    }

}

#[cw_serde]
pub struct VipPerk {
    pub vip_level: VipLevel,
    pub profile_background: bool,
    pub profile_nft_showcase: bool,
    pub profile_description: bool,
    pub profile_links: bool,
    pub marketplace_fees_discount: Decimal,
    pub price_in_reward_tokens: Uint128
}