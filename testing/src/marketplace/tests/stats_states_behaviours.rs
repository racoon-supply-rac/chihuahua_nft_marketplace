#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coins, Addr, Timestamp, Uint128};

    use cw2981_multiroyalties::Royalty;
    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftContractInfo, NftContractType,
    };
    use nft_marketplace_utils::nft_sale::NftSale;

    use crate::common::utils::constants::{OWNER, ROYALTY_RECEIVER1, ROYALTY_RECEIVER2, WALLET2};
    use crate::common::utils::utils_common::tests::instantiate_necessary_for_tests;
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_add_new_collection, marketplace_test_exec_add_nft_code_id,
        marketplace_test_exec_buy_nft, marketplace_test_exec_enable_disable,
        marketplace_test_exec_sell_nft, marketplace_test_query_get_nft_coll_info,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        cw2981_multi_test_exec_approve, cw2981_multi_test_exec_mint,
    };
    use crate::common::utils::utils_price_oracle_contract_test::tests::oracle_test_exec_update_config;

    #[test]
    fn test_marketplace_smart_contract_buy_nft_function_floor_denoms_collections() {
        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let native_huahua = necessary.native_huahua;
        let native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let cw721_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let cw721_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let _reward_token = necessary.cw20_reward_token;
        let _invalid_reward_token = necessary.cw20_invalid_reward_token;

        // Enable the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Need to add Denom in the Price Oracle
        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr),
            info,
            vec![oracle::msg::UpdateConfigEnum::AddDenoms {
                denoms: vec![native_huahua.clone(), native_atom.clone()],
            }],
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Mint several NFTs
        let info = mock_info(OWNER, &[]);
        for (token_number1, token_number2) in (1..22).enumerate() {
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&token_number1.to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&token_number2.to_string());
            let execute_output = cw2981_multi_test_exec_mint(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                info.clone(),
                token_id_iter_str.to_string(),
                OWNER.to_string(),
                Some(vec![
                    Royalty {
                        receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()),
                        royalty_permille_int: 11,
                    },
                    Royalty {
                        receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()),
                        royalty_permille_int: 15,
                    },
                ]),
            );
            assert!(execute_output.is_ok(), "{}", false);
            let execute_output = cw2981_multi_test_exec_mint(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
                info.clone(),
                token_id_iter_str2.to_string(),
                WALLET2.to_string(),
                Some(vec![
                    Royalty {
                        receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()),
                        royalty_permille_int: 11,
                    },
                    Royalty {
                        receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()),
                        royalty_permille_int: 15,
                    },
                ]),
            );
            assert!(execute_output.is_ok(), "{}", false);
        }

        // Add accepted code id and stuff
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_nft_code_id(
            &mut app,
            nft_marketplace_smart_contract_addr.clone(),
            vec![NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            }],
            info.clone(),
        );
        assert!(execute_output.is_ok());

        // Add the collection to the marketplace contract
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Add the collection to the marketplace contract
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Put some NFTs for sale to check if the floor is accurate by collection and denom
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let info2 = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        // Put up for sale several NFTs
        for (token_number1, token_number2) in (1..5).enumerate() {
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&token_number1.to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&token_number2.to_string());
            let execute_output = cw2981_multi_test_exec_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                token_id_iter_str.to_string(),
                1571797419u64 + 87000u64,
            );
            assert!(execute_output.is_ok(), "{}", false);
            let execute_output = marketplace_test_exec_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str.to_string(),
                    sale_price_value: Uint128::new(100_000_000u128 + token_number1 as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok(), "{}", false);

            // Wallet 2
            let execute_output = cw2981_multi_test_exec_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                token_id_iter_str2.to_string(),
                1571797419u64 + 87000u64,
            );
            assert!(execute_output.is_ok(), "{}", false);

            let execute_output = marketplace_test_exec_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                NftSale {
                    seller: WALLET2.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr2.to_string(),
                    token_id: token_id_iter_str2.to_string(),
                    sale_price_value: Uint128::new(200_000_000u128 + token_number1 as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok(), "{}", false);
        }

        for token_number1 in 6..8 {
            let token_number2 = token_number1;
            let mut token_id_iter_str: String = "Token".to_string();
            token_id_iter_str.push_str(&token_number1.to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string();
            token_id_iter_str2.push_str(&token_number2.to_string());
            let execute_output = cw2981_multi_test_exec_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                token_id_iter_str.to_string(),
                1571797419u64 + 87000u64,
            );
            assert!(execute_output.is_ok(), "{}", false);
            let execute_output = marketplace_test_exec_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str.to_string(),
                    sale_price_value: Uint128::new(10_000u128 + token_number1 as u128),
                    sale_price_denom: native_atom.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok(), "{}", false);

            // Wallet 2
            let execute_output = cw2981_multi_test_exec_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                token_id_iter_str2.to_string(),
                1571797419u64 + 87000u64,
            );
            assert!(execute_output.is_ok(), "{}", false);

            let execute_output = marketplace_test_exec_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                NftSale {
                    seller: WALLET2.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr2.to_string(),
                    token_id: token_id_iter_str2.to_string(),
                    sale_price_value: Uint128::new(20_000u128 + token_number1 as u128),
                    sale_price_denom: native_atom.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok(), "{}", false);
        }
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(10_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_000u128));
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(200_000_000u128));

        // Owner buy Tokenx1 from Wallet2
        let info = mock_info(OWNER, &coins(200_000_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr2.to_string(),
            "Tokenx1".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(200_000_001u128));

        // Test until there is nothing left
        let info = mock_info(OWNER, &coins(200_000_001u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr2.to_string(),
            "Tokenx2".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
        let info = mock_info(OWNER, &coins(200_000_002u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr2.to_string(),
            "Tokenx3".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
        let info = mock_info(OWNER, &coins(200_000_003u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr2.to_string(),
            "Tokenx4".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(
            query_output[1].current_floor,
            Uint128::new(1_000_000_000_000_000_000u128)
        );

        // Wallet 2
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info2.clone(),
            "Tokenx5".to_string(),
            1571797419u64 + 87000u64,
        );
        assert!(execute_output.is_ok(), "{}", false);

        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info2,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw721_base_smart_contract_addr2.to_string(),
                token_id: "Tokenx5".to_string(),
                sale_price_value: Uint128::new(20_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2)),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(20_000u128));

        // Need to test the General Stats
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token5".to_string(),
            1571797419u64 + 87000u64,
        );
        assert!(execute_output.is_ok(), "{}", false);

        let info = mock_info(OWNER, &coins(6_900_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token5".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        let info = mock_info(WALLET2, &coins(200_000_000u128, native_huahua));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr),
            info,
            cw721_base_smart_contract_addr1,
            "Token5".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
    }
}
