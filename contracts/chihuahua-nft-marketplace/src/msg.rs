use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{coins, to_binary, Uint128, WasmMsg};

use general_utils::denominations::{AcceptedDenominations, Denomination};
use general_utils::error::ContractError;
use nft_marketplace_utils::nft_collection::{
    NftCollectionAddress, NftCollectionAddressTokenId, NftContractInfo, NftContractType, TokenId,
};
use nft_marketplace_utils::nft_offer::NftOffer;
use nft_marketplace_utils::nft_sale::NftSale;
use nft_marketplace_utils::profile::{Profile, ProfileUpdateAction, TradeInfo};
use nft_marketplace_utils::reward_system::RewardSystem;

#[cw_serde]
pub struct InstantiateMsg {
    pub contract_owner: String,
    pub accepted_ibc_denominations: AcceptedDenominations,
    pub accepted_nft_code_ids: Vec<NftContractInfo>,
    pub marketplace_pct_fees_decimal_string: String,
    pub marketplace_listing_fee_value: Uint128,
    pub marketplace_listing_fee_denom: Denomination,
    pub oracle_contract_address: String,
    pub reward_system: RewardSystem,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    RemoveSomeExpiredSales {},
    UpdateConfig {
        list_of_updates: Vec<UpdateConfigEnum>,
    },
    ClaimMarketplaceFees {},
    AddNewCollection {
        nft_collection_address: NftCollectionAddress,
        nft_contract_info: NftContractInfo,
    },
    TransferMyNft {
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        recipient: String,
    },
    SellNft {
        sale_info: NftSale,
    },
    CancelSale {
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        additional_info: Option<String>,
    },
    UpdateSale {
        sale_info: NftSale,
    },
    BuyNft {
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        additional_info: Option<String>,
    },
    Offer {
        offer: NftOffer,
    },
    CancelOffer {
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        additional_info: Option<String>,
    },
    AnswerOffer {
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        from: String,
        if_accepted: bool,
        answer_msg: Option<String>,
    },
    CreateMyProfile {
        additional_info: Option<String>,
    },
    UpdateMyProfile {
        profile: Profile,
        profile_update_action: ProfileUpdateAction,
    },
    SendMessage {
        to: String,
        message: String,
    },
    LevelUpProfile {}
}

impl ExecuteMsg {
    pub fn wasm_execute_message_create_profile(
        contract_address: String,
        additional_info: Option<String>,
    ) -> Result<Option<WasmMsg>, ContractError> {
        Ok(Some(WasmMsg::Execute {
            contract_addr: contract_address,
            msg: to_binary(&ExecuteMsg::CreateMyProfile { additional_info })?,
            funds: vec![],
        }))
    }

    pub fn wasm_execute_message_cancel_sale(
        nft_collection_address: String,
        contract_address: String,
        token_id: TokenId,
        seller: String,
    ) -> Result<Option<WasmMsg>, ContractError> {
        Ok(Some(WasmMsg::Execute {
            contract_addr: contract_address,
            msg: to_binary(&ExecuteMsg::CancelSale {
                nft_collection_address,
                token_id,
                additional_info: Some(seller),
            })?,
            funds: vec![],
        }))
    }

    pub fn wasm_execute_message_sell(
        marketplace_listing_fee_value: Uint128,
        contract_address: String,
        sale_info: NftSale,
        marketplace_listing_fee_denom: String,
    ) -> Result<Option<WasmMsg>, ContractError> {
        Ok(Some(WasmMsg::Execute {
            contract_addr: contract_address,
            msg: to_binary(&ExecuteMsg::SellNft { sale_info })?,
            // Fees are from to the contract
            funds: coins(
                marketplace_listing_fee_value.u128(),
                marketplace_listing_fee_denom,
            ),
        }))
    }

    pub fn wasm_execute_cancel_offer(
        nft_collection_address: String,
        contract_address: String,
        token_id: TokenId,
        offerer_address: String,
    ) -> Result<Option<WasmMsg>, ContractError> {
        Ok(Some(WasmMsg::Execute {
            contract_addr: contract_address,
            msg: to_binary(&ExecuteMsg::CancelOffer {
                nft_collection_address,
                token_id,
                additional_info: Some(offerer_address),
            })?,
            funds: vec![],
        }))
    }

