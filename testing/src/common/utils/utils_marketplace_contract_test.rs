#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use anyhow::Result as AnyResult;
    use cosmwasm_std::{to_binary, Addr, Decimal, Empty, MessageInfo, StdResult, Uint128};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

    use general_utils::denominations::AcceptedDenominations;
    use nft_marketplace_utils::config::ConfigRewardGenStatsMsg;
    use nft_marketplace_utils::marketplace_statistics::MarketplaceStatsByDenom;
    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftCollectionAddressTokenId, NftCollectionInfoByDenom,
        NftContractInfo, TokenId,
    };
    use nft_marketplace_utils::nft_offer::NftOffer;
    use nft_marketplace_utils::nft_sale::{NftSale, TokenSaleHistory, TokensAndIfSaleInfo};
    use nft_marketplace_utils::profile::{Profile, ProfileUpdateAction};
    use nft_marketplace_utils::reward_system::{RewardSystem, VipLevel, VipPerk};

    use crate::common::utils::constants::OWNER;
    use chihuahua_nft_marketplace::msg::{ReceiveMsg, UpdateConfigEnum};

    pub fn smart_contract_def_test_nft_marketplace() -> Box<dyn Contract<Empty>> {
        let smart_contract = ContractWrapper::new(
            chihuahua_nft_marketplace::contract::execute,
            chihuahua_nft_marketplace::contract::instantiate,
            chihuahua_nft_marketplace::contract::query,
        );
        Box::new(smart_contract)
    }

    pub fn default_init_msg_mkpc() -> chihuahua_nft_marketplace::msg::InstantiateMsg {
        chihuahua_nft_marketplace::msg::InstantiateMsg {
            contract_owner: OWNER.to_string(),
            accepted_ibc_denominations: AcceptedDenominations {
                list_of_denoms: vec![
                    "uhuahua".to_string(),
                    "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                        .to_string(),
                ],
            },
            marketplace_listing_fee_value: Uint128::new(6900000),
            marketplace_listing_fee_denom: "uhuahua".to_string(),
            oracle_contract_address: "".to_string(),
            marketplace_pct_fees_decimal_string: "0.042".to_string(),
            reward_system: RewardSystem {
                reward_token_address: "".to_string(),
                reward_token_per_1usdc_volume: Uint128::new(1_000_000u128),
                total_reward_tokens_distributed: Uint128::zero(),
                vip_perks: vec![
                    VipPerk {
                        vip_level: VipLevel::Level1,
                        profile_background: true,
                        profile_nft_showcase: false,
                        profile_description: false,
                        profile_links: false,
                        marketplace_fees_discount: Decimal::from_str("0.025").unwrap(),
                        level_up_price_in_reward_tokens: Uint128::new(1_000u128),
                    },
                    VipPerk {
                        vip_level: VipLevel::Level2,
                        profile_background: true,
                        profile_nft_showcase: false,
                        profile_description: true,
                        profile_links: false,
                        marketplace_fees_discount: Decimal::from_str("0.05").unwrap(),
                        level_up_price_in_reward_tokens: Uint128::new(10_000u128),
                    },
                    VipPerk {
                        vip_level: VipLevel::Level3,
                        profile_background: true,
                        profile_nft_showcase: true,
                        profile_description: true,
                        profile_links: true,
                        marketplace_fees_discount: Decimal::from_str("0.1").unwrap(),
                        level_up_price_in_reward_tokens: Uint128::new(50_000u128),
                    },
                ],
            },
            accepted_nft_code_ids: vec![],
        }
    }

    pub fn instantiate_smart_contract_test_nft_marketplace(
        app: &mut App,
        _accepted_ibc_denominations: AcceptedDenominations,
        oracle_contract_address: String,
        reward_token: String,
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_nft_marketplace());
        let mut init_msg = default_init_msg_mkpc();
        init_msg.oracle_contract_address = oracle_contract_address;
        init_msg.reward_system.reward_token_address = reward_token;
        app.instantiate_contract(
            contract_code_id,
            Addr::unchecked(OWNER),
            &init_msg,
            &[],
            "chihuahua_nft_marketplace_code",
            None,
        )
        .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_custom_smart_contract_test_nft_marketplace(
        app: &mut App,
        accepted_ibc_denominations: Option<AcceptedDenominations>,
        reward_token: Option<String>,
        contract_owner: Option<String>,
        marketplace_listing_fee_value: Option<Uint128>,
        marketplace_listing_fee_denom: Option<String>,
        oracle_contract_address: Option<String>,
        marketplace_pct_fees_decimal_string: Option<String>,
        reward_system: Option<RewardSystem>,
    ) -> AnyResult<Addr> {
        let contract_code_id = app.store_code(smart_contract_def_test_nft_marketplace());
        let mut init_msg = default_init_msg_mkpc();
        init_msg.reward_system = reward_system.unwrap_or(init_msg.reward_system);
        init_msg.oracle_contract_address =
            oracle_contract_address.unwrap_or(init_msg.oracle_contract_address);
        init_msg.reward_system.reward_token_address =
            reward_token.unwrap_or(init_msg.reward_system.reward_token_address);
        init_msg.accepted_ibc_denominations =
            accepted_ibc_denominations.unwrap_or(init_msg.accepted_ibc_denominations);
        init_msg.contract_owner = contract_owner.unwrap_or(init_msg.contract_owner);
        init_msg.marketplace_listing_fee_value =
            marketplace_listing_fee_value.unwrap_or(init_msg.marketplace_listing_fee_value);
        init_msg.marketplace_listing_fee_denom =
            marketplace_listing_fee_denom.unwrap_or(init_msg.marketplace_listing_fee_denom);
        init_msg.marketplace_pct_fees_decimal_string = marketplace_pct_fees_decimal_string
            .unwrap_or(init_msg.marketplace_pct_fees_decimal_string);
        app.instantiate_contract(
            contract_code_id,
            Addr::unchecked(OWNER),
            &init_msg,
            &[],
            "chihuahua_nft_marketplace_code",
            None,
        )
    }

    // Execute Functions: NFT Marketplace
    pub fn marketplace_test_exec_update_config(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        configs_to_update: Vec<UpdateConfigEnum>,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::UpdateConfig {
            list_of_updates: configs_to_update,
        };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn marketplace_test_exec_add_new_collection(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        nft_contract_info: NftContractInfo,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::AddNewCollection {
            nft_collection_address,
            nft_contract_info,
        };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn marketplace_test_exec_sell_nft(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        sale_info: NftSale,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::SellNft { sale_info };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &info.funds,
        )
    }

    pub fn marketplace_test_exec_offer(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        offer_info: NftOffer,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::Offer { offer: offer_info };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &info.funds,
        )
    }

    pub fn marketplace_test_exec_transfer_my_nft(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        recipient: String,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::TransferMyNft {
            nft_collection_address,
            token_id,
            recipient,
        };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_cancel_offer(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::CancelOffer {
            nft_collection_address,
            token_id,
            additional_info: None,
        };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn marketplace_test_exec_answer_offer(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        from: String,
        if_accepted: bool,
        answer_msg: Option<String>,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::AnswerOffer {
            nft_collection_address,
            token_id,
            from,
            if_accepted,
            answer_msg,
        };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_cancel_sale(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        additional_info: Option<String>,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::CancelSale {
            nft_collection_address,
            token_id,
            additional_info,
        };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_send_message(
        app: &mut App,
        nft_marketplace_contract_addr: String,
        info: MessageInfo,
        to: String,
        message: String,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::SendMessage { to, message };
        app.execute_contract(
            info.sender,
            Addr::unchecked(nft_marketplace_contract_addr),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_update_sale(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        sale_info: NftSale,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::UpdateSale { sale_info };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_buy_nft(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        additional_info: Option<String>,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::BuyNft {
            nft_collection_address,
            token_id,
            additional_info,
        };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &info.funds,
        )
    }

    pub fn marketplace_test_exec_enable_disable(
        app: &mut App,
        nft_marketplace_contract_addr: String,
        info: MessageInfo,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::UpdateConfig {
            list_of_updates: vec![UpdateConfigEnum::EnableDisable {}],
        };
        app.execute_contract(
            info.sender,
            Addr::unchecked(nft_marketplace_contract_addr),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_add_nft_code_id(
        app: &mut App,
        nft_marketplace_contract_addr: String,
        nft_contracts_info: Vec<NftContractInfo>,
        info: MessageInfo,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::UpdateConfig {
            list_of_updates: vec![UpdateConfigEnum::UpdateAcceptedNftContracts {
                contracts: nft_contracts_info,
            }],
        };
        app.execute_contract(
            info.sender,
            Addr::unchecked(nft_marketplace_contract_addr),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_claim_mkpc_fees(
        app: &mut App,
        nft_marketplace_contract_addr: String,
        info: MessageInfo,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::ClaimMarketplaceFees {};
        app.execute_contract(
            info.sender,
            Addr::unchecked(nft_marketplace_contract_addr),
            &msg,
            &[],
        )
    }

    pub fn marketplace_test_exec_create_my_profile(
        app: &mut App,
        contract_addr: String,
        info: MessageInfo,
        additional_info: Option<String>,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::CreateMyProfile { additional_info };
        app.execute_contract(info.sender, Addr::unchecked(contract_addr), &msg, &[])
    }

    pub fn marketplace_test_exec_rm_exp_sale(
        app: &mut App,
        contract_addr: String,
        info: MessageInfo,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::RemoveExpiredSale {};
        app.execute_contract(info.sender, Addr::unchecked(contract_addr), &msg, &[])
    }

    pub fn marketplace_test_exec_update_my_profile(
        app: &mut App,
        contract_addr: String,
        info: MessageInfo,
        profile: Profile,
        profile_update_action: ProfileUpdateAction,
    ) -> AnyResult<AppResponse> {
        let msg = chihuahua_nft_marketplace::msg::ExecuteMsg::UpdateMyProfile {
            profile,
            profile_update_action,
        };
        app.execute_contract(info.sender, Addr::unchecked(contract_addr), &msg, &[])
    }

    pub fn marketplace_test_exec_lvl_up_profile(
        app: &mut App,
        info: MessageInfo,
        contract_addr: &Addr,
        cw20_denom: String,
        amount_of_tokens: Uint128,
        cw20_msg: ReceiveMsg,
    ) -> AnyResult<AppResponse> {
        let msg = cw20::Cw20ExecuteMsg::Send {
            contract: contract_addr.to_string(),
            amount: amount_of_tokens,
            msg: to_binary(&cw20_msg).unwrap(),
        };
        app.execute_contract(info.sender, Addr::unchecked(cw20_denom), &msg, &info.funds)
    }

    // Query Functions: NFT Marketplace
    pub fn marketplace_test_query_get_config<T: Into<String>>(
        app: &App,
        contract_addr: T,
    ) -> ConfigRewardGenStatsMsg {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetConfig {};
        let result: ConfigRewardGenStatsMsg =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    pub fn marketplace_test_query_get_mkpc_vol<T: Into<String>>(
        app: &App,
        contract_addr: T,
    ) -> Uint128 {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetMarketplaceVolume {};
        let result: Uint128 = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    pub fn marketplace_test_query_get_nft_coll_vol<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress,
    ) -> Uint128 {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetNftCollectionVolume {
                nft_collection_address,
            };
        let result: Uint128 = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    pub fn marketplace_test_query_get_profile_info(
        app: &App,
        contract_addr: String,
        address: String,
    ) -> StdResult<Profile> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetProfileInfo {
                address_or_username: address,
            };
        let result: StdResult<Profile> = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_nft_for_sale_info<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
    ) -> StdResult<NftSale> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetNftForSaleInfo {
                nft_collection_address,
                token_id,
            };
        let result: StdResult<NftSale> = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_token_sale_hist<T: Into<String>>(
        app: &App,
        contract_addr: T,
        collection_address: NftCollectionAddress,
        token_id: TokenId,
    ) -> StdResult<Vec<TokenSaleHistory>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetTokenIdSaleHistory {
                nft_collection_address: collection_address,
                token_id,
            };
        let result: StdResult<Vec<TokenSaleHistory>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_seller_nfts_for_sale<T: Into<String>>(
        app: &App,
        contract_addr: T,
        seller_address: String,
        start_after_token_id: Option<NftCollectionAddressTokenId>,
        output_length: Option<u32>,
    ) -> StdResult<Vec<NftSale>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetSellerAllNftsForSale {
                seller_address,
                start_after_collection_token_id: start_after_token_id,
                output_length,
            };
        let result: StdResult<Vec<NftSale>> = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_tokens_by_coll<T: Into<String>>(
        app: &App,
        contract_addr: T,
        address: String,
        nft_collection_address: NftCollectionAddress,
        output_length: Option<u32>,
    ) -> StdResult<Vec<TokensAndIfSaleInfo>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetAllTokensByCollAndIfForSale {
                address,
                nft_collection_address,
                output_length,
            };
        let result: StdResult<Vec<TokensAndIfSaleInfo>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_all_offers_address<T: Into<String>>(
        app: &App,
        contract_addr: T,
        offerer_address: String,
        start_after_token_id: Option<(NftCollectionAddress, TokenId)>,
        output_length: Option<u32>,
    ) -> StdResult<Vec<NftOffer>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetAllOffersAddress {
                address: offerer_address,
                start_after: start_after_token_id,
                output_length,
            };
        let result: StdResult<Vec<NftOffer>> = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_all_offers_token<T: Into<String>>(
        app: &App,
        contract_addr: T,
        token_id: TokenId,
        nft_collection_address: NftCollectionAddress,
        start_after: Option<(NftCollectionAddress, TokenId)>,
        output_length: Option<u32>,
    ) -> StdResult<Vec<NftOffer>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetAllOffersTokenId {
                token_id,
                nft_collection_address,
                output_length,
                start_after,
            };
        let result: StdResult<Vec<NftOffer>> = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_mkpc_info<T: Into<String>>(
        app: &App,
        contract_addr: T,
    ) -> StdResult<Vec<MarketplaceStatsByDenom>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetMarketplaceInfo {};
        let result: StdResult<Vec<MarketplaceStatsByDenom>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_coll_all_nfts_for_sale<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress,
        start_after_token_id: Option<TokenId>,
        output_length: Option<u32>,
    ) -> StdResult<Vec<NftSale>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetCollectionAllNftsForSale {
                nft_collection_address,
                start_after_token_id,
                output_length,
            };
        let result: StdResult<Vec<NftSale>> = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_nft_coll_info<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress,
    ) -> StdResult<Vec<NftCollectionInfoByDenom>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetNftCollectionInfo {
                nft_collection_address,
            };
        let result: StdResult<Vec<NftCollectionInfoByDenom>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn marketplace_test_query_get_token_ids_by_coll<T: Into<String>>(
        app: &App,
        contract_addr: T,
        address: String,
        list_of_collections: Vec<NftCollectionAddress>,
    ) -> StdResult<Vec<(NftCollectionAddress, Vec<String>)>> {
        let msg: chihuahua_nft_marketplace::msg::QueryMsg =
            chihuahua_nft_marketplace::msg::QueryMsg::GetTokenIdsByCollection {
                address,
                list_of_collections,
            };
        let result: StdResult<Vec<(NftCollectionAddress, Vec<String>)>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }
}
