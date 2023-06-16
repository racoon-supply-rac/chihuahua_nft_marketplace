use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, Uint128};

use general_utils::denominations::Denomination;
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    BuyAndSellCannotBeFilledTogether, BuyAndSellCannotBeNoneTogether, InvalidNftShowcaseReceived,
    InvalidUsername,
};

use crate::helpers::check_if_nft_is_owned;
use crate::nft_collection::{NftCollectionAddress, TokenId};
use crate::reward_system::{RewardSystem, VipLevel};

#[cw_serde]
pub enum ProfileUpdateAction {
    Add,
    Remove,
}

#[cw_serde]
pub struct NftShowcase {
    pub collection: NftCollectionAddress,
    pub token_id: TokenId,
}

#[cw_serde]
pub struct Socials {
    pub twitter_link: Option<String>,
    pub discord_link: Option<String>,
    pub telegram_link: Option<String>,
    pub additional_social_link: Option<String>,
}

#[cw_serde]
pub struct TradeInfo {
    pub denom: Denomination,
    pub volume_value: Uint128,
}

#[cw_serde]
pub struct ProfileMessage {
    pub from_address: String,
    pub from_username: Option<String>,
    pub message: String,
}

#[cw_serde]
pub struct ProfileMessages {
    pub display_on_profile: bool,
    pub messages: Vec<ProfileMessage>,
}

#[cw_serde]
pub struct Profile {
    pub address: String,
    pub username: Option<String>,
    pub vip_level: Option<VipLevel>,
    pub profile_nft_collection: Option<NftCollectionAddress>,
    pub profile_nft_token_id: Option<TokenId>,
    pub background_nft_collection: Option<NftCollectionAddress>,
    pub background_nft_token_id: Option<TokenId>,
    pub description: Option<String>,
    pub nft_showcase: Option<Vec<NftShowcase>>,
    pub links: Option<Socials>,
    pub profile_messages: Option<ProfileMessages>,
    pub number_of_trades: Option<u64>,
    pub buy_info: Option<Vec<TradeInfo>>,
    pub sell_info: Option<Vec<TradeInfo>>,
    pub display_trade_info: Option<bool>,
}

impl Profile {
    pub fn new(address: String) -> Self {
        Profile {
            address,
            username: None,
            vip_level: Some(VipLevel::Level0),
            profile_nft_collection: None,
            profile_nft_token_id: None,
            background_nft_collection: None,
            background_nft_token_id: None,
            description: None,
            nft_showcase: None,
            links: None,
            profile_messages: Some(ProfileMessages { display_on_profile: true, messages: vec![] }),
            number_of_trades: Some(0),
            buy_info: Some(vec![]),
            sell_info: Some(vec![]),
            display_trade_info: Some(false),
        }
    }

