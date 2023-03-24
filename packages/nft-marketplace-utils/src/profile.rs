use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Empty, Uint128};
use general_utils::denominations::Denomination;
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{BuyAndSellCannotBeFilledTogether, BuyAndSellCannotBeNoneTogether};
use crate::nft_collection::{NftCollectionAddress, TokenId};
use crate::reward_system::{RewardSystem, VipLevel};

#[cw_serde]
pub struct NftShowcase {
    pub collection: NftCollectionAddress,
    pub token_id: TokenId,
}

#[cw_serde]
pub struct Socials {
    pub twitter_link: String,
    pub discord_link: String,
    pub telegram_link: String,
    pub additional_social_link: String,
}

#[cw_serde]
pub struct TradeInfo {
    pub denom: Denomination,
    pub volume_value: Uint128
}

pub type ProfileExtension = Option<Empty>;

#[cw_serde]
pub struct Profile {
    pub address: String,
    pub vip_level: Option<VipLevel>,
    pub profile_nft_collection: Option<NftCollectionAddress>,
    pub profile_nft_token_id: Option<TokenId>,
    pub background_nft_collection: Option<NftCollectionAddress>,
    pub background_nft_token_id: Option<TokenId>,
    pub description: Option<String>,
    pub nft_showcase: Option<Vec<NftShowcase>>,
    pub links: Option<Socials>,
    pub extended_profile: ProfileExtension,
    pub number_of_trades: Option<u64>,
    pub buy_info: Option<Vec<TradeInfo>>,
    pub sell_info: Option<Vec<TradeInfo>>
}

impl Profile {
    pub fn new(address: String) -> Self {
        Profile {
            address,
            vip_level: Some(VipLevel::Level0),
            profile_nft_collection: None,
            profile_nft_token_id: None,
            background_nft_collection: None,
            background_nft_token_id: None,
            description: None,
            nft_showcase: None,
            links: None,
            extended_profile: None,
            number_of_trades: Some(0),
            buy_info: Some(vec![]),
            sell_info: Some(vec![]),
        }
    }

    pub fn user_update_profile(mut self, profile: Option<Profile>, reward_system: RewardSystem) -> Self {
        if let Some(profile) = profile {
            // Only change what a user can change
            self.profile_nft_collection = profile.profile_nft_collection;
            self.profile_nft_token_id = profile.profile_nft_token_id;
            for vip_perk_system in reward_system.vip_perks {
                if self.vip_level == Some(vip_perk_system.vip_level) {
                    if vip_perk_system.profile_description {
                        self.description = profile.description.clone();
                    }
                    if vip_perk_system.profile_background {
                        self.background_nft_token_id = profile.background_nft_token_id.clone();
                        self.background_nft_collection = profile.background_nft_collection.clone();
                    }
                    if vip_perk_system.profile_nft_showcase {
                        self.nft_showcase = profile.nft_showcase.clone()
                    }
                    if vip_perk_system.profile_links {
                        self.links = profile.links.clone()
                    }
                }
            }
        }
        self
    }

    pub fn level_up(mut self) -> Self {
        self.vip_level = Some(self.vip_level.unwrap().next_level());
        self
    }

    pub fn realise_transaction(
        mut self,
        buy: Option<TradeInfo>,
        sell: Option<TradeInfo>
    ) -> Result<Self, ContractError> {
        match (buy, sell) {
            (None, None) => {
                return Err(ContractError::NftMarketplace(BuyAndSellCannotBeNoneTogether {}));
            }
            (Some(_), Some(_)) => {
                return Err(ContractError::NftMarketplace(BuyAndSellCannotBeFilledTogether {}));
            }
            (Some(buy), None) => {
                if let Some(vec) = &mut self.buy_info {
                    if let Some(buying) = vec.iter_mut().find(|b| b.denom == buy.denom) {
                        buying.volume_value += buy.volume_value;
                    } else {
                        vec.push(buy);
                    }
                } else {
                    self.buy_info = Some(vec![buy]);
                }
            }
            (None, Some(sell)) => {
                if let Some(vec) = &mut self.sell_info {
                    if let Some(sale) = vec.iter_mut().find(|s| s.denom == sell.denom) {
                        sale.volume_value += sell.volume_value;
                    } else {
                        vec.push(sell);
                    }
                } else {
                    self.sell_info = Some(vec![sell]);
                }
            }
        }
        match self.number_of_trades {
            Some(n) => {
                self.number_of_trades = Some(n + 1);
            },
            None => {
                self.number_of_trades = Some(1);
            }
        }
        Ok(self)
    }

}