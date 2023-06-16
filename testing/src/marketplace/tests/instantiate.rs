#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
    use cw_multi_test::App;

    use general_utils::denominations::AcceptedDenominations;
    use nft_marketplace_utils::reward_system::{VipLevel, VipPerk};
    use price_oracle_utils::oracle::{OraclePrice, OraclePrices};

    use crate::common::utils::constants::{FEEDER, OWNER};
    use crate::common::utils::utils_common::tests::{
        instantiate_cw20, instantiate_necessary_for_tests, mock_app,
    };
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        default_init_msg_mkpc, instantiate_custom_smart_contract_test_nft_marketplace,
        marketplace_test_query_get_config, marketplace_test_query_get_mkpc_info,
        marketplace_test_query_get_mkpc_vol,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        instantiate_smart_contract_test_cw2981_multi, smart_contract_def_test_cw2981_multi,
    };
    use crate::common::utils::utils_price_oracle_contract_test::tests::{
        instantiate_smart_contract_test_price_oracle, oracle_test_exec_feed_prices,
    };

    #[test]
    fn test_marketplace_smart_contract_instantiate_contract() {
        let _deps = mock_dependencies();

        let (app, necessary) = instantiate_necessary_for_tests();
        let native_huahua = necessary.native_huahua;
        let native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let _cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let reward_token = necessary.cw20_reward_token;
        let _invalid_reward_token = necessary.cw20_invalid_reward_token;

        // Validation: Stats to init are ok and logic works as expected:
        // - Every element in the instantiate message [excluding reward system] is validated [ok]
        // - CONFIG -> every element that goes in the state is validated [ok]

        // - Every element in the Reward System message is validated [ok]
        // - REWARD_SYSTEM state [ok]

        // - MARKETPLACE_STATS_BY_DENOM [ok]
        // - NFT_COLLECTION_VOLUME_USDC [ok]
        // - GENERAL_STATS [ok]

        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        // CONFIG
        assert!(!query_output.contract_enabled);
        assert_eq!(query_output.contract_owner, OWNER.to_string());
        assert_eq!(
            query_output.accepted_ibc_denominations,
            AcceptedDenominations {
                list_of_denoms: vec![native_huahua.clone(), native_atom]
            }
        );
        assert_eq!(
            query_output.marketplace_pct_fees,
            Decimal::from_str("0.042").unwrap()
        );
        assert_eq!(
            query_output.marketplace_listing_fee_value,
            Uint128::new(6_900_000u128)
        );
        assert_eq!(query_output.marketplace_listing_fee_denom, native_huahua);
        assert_eq!(
            query_output.oracle_contract_address,
            price_oracle_smart_contract_addr
        );
        assert_eq!(
            query_output.reward_system.reward_token_address,
            reward_token
        );

        // REWARD_SYSTEM
        assert_eq!(
            query_output.reward_system.total_reward_tokens_distributed,
            Uint128::zero()
        );
        assert_eq!(
            query_output.reward_system.reward_token_per_1usdc_volume,
            Uint128::new(1_000_000u128)
        );
        assert_eq!(query_output.reward_system.vip_perks.len(), 3);

        // GENERAL_STATS
        assert_eq!(query_output.general_stats.top_10_volume_usdc, []);
        assert_eq!(
            query_output.general_stats.last_collection_added,
            "".to_string()
        );
        assert_eq!(
            query_output.general_stats.lowest_volume_usdc,
            Uint128::zero()
        );

        // MARKETPLACE_STATS_BY_DENOM
        let query_output =
            marketplace_test_query_get_mkpc_info(&app, nft_marketplace_smart_contract_addr.clone())
                .unwrap();
        assert_eq!(query_output.len(), 2);
        let huahua_output = query_output[0].clone();
        assert_eq!(huahua_output.denom, native_huahua);
        assert_eq!(huahua_output.nfts_for_sale, 0);
        assert_eq!(huahua_output.realized_sales_counter, 0);
        assert_eq!(huahua_output.total_realized_sales_volume, Uint128::zero());
        assert_eq!(huahua_output.total_marketplace_fees, Uint128::zero());
        assert_eq!(huahua_output.marketplace_fees_to_claim, Uint128::zero());

        // NFT_COLLECTION_VOLUME_USDC
        let query_output =
            marketplace_test_query_get_mkpc_vol(&app, nft_marketplace_smart_contract_addr);
        assert_eq!(query_output, Uint128::zero());
    }

    #[test]
    fn test_marketplace_smart_contract_invalid_instantiate_contract() {
        let _deps = mock_dependencies();

        // Prepare for the testing
        let mut app: App = mock_app();
        let native_huahua: String = "uhuahua".to_string();
        let native_atom: String =
            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string();
        let accepted_denoms: AcceptedDenominations = AcceptedDenominations {
            list_of_denoms: vec![native_huahua.clone(), native_atom.clone()],
        };
        let price_oracle_smart_contract_address: Addr =
            instantiate_smart_contract_test_price_oracle(&mut app, accepted_denoms.clone());

        let reward_token = instantiate_cw20(&mut app);
        let invalid_reward_token = instantiate_cw20(&mut app);

        let contract_code_id = app.store_code(smart_contract_def_test_cw2981_multi());
        let (cw2981_base_smart_contract_address1, _code_id_nft) =
            instantiate_smart_contract_test_cw2981_multi(&mut app, contract_code_id);

        let info = mock_info(FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_address.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: native_huahua,
                        value_usdc_6_decimals: Uint128::new(119u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: native_atom,
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1_u64),
            },
        );
        assert!(execute_output.is_ok());

        // Valid instantiation
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_ok());

        // Invalid Oracle Address
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(OWNER.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid Oracle Address
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(cw2981_base_smart_contract_address1.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid Reward Address
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(OWNER.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid Reward Address
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(OWNER.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid listing fee
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            Some(Uint128::zero()),
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid listing denom
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            Some(invalid_reward_token.to_string()),
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid listing denom
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            Some(reward_token.to_string()),
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid marketplace fees
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            Some("0.0".to_string()),
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid marketplace fees
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            Some("100.0".to_string()),
            None,
        );
        assert!(mkpc_addr.is_err());

        // Invalid reward system
        let mut init_msg = default_init_msg_mkpc();
        init_msg.reward_system.reward_token_address = reward_token.to_string();
        init_msg.reward_system.vip_perks = vec![];
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms.clone()),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            Some(init_msg.reward_system),
        );
        assert!(mkpc_addr.is_err());

        // Invalid reward system
        let mut init_msg = default_init_msg_mkpc();
        init_msg.reward_system.reward_token_address = reward_token.to_string();
        init_msg.reward_system.vip_perks[0] = VipPerk {
            vip_level: VipLevel::Level2,
            profile_background: false,
            profile_nft_showcase: false,
            profile_description: false,
            profile_links: false,
            marketplace_fees_discount: Default::default(),
            level_up_price_in_reward_tokens: Default::default(),
        };
        let mkpc_addr = instantiate_custom_smart_contract_test_nft_marketplace(
            &mut app,
            Some(accepted_denoms),
            Some(reward_token.to_string()),
            None,
            None,
            None,
            Some(price_oracle_smart_contract_address.to_string()),
            None,
            Some(init_msg.reward_system),
        );
        assert!(mkpc_addr.is_err());
    }
}