    pub fn user_update_profile(
        mut self,
        deps: Deps,
        new_profile_info: Profile,
        reward_system: RewardSystem,
        profile_update_action: ProfileUpdateAction,
    ) -> Result<Self, ContractError> {
        if let Some(new_nft_showcase) = new_profile_info.nft_showcase.clone() {
            if new_nft_showcase.len() > 4 {
                return Err(ContractError::NftMarketplaceError(InvalidNftShowcaseReceived {}));
            }
        }

        match profile_update_action {
            ProfileUpdateAction::Add => {
                if let Some(display_trade_info) = new_profile_info.display_trade_info {
                    self.display_trade_info = Some(display_trade_info);
                }
                if let Some(new_profile_messages) = new_profile_info.profile_messages {
                    let mut current_message_info =
                        self.profile_messages
                            .clone()
                            .unwrap_or_else(|| ProfileMessages {
                                display_on_profile: false,
                                messages: vec![],
                            });
                    current_message_info.display_on_profile =
                        new_profile_messages.display_on_profile;
                    self.profile_messages = Some(ProfileMessages {
                        display_on_profile: current_message_info.display_on_profile,
                        messages: self
                            .profile_messages
                            .clone()
                            .unwrap_or_else(|| ProfileMessages {
                                display_on_profile: current_message_info.display_on_profile,
                                messages: vec![],
                            })
                            .messages,
                    });
                }
                if new_profile_info.username.is_some() {
                    if new_profile_info.username.clone().unwrap().is_ascii()
                        && new_profile_info
                            .username
                            .clone()
                            .unwrap()
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
                        && new_profile_info.username.clone().unwrap().len() > 50
                        && new_profile_info.username.clone().unwrap().len() < 2
                    {
                        return Err(ContractError::NftMarketplaceError(InvalidUsername {}));
                    } else {
                        self.username = Some(new_profile_info.username.clone().unwrap());
                    }
                }
                if let (Some(profile_nft_collection), Some(profile_nft_token_id)) = (
                    new_profile_info.profile_nft_collection,
                    new_profile_info.profile_nft_token_id,
                ) {
                    check_if_nft_is_owned(
                        deps,
                        &self.address,
                        &profile_nft_collection,
                        &profile_nft_token_id,
                    )?;
                    self.profile_nft_collection = Some(profile_nft_collection);
                    self.profile_nft_token_id = Some(profile_nft_token_id);
                }
                for vip_perk_system in reward_system.vip_perks.into_iter() {
                    if self.vip_level == Some(vip_perk_system.vip_level) {
                        let new_links = new_profile_info.links.clone();
                        let new_showcase = new_profile_info.nft_showcase.clone();
                        let new_background_nft_collection =
                            new_profile_info.background_nft_collection.clone();
                        let new_background_nft_token_id =
                            new_profile_info.background_nft_token_id.clone();

                        if vip_perk_system.profile_description
                            && new_profile_info.description.is_some()
                        {
                            self.description = new_profile_info.description.clone();
                        }
                        if vip_perk_system.profile_background {
                            if let (Some(collection), Some(token_id)) = (
                                new_background_nft_collection.clone(),
                                new_background_nft_token_id.clone(),
                            ) {
                                check_if_nft_is_owned(deps, &self.address, &collection, &token_id)?;
                                self.background_nft_token_id = new_background_nft_token_id;
                                self.background_nft_collection = new_background_nft_collection;
                            }
                        }
                        if vip_perk_system.profile_nft_showcase {
                            if let Some(received_showcase) = new_showcase {
                                let mut current_showcase =
                                    self.nft_showcase.clone().unwrap_or_default();
                                for showcase in received_showcase {
                                    check_if_nft_is_owned(
                                        deps,
                                        &self.address,
                                        &showcase.collection,
                                        &showcase.token_id,
                                    )?;
                                    if current_showcase.len() >= 4 {
                                        current_showcase.remove(0);
                                        current_showcase.push(showcase.clone());
                                    } else {
                                        current_showcase.push(showcase.clone());
                                    }
                                }
                                self.nft_showcase = Some(current_showcase);
                            }
                        }
                        if vip_perk_system.profile_links {
                            if let Some(new_links) = new_links {
                                let mut current_links = self.links.unwrap_or(Socials {
                                    twitter_link: None,
                                    discord_link: None,
                                    telegram_link: None,
                                    additional_social_link: None,
                                });
                                if let Some(telegram_link) = new_links.telegram_link {
                                    current_links.telegram_link = Some(telegram_link);
                                }
                                if let Some(discord_link) = new_links.discord_link {
                                    current_links.discord_link = Some(discord_link);
                                }
                                if let Some(twitter_link) = new_links.twitter_link {
                                    current_links.twitter_link = Some(twitter_link);
                                }
                                if let Some(additional_social_link) =
                                    new_links.additional_social_link
                                {
                                    current_links.additional_social_link =
                                        Some(additional_social_link);
                                }
                                self.links = Some(current_links);
                            }
                        }
                    }
                }
            }
            ProfileUpdateAction::Remove => {
                if new_profile_info.display_trade_info.is_some() {
                    self.display_trade_info = None;
                }
                if new_profile_info.username.is_some() {
                    self.username = None;
                }
                if new_profile_info.profile_nft_collection.is_some()
                    && new_profile_info.profile_nft_token_id.is_some()
                {
                    self.profile_nft_collection = None;
                    self.profile_nft_token_id = None;
                }
                for vip_perk_system in reward_system.vip_perks {
                    if self.vip_level == Some(vip_perk_system.vip_level) {
                        if vip_perk_system.profile_description
                            && new_profile_info.description.is_some()
                        {
                            self.description = None;
                        }
                        if vip_perk_system.profile_background
                            && new_profile_info.background_nft_collection.is_some()
                            && new_profile_info.background_nft_token_id.is_some()
                        {
                            self.background_nft_token_id = None;
                            self.background_nft_collection = None;
                        }
                        if vip_perk_system.profile_nft_showcase
                            && new_profile_info.nft_showcase.is_some()
                        {
                            self.nft_showcase = None;
                        }
                        if vip_perk_system.profile_links && new_profile_info.links.is_some() {
                            let new_links = new_profile_info.links.clone().unwrap();
                            let mut current_links = self.links.unwrap_or(Socials {
                                twitter_link: None,
                                discord_link: None,
                                telegram_link: None,
                                additional_social_link: None,
                            });
                            if new_links.telegram_link.is_some() {
                                current_links.telegram_link = None;
                            }
                            if new_links.discord_link.is_some() {
                                current_links.discord_link = None;
                            }
                            if new_links.twitter_link.is_some() {
                                current_links.twitter_link = None;
                            }
                            if new_links.additional_social_link.is_some() {
                                current_links.additional_social_link = None;
                            }
                            self.links = Some(current_links);
                        }
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn level_up(mut self) -> Self {
        self.vip_level = Some(self.vip_level.unwrap().next_level());
        self
    }

    pub fn receive_message(
        mut self,
        from_address: String,
        from_username: Option<String>,
        message: String,
    ) -> Self {
        let new_profile_messages = if let Some(mut profile_messages) = self.profile_messages {
            if profile_messages.messages.len() >= 10 {
                profile_messages.messages.remove(0);
            }
            profile_messages.messages.push(ProfileMessage {
                from_address,
                from_username,
                message,
            });
            profile_messages
        } else {
            ProfileMessages {
                display_on_profile: false,
                messages: vec![ProfileMessage {
                    from_address,
                    from_username,
                    message,
                }],
            }
        };
        self.profile_messages = Some(new_profile_messages);
        self
    }

    pub fn realise_transaction(
        mut self,
        buy: Option<TradeInfo>,
        sell: Option<TradeInfo>,
    ) -> Result<Self, ContractError> {
        match (buy, sell) {
            (None, None) => {
                return Err(ContractError::NftMarketplaceError(
                    BuyAndSellCannotBeNoneTogether {},
                ));
            }
            (Some(_), Some(_)) => {
                return Err(ContractError::NftMarketplaceError(
                    BuyAndSellCannotBeFilledTogether {},
                ));
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
            }
            None => {
                self.number_of_trades = Some(1);
            }
        }
        Ok(self)
    }

    pub fn nft_used_in_profile_check_and_reset(
        mut self,
        token_id: TokenId,
        nft_collection_address: NftCollectionAddress,
    ) -> Result<Self, ContractError> {
        self.background_nft_collection = self
            .background_nft_collection
            .take()
            .filter(|addr| *addr != nft_collection_address);
        self.background_nft_token_id = self
            .background_nft_token_id
            .take()
            .filter(|id| *id != token_id);
        self.profile_nft_collection = self
            .profile_nft_collection
            .take()
            .filter(|addr| *addr != nft_collection_address);
        self.profile_nft_token_id = self
            .profile_nft_token_id
            .take()
            .filter(|id| *id != token_id);
        if let Some(nft_showcase) = &mut self.nft_showcase {
            *nft_showcase = nft_showcase
                .iter()
                .filter(|nft_show| {
                    nft_show.token_id != token_id || nft_show.collection != nft_collection_address
                })
                .cloned()
                .collect();
        }
        Ok(self)
    }
}
