use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, to_binary, Addr, Attribute, BankMsg, Decimal, Deps, Event, Response, StdError, Uint128,
    WasmMsg,
};
use cw721::Cw721ExecuteMsg;

use general_utils::denominations::DenominationValue;

use crate::inputs::Buyer;
use crate::nft_offer::NftOffer;
use crate::nft_sale::NftSale;
use crate::profile::Profile;
use crate::reward_system::{RewardSystem, VipLevel};

#[cw_serde]
pub struct ResponseHandler {
    pub response: Response,
}

#[cw_serde]
pub struct RoyaltiesInfoResponse {
    pub address: Addr,
    // Note that this must be the same denom as that passed in to RoyaltyInfo
    // rounding up or down is at the discretion of the implementer
    pub royalty_amount: Uint128,
}

impl ResponseHandler {
    pub fn init_response() -> Self {
        ResponseHandler {
            response: Response::new()
                .add_attribute("action", "Instantiate NFT Marketplace contract"),
        }
    }

    pub fn update_config() -> Self {
        ResponseHandler {
            response: Response::new().add_attribute("action", "Admin update Config state"),
        }
    }

    pub fn add_nft_collection(nft_collection_address: &str) -> Self {
        let mut response = Response::new();
        response = response
            .add_attribute("action", "Add new NFT collection")
            .add_attribute("collection address", nft_collection_address.to_string());

        ResponseHandler { response }
    }

