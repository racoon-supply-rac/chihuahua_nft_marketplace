#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use cosmwasm_std::{Addr, Decimal, Empty, MessageInfo, StdResult, to_binary, Uint128};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use general_utils::denominations::AcceptedDenominations;
    use nft_marketplace_utils::config::{ConfigRewardGenStatsMsg};
    use anyhow::Result as AnyResult;
    use nft_marketplace_utils::marketplace_statistics::MarketplaceStatsByDenom;
    use nft_marketplace_utils::nft_collection::{NftCollectionAddress, NftCollectionAddressTokenId, NftCollectionInfoByDenom, TokenId};
    use nft_marketplace_utils::nft_offer::NftOffer;
    use nft_marketplace_utils::nft_sale::{NftSale, TokenSaleHistory};
    use nft_marketplace_utils::profile::Profile;
    use nft_marketplace_utils::reward_system::{RewardSystem, VipLevel, VipPerk};
    use crate::msg::{ReceiveMsg, UpdateConfigEnum};
    use crate::tests::tests::OWNER;

    pub fn smart_contract_def_test_nft_marketplace() -> Box<dyn Contract<Empty>> {
        let smart_contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(smart_contract)
    }

    pub fn instantiate_smart_contract_test_nft_marketplace(
        app: &mut App,
        accepted_ibc_denominations: AcceptedDenominations,
        oracle_contract_address: String,
        reward_token: String
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_nft_marketplace());
        let init_msg = crate::msg::InstantiateMsg {
            contract_owner: OWNER.to_string(),
            accepted_ibc_denominations,
            marketplace_listing_fee_value: Uint128::new(6900000),
            marketplace_listing_fee_denom: "uhuahua".to_string(),
            oracle_contract_address,
            marketplace_pct_fees_decimal_string: "0.042".to_string(),
            reward_system: RewardSystem {
                reward_token_address: reward_token,
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
                        price_in_reward_tokens: Uint128::new(1_000u128),
                    },
                    VipPerk {
                        vip_level: VipLevel::Level2,
                        profile_background: true,
                        profile_nft_showcase: false,
                        profile_description: true,
                        profile_links: false,
                        marketplace_fees_discount: Decimal::from_str("0.05").unwrap(),
                        price_in_reward_tokens: Uint128::new(10_000u128)
                    },
                    VipPerk {
                        vip_level: VipLevel::Level3,
                        profile_background: true,
                        profile_nft_showcase: true,
                        profile_description: true,
                        profile_links: true,
                        marketplace_fees_discount: Decimal::from_str("0.1").unwrap(),
                        price_in_reward_tokens: Uint128::new(50_000u128)
                    }
                ],
            },
        };
        app.instantiate_contract(
            contract_code_id,
            Addr::unchecked(OWNER),
            &init_msg,
            &[],
            "chihuahua_nft_marketplace_code",
            None,
        ).unwrap()
    }

    // Execute Functions: NFT Marketplace
    pub fn execute_function_marketplace_test_update_config(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        configs_to_update: Vec<UpdateConfigEnum>
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::UpdateConfig { list_of_updates: configs_to_update };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_add_new_collection(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::AddNewCollection { nft_collection_address };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_sell_nft(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        sale_info: NftSale
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::SellNft { sale_info };
        app.execute_contract(info.sender, nft_marketplace_contract_addr.clone(), &msg, &info.funds)
    }

    pub fn execute_function_marketplace_test_offer(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        offer_info: NftOffer
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::Offer { offer: offer_info };
        app.execute_contract(info.sender, nft_marketplace_contract_addr.clone(), &msg, &info.funds)
    }

    pub fn execute_function_marketplace_test_cancel_offer(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::CancelOffer { nft_collection_address, token_id, additional_info: None };
        app.execute_contract(info.sender, nft_marketplace_contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_accept_offer(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId,
        from: String
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::AcceptOffer { nft_collection_address, token_id, from };
        app.execute_contract(info.sender, nft_marketplace_contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_cancel_sale(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::CancelSale { nft_collection_address, token_id, additional_info: None };
        app.execute_contract(info.sender, nft_marketplace_contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_update_sale(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        sale_info: NftSale
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::UpdateSale { sale_info };
        app.execute_contract(info.sender, nft_marketplace_contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_buy_nft(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::BuyNft { nft_collection_address, token_id, additional_info: None };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &info.funds
        )
    }

    pub fn execute_function_marketplace_test_enable_disable(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::UpdateConfig { list_of_updates: vec![UpdateConfigEnum::EnableDisable {}] };
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[]
        )
    }

    pub fn execute_function_marketplace_test_claim_marketplace_fees(
        app: &mut App,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::ClaimMarketplaceFees {};
        app.execute_contract(
            info.sender,
            nft_marketplace_contract_addr.clone(),
            &msg,
            &[]
        )
    }

    pub fn execute_function_marketplace_test_create_my_profile(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        profile: Option<Profile>
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::CreateMyProfile { profile, additional_info: None };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_update_my_profile(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        profile: Profile
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::UpdateMyProfile { profile };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_marketplace_test_level_up_my_profile(
        app: &mut App,
        info: MessageInfo,
        contract_addr: &Addr,
        cw20_denom: String,
        amount_of_tokens: Uint128,
        cw20_msg: ReceiveMsg
    ) -> AnyResult<AppResponse> {
        let msg = cw20::Cw20ExecuteMsg::Send {
            contract: contract_addr.to_string(),
            amount: amount_of_tokens.clone(),
            msg: to_binary(&cw20_msg).unwrap(),
        };
        app.execute_contract(info.sender, Addr::unchecked(cw20_denom.clone()), &msg, &info.funds)
    }

    // Query Functions: NFT Marketplace
    pub fn query_function_marketplace_test_get_config<T: Into<String>>(
        app: &App,
        contract_addr: T
    ) -> ConfigRewardGenStatsMsg {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetConfig {};
        let result: ConfigRewardGenStatsMsg =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    pub fn query_function_marketplace_test_get_marketplace_volume<T: Into<String>>(
        app: &App,
        contract_addr: T
    ) -> Uint128 {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetMarketplaceVolume {};
        let result: Uint128 =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    pub fn query_function_marketplace_test_get_nft_collection_volume<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress
    ) -> Uint128 {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetNftCollectionVolume { nft_collection_address };
        let result: Uint128 =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    pub fn query_function_marketplace_test_get_profile_info<T: Into<String>>(
        app: &App,
        contract_addr: T,
        address: String
    ) -> StdResult<Profile>  {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetProfileInfo { address };
        let result: StdResult<Profile> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_nft_for_sale_info<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress,
        token_id: TokenId
    ) -> StdResult<NftSale> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetNftForSaleInfo { nft_collection_address, token_id };
        let result: StdResult<NftSale> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_token_id_sale_history<T: Into<String>>(
        app: &App,
        contract_addr: T,
        collection_address: NftCollectionAddress,
        token_id: TokenId
    ) -> StdResult<Vec<TokenSaleHistory>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetTokenIdSaleHistory { nft_collection_address: collection_address, token_id };
        let result: StdResult<Vec<TokenSaleHistory>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_seller_all_nfts_for_sale<T: Into<String>>(
        app: &App,
        contract_addr: T,
        seller_address: String,
        start_after_token_id: Option<NftCollectionAddressTokenId>,
        output_length: Option<u32>
    ) -> StdResult<Vec<NftSale>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetSellerAllNftsForSale {
            seller_address,
            start_after_collection_token_id: start_after_token_id,
            output_length
        };
        let result: StdResult<Vec<NftSale>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_all_offers_address<T: Into<String>>(
        app: &App,
        contract_addr: T,
        offerer_address: String,
        start_after_token_id: Option<(NftCollectionAddress, TokenId)>,
        output_length: Option<u32>
    ) -> StdResult<Vec<NftOffer>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetAllOffersAddress {
            address: offerer_address,
            start_after: start_after_token_id,
            output_length
        };
        let result: StdResult<Vec<NftOffer>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_all_offers_token_id<T: Into<String>>(
        app: &App,
        contract_addr: T,
        token_id: TokenId,
        nft_collection_address: NftCollectionAddress,
        start_after: Option<(NftCollectionAddress, TokenId)>,
        output_length: Option<u32>
    ) -> StdResult<Vec<NftOffer>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetAllOffersTokenId {
            token_id: token_id.to_string(),
            nft_collection_address: nft_collection_address,
            output_length,
            start_after: start_after,
        };
        let result: StdResult<Vec<NftOffer>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_marketplace_info<T: Into<String>>(
        app: &App,
        contract_addr: T
    ) -> StdResult<Vec<MarketplaceStatsByDenom>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetMarketplaceInfo {};
        let result: StdResult<Vec<MarketplaceStatsByDenom>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_collection_all_nfts_for_sale<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress,
        start_after_token_id: Option<TokenId>,
        output_length: Option<u32>
    ) -> StdResult<Vec<NftSale>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetCollectionAllNftsForSale {
            nft_collection_address,
            start_after_token_id,
            output_length
        };
        let result: StdResult<Vec<NftSale>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_nft_collection_info<T: Into<String>>(
        app: &App,
        contract_addr: T,
        nft_collection_address: NftCollectionAddress
    ) -> StdResult<Vec<NftCollectionInfoByDenom>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetNftCollectionInfo { nft_collection_address };
        let result: StdResult<Vec<NftCollectionInfoByDenom>>  =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_function_marketplace_test_get_token_ids_by_collection<T: Into<String>>(
        app: &App,
        contract_addr: T,
        address: String,
        list_of_collections: Vec<NftCollectionAddress>
    ) -> StdResult<Vec<(NftCollectionAddress, Vec<String>)>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetTokenIdsByCollection { address, list_of_collections };
        let result: StdResult<Vec<(NftCollectionAddress, Vec<String>)>>  =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }
}