    pub fn wasm_execute_buy_nft(
        nft_collection_address: String,
        contract_address: String,
        token_id: TokenId,
        offerer_address: String,
        offer_price_value: Uint128,
        offer_price_denom: String,
    ) -> Result<Option<WasmMsg>, ContractError> {
        Ok(Some(WasmMsg::Execute {
            contract_addr: contract_address,
            msg: to_binary(&ExecuteMsg::BuyNft {
                nft_collection_address,
                token_id,
                additional_info: Some(offerer_address),
            })?,
            funds: coins(offer_price_value.u128(), offer_price_denom),
        }))
    }
}

#[cw_serde]
pub enum UpdateConfigEnum {
    EnableDisable {},
    UpdateAcceptedNftContracts { contracts: Vec<NftContractInfo> },
    AddDenoms { denoms: Vec<Denomination> },
    RemoveDenoms { denoms: Vec<Denomination> },
    UpdateOwner { address: String },
    UpdateRewardSystem { reward_system: RewardSystem },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(nft_marketplace_utils::config::ConfigRewardGenStatsMsg)]
    GetConfig {},
    #[returns(Uint128)]
    GetMarketplaceVolume {},
    #[returns(Uint128)]
    GetNftCollectionVolume {
        nft_collection_address: NftCollectionAddress,
    },
    #[returns(NftContractType)]
    GetNftCollectionType {
        nft_collection_address: NftCollectionAddress,
    },
    #[returns(nft_marketplace_utils::nft_collection::NftCollectionInfoByDenom)]
    GetNftCollectionInfo {
        nft_collection_address: NftCollectionAddress,
    },
    #[returns(nft_marketplace_utils::nft_sale::NftSale)]
    GetNftForSaleInfo {
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
    },
    #[returns(Vec<nft_marketplace_utils::nft_sale::NftSale>)]
    GetSellerAllNftsForSale {
        seller_address: String,
        start_after_collection_token_id: Option<NftCollectionAddressTokenId>,
        output_length: Option<u32>,
    },
    #[returns(Vec<nft_marketplace_utils::nft_sale::NftSale>)]
    GetAllTokensByCollAndIfForSale {
        address: String,
        nft_collection_address: NftCollectionAddress,
        output_length: Option<u32>,
    },
    #[returns(Vec<nft_marketplace_utils::nft_sale::NftSale>)]
    GetCollectionAllNftsForSale {
        nft_collection_address: NftCollectionAddress,
        start_after_token_id: Option<TokenId>,
        output_length: Option<u32>,
    },
    #[returns(Vec<nft_marketplace_utils::marketplace_statistics::MarketplaceStatsByDenom>)]
    GetMarketplaceInfo {},
    #[returns(Vec<nft_marketplace_utils::nft_sale::TokenSaleHistory>)]
    GetTokenIdSaleHistory {
        token_id: TokenId,
        nft_collection_address: NftCollectionAddress,
    },
    #[returns(nft_marketplace_utils::profile::Profile)]
    GetProfileInfo { address_or_username: String },
    #[returns(nft_marketplace_utils::nft_offer::NftOffer)]
    GetAllOffersTokenId {
        token_id: TokenId,
        nft_collection_address: NftCollectionAddress,
        start_after: Option<(NftCollectionAddress, TokenId)>,
        output_length: Option<u32>,
    },
    #[returns(Vec<nft_marketplace_utils::nft_offer::NftOffer>)]
    GetAllOffersAddress {
        address: String,
        start_after: Option<(NftCollectionAddress, TokenId)>,
        output_length: Option<u32>,
    },
    #[returns(Vec<nft_marketplace_utils::nft_collection::NftCollectionAddress>)]
    GetTokenIdsByCollection {
        address: String,
        list_of_collections: Vec<NftCollectionAddress>,
    },
}