    pub fn transfer_my_nft(
        token_id: String,
        nft_collection_address: String,
        recipient: String,
        create_profile_msg: Option<WasmMsg>,
        cancel_sale: Option<WasmMsg>,
    ) -> Result<Self, StdError> {
        let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
            token_id,
            recipient: recipient.clone(),
        };
        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: nft_collection_address,
            msg: to_binary(&cw721_transfer_msg)?,
            funds: vec![],
        };

        let mut response = Response::new();
        if let Some(create_profile_msg) = create_profile_msg {
            response = response.add_message(create_profile_msg);
        }

        if let Some(cancel_sale) = cancel_sale {
            response = response.add_message(cancel_sale);
        }
        response = response
            .add_attribute("action", "Transfer my NFT")
            .add_attribute("Transfer to", recipient)
            .add_message(exec_cw721_transfer);

        Ok(ResponseHandler { response })
    }

    pub fn enable_disable(enabled: bool) -> Self {
        ResponseHandler {
            response: Response::new()
                .add_attribute("action", "Enable Disable Contract")
                .add_attribute("Contract status is now", enabled.to_string()),
        }
    }

    pub fn claim_marketplace_fees(owner: String, vec_of_fees: Vec<DenominationValue>) -> Self {
        let mut response = Response::new();
        let vec_of_messages: Vec<BankMsg> = vec_of_fees
            .iter()
            .map(|denom_value| BankMsg::Send {
                to_address: owner.clone(),
                amount: vec![coin(
                    denom_value.value.clone().u128(),
                    denom_value.denom.clone(),
                )],
            })
            .collect();
        response = response
            .add_event(Event::new("Claim marketplace fees"))
            .add_messages(vec_of_messages);

        ResponseHandler { response }
    }

    pub fn execute_update_sale(cancel_sale: WasmMsg, make_sale: WasmMsg) -> Self {
        let mut response = Response::new().add_event(Event::new("Update Existing Sale"));
        response = response.add_message(cancel_sale);
        response = response.add_message(make_sale);

        ResponseHandler { response }
    }

    pub fn execute_accept_offer(
        accepter: String,
        offerer: String,
        cancel_sale: Option<WasmMsg>,
        sell_nft: WasmMsg,
        buy_nft: WasmMsg,
        answer_msg: String,
    ) -> Self {
        let mut response = Response::new()
            .add_event(Event::new("Accept Offer"))
            .add_attribute("Accepter", accepter)
            .add_attribute("Offerer", offerer)
            .add_attribute("Answer Message", answer_msg);
        if let Some(cancel_sale) = cancel_sale {
            response = response.clone().add_message(cancel_sale);
        }
        response = response.clone().add_message(sell_nft);
        response = response.clone().add_message(buy_nft);

        ResponseHandler { response }
    }

    pub fn offer_expired() -> Self {
        ResponseHandler {
            response: Response::new()
                .add_attribute("action", "Accepted Expired Offer")
                .add_attribute("outcome", "Acceptation of the offer is cancelled"),
        }
    }

    pub fn register_nft_sale_response(
        nft_for_sale: NftSale,
        create_profile_msg: Option<WasmMsg>,
    ) -> Self {
        let mut response = Response::new();
        if let Some(create_profile_msg) = create_profile_msg {
            response = response.add_message(create_profile_msg);
        }
        response = response.add_event(
            Event::new("Register NFT For Sale")
                .add_attribute("Seller", nft_for_sale.seller.to_string())
                .add_attribute("Token", nft_for_sale.token_id.to_string())
                .add_attribute(
                    "Collection",
                    nft_for_sale.nft_collection_address.to_string(),
                )
                .add_attribute("Price", nft_for_sale.sale_price_value.to_string())
                .add_attribute("Denom", nft_for_sale.sale_price_denom),
        );

        Self { response }
    }

    pub fn expired_nft_sale_response(buyer: Buyer) -> Self {
        let response = Response::new()
            .add_message(BankMsg::Send {
                to_address: buyer.sender.clone(),
                amount: vec![coin(buyer.amount.clone().u128(), buyer.denom.clone())],
            })
            .add_event(
                Event::new("Purchase Cancelled")
                    .add_attribute("Sale status", "Expired")
                    .add_attribute("Fund value returned", buyer.amount.to_string())
                    .add_attribute("Fund denom returned", buyer.denom.to_string())
                    .add_attribute("Fund returned to buyer", buyer.sender),
            );

        Self { response }
    }

    pub fn cancel_nft_sale_response(nft_for_sale_info: NftSale) -> Self {
        let response = Response::new().add_event(
            Event::new("Sale Cancelled")
                .add_attribute("Sale status", "Cancelled")
                .add_attribute("Token ID", nft_for_sale_info.token_id.to_string())
                .add_attribute("Collection", nft_for_sale_info.nft_collection_address),
        );
        Self { response }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute_succes_nft_sale_response(
        _deps: Deps,
        buyer: Buyer,
        nft_for_sale_info: NftSale,
        marketplace_fees_pct: Decimal,
        royalties: Vec<RoyaltiesInfoResponse>,
        seller_profile: Profile,
        _buyer_profile: Profile,
        reward_system: RewardSystem,
        realised_sale_value_usdc: Uint128,
    ) -> Result<Self, StdError> {
        // Discount for VIP system
        let mut discount_from_vip: Decimal = Decimal::from_str("1.0").unwrap();
        for vip_lvl in reward_system.vip_perks {
            if Some(vip_lvl.vip_level) == seller_profile.vip_level.clone() {
                discount_from_vip =
                    discount_from_vip.checked_sub(vip_lvl.marketplace_fees_discount)?;
            }
        }

        // For each sale and buy -> send reward tokens
        let mut reward_transfer_msgs: Vec<BankMsg> = Vec::with_capacity(2_usize);
        for addr in vec![buyer.sender.clone(), seller_profile.address].into_iter() {
            let reward_tokens_to_dist: Uint128 = realised_sale_value_usdc
                .checked_div(Uint128::new(1_000_000u128))?
                .checked_mul(reward_system.reward_token_per_1usdc_volume)?;
            if reward_tokens_to_dist >= Uint128::new(1u128) {
                reward_transfer_msgs.push(BankMsg::Send {
                    to_address: addr.to_string(),
                    amount: vec![coin(
                        reward_tokens_to_dist.u128(),
                        reward_system.reward_token_address.clone(),
                    )],
                });
            }
        }

        let mut response = Response::new();
        let total_sale_price_value = nft_for_sale_info.sale_price_value;
        let marketplace_revenues = NftSale::compute_marketplace_fees(
            marketplace_fees_pct.checked_mul(discount_from_vip)?,
            nft_for_sale_info.sale_price_value,
        );
        let attributes_for_royalties: Vec<Attribute> = royalties
            .iter()
            .filter(|royalty| royalty.royalty_amount > Uint128::zero())
            .map(|royalty| {
                let owned_royalty_owner = format!("Royalty Receiver {}", royalty.address);
                Attribute {
                    key: owned_royalty_owner,
                    value: royalty.royalty_amount.clone().to_string(),
                }
            })
            .collect();
        let messages_for_royalties: Vec<BankMsg> = royalties
            .iter()
            .filter(|royalty| royalty.royalty_amount > Uint128::zero())
            .map(|royalty| BankMsg::Send {
                to_address: royalty.address.clone().to_string(),
                amount: vec![coin(
                    royalty.royalty_amount.clone().u128(),
                    nft_for_sale_info.sale_price_denom.clone(),
                )],
            })
            .collect();

        let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
            token_id: nft_for_sale_info.token_id.to_string(),
            recipient: buyer.sender.to_string(),
        };
        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: nft_for_sale_info.nft_collection_address.to_string(),
            msg: to_binary(&cw721_transfer_msg)?,
            funds: vec![],
        };

        response = response
            .add_event(
                Event::new("NFT Sold")
                    .add_attribute("Sold by", nft_for_sale_info.seller.to_string())
                    .add_attribute("Sold to", buyer.sender.to_string())
                    .add_attribute("Token ID", nft_for_sale_info.token_id.to_string())
                    .add_attribute(
                        "Collection",
                        nft_for_sale_info.nft_collection_address.to_string(),
                    )
                    .add_attribute("Sold for", nft_for_sale_info.sale_price_value.to_string())
                    .add_attribute(
                        "Sold in denom",
                        nft_for_sale_info.sale_price_denom.to_string(),
                    )
                    .add_attribute("Marketplace fees", marketplace_revenues.to_string()),
            )
            .add_message(BankMsg::Send {
                to_address: nft_for_sale_info.seller,
                amount: vec![coin(
                    total_sale_price_value.u128()
                        - marketplace_revenues.clone().u128()
                        - royalties
                            .iter()
                            .map(|r| r.royalty_amount)
                            .sum::<Uint128>()
                            .u128(),
                    buyer.denom,
                )],
            })
            .add_message(exec_cw721_transfer)
            .add_messages(reward_transfer_msgs);
        if !messages_for_royalties.is_empty() {
            response = response.add_attributes(attributes_for_royalties);
            response = response.add_messages(messages_for_royalties);
        }
        Ok(ResponseHandler { response })
    }

    pub fn execute_rejected_or_expired_offer(
        cancel_msg: WasmMsg,
        offer: NftOffer,
        if_accepted: bool,
        answer_msg: String,
    ) -> Result<Self, StdError> {
        let type_of_outcome = if if_accepted {
            "Accepted an Expired Offer".to_string()
        } else {
            "Offer has been Rejected".to_string()
        };
        let mut response = Response::new();
        response = response
            .add_event(
                Event::new(type_of_outcome)
                    .add_attribute("Action", "Offer has been removed".to_string())
                    .add_attribute("Action", "Offerer is getting reimbursed".to_string())
                    .add_attribute("Answer Message", answer_msg)
                    .add_attribute("Sold by", offer.offerer_address.to_string())
                    .add_attribute("Token ID", offer.token_id.to_string())
                    .add_attribute("Collection", offer.nft_collection_address.to_string())
                    .add_attribute("Offer for", offer.offer_price_value.clone().to_string())
                    .add_attribute("Offer in denom", offer.offer_price_denom),
            )
            .add_message(cancel_msg);
        Ok(ResponseHandler { response })
    }

    pub fn level_up_profile(
        previous_level: VipLevel,
        current_level: VipLevel,
    ) -> Result<Self, StdError> {
        let response = Response::new()
            .add_event(Event::new("Level Up Profile"))
            .add_attribute("From level", previous_level.to_string())
            .add_attribute("To level", current_level.to_string());
        Ok(Self { response })
    }

    pub fn send_message(
        sender_address: String,
        sender_username: Option<String>,
        receiver_address: String,
        receiver_username: Option<String>,
    ) -> Self {
        let response = Response::new()
            .add_event(Event::new("Send Message"))
            .add_attribute("From Address", sender_address)
            .add_attribute(
                "From Username",
                sender_username.unwrap_or_else(|| "Does not exist".to_string()),
            )
            .add_attribute("To Address", receiver_address)
            .add_attribute(
                "To Username",
                receiver_username.unwrap_or_else(|| "Does not exist".to_string()),
            );
        Self { response }
    }

    pub fn create_or_update_profile(profile: Profile) -> Result<Self, StdError> {
        let response = Response::new()
            .add_event(Event::new("Profile"))
            .add_attribute("Profile Address", profile.address)
            .add_attribute(
                "Profile Picture Collection",
                if profile.profile_nft_collection.is_none() {
                    "None".to_string()
                } else {
                    profile.profile_nft_collection.unwrap()
                },
            )
            .add_attribute(
                "Profile Picture Token ID",
                if profile.profile_nft_token_id.is_none() {
                    "None".to_string()
                } else {
                    profile.profile_nft_token_id.unwrap()
                },
            )
            .add_attribute(
                "Profile Background Collection",
                if profile.background_nft_collection.is_none() {
                    "None".to_string()
                } else {
                    profile.background_nft_collection.unwrap()
                },
            )
            .add_attribute(
                "Profile Picture Token ID",
                if profile.background_nft_token_id.is_none() {
                    "None".to_string()
                } else {
                    profile.background_nft_token_id.unwrap()
                },
            )
            .add_attribute(
                "Profile Description",
                if profile.description.is_none() {
                    "None".to_string()
                } else {
                    profile.description.unwrap()
                },
            )
            .add_attribute(
                "Profile Links",
                if profile.links.is_none() {
                    "Links not updated"
                } else {
                    "Links updated"
                },
            )
            .add_attribute(
                "Profile NFT Showcase",
                if profile.nft_showcase.is_none() {
                    "NFT Showcase not updated"
                } else {
                    "NFT Showcase updated"
                },
            );
        Ok(ResponseHandler { response })
    }

    pub fn nft_offer_response(nft_offer: NftOffer, create_profile_msg: Option<WasmMsg>) -> Self {
        let mut response = Response::new();
        if let Some(create_profile_msg) = create_profile_msg {
            response = response.add_message(create_profile_msg);
        }
        response = response.add_event(
            Event::new("NFT Offer".to_string())
                .add_attribute("Offerer", nft_offer.offerer_address.to_string())
                .add_attribute("Token ID", nft_offer.token_id.to_string())
                .add_attribute("Collection", nft_offer.nft_collection_address.to_string())
                .add_attribute("Amount Offered", nft_offer.offer_price_value.to_string())
                .add_attribute("Denom offered", nft_offer.offer_price_denom.to_string())
                .add_attribute("Expiration", nft_offer.offer_expiration.to_string()),
        );
        Self { response }
    }

    pub fn nft_cancel_offer_response(nft_offer: NftOffer) -> Self {
        let response = Response::new()
            .add_event(
                Event::new("NFT Offer Cancellation")
                    .add_attribute("Offerer", nft_offer.offerer_address.to_string())
                    .add_attribute("Token ID", nft_offer.token_id.to_string())
                    .add_attribute("Collection", nft_offer.nft_collection_address.to_string())
                    .add_attribute(
                        "Amount Offered",
                        nft_offer.offer_price_value.clone().to_string(),
                    )
                    .add_attribute("Denom offered", nft_offer.offer_price_denom.to_string())
                    .add_attribute("Expiration", nft_offer.offer_expiration.to_string()),
            )
            .add_message(BankMsg::Send {
                to_address: nft_offer.offerer_address.clone(),
                amount: vec![coin(
                    nft_offer.offer_price_value.clone().u128(),
                    nft_offer.offer_price_denom,
                )],
            });
        Self { response }
    }
}
