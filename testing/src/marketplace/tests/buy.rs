#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128};

    use cw2981_multiroyalties::Royalty;
    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftContractInfo, NftContractType,
    };
    use nft_marketplace_utils::nft_sale::NftSale;
    use nft_marketplace_utils::profile::TradeInfo;

    use crate::common::utils::constants::{OWNER, ROYALTY_RECEIVER1, ROYALTY_RECEIVER2, WALLET2};
    use crate::common::utils::utils_common::tests::{
        instantiate_necessary_for_tests, query_account_cw20_balance,
        query_account_native_denom_balance,
    };
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_add_new_collection, marketplace_test_exec_add_nft_code_id,
        marketplace_test_exec_buy_nft, marketplace_test_exec_claim_mkpc_fees,
        marketplace_test_exec_create_my_profile, marketplace_test_exec_enable_disable,
        marketplace_test_exec_sell_nft, marketplace_test_query_get_coll_all_nfts_for_sale,
        marketplace_test_query_get_mkpc_info, marketplace_test_query_get_mkpc_vol,
        marketplace_test_query_get_nft_coll_info, marketplace_test_query_get_nft_coll_vol,
        marketplace_test_query_get_nft_for_sale_info, marketplace_test_query_get_profile_info,
        marketplace_test_query_get_token_sale_hist, marketplace_test_query_get_tokens_by_coll,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        cw2981_multi_test_exec_approve, cw2981_multi_test_exec_mint,
        cw2981_multi_test_query_owner_of, cw2981_multi_test_query_royalty_info,
    };

    #[test]
    fn test_marketplace_smart_contract_buy_nft_function() {
        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();

        let native_huahua = necessary.native_huahua;
        let native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let cw721_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let cw721_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let reward_token = necessary.cw20_reward_token;
        let _invalid_reward_token = necessary.cw20_invalid_reward_token;

        // - Make sure the info about the purchase is valid -> check if the item is for sale
        // - additional info can only be used by the contract
        // - Cant buy your own NFT
        // - States need to be updated
        // - The right values are sent to royalty, marketplace, nft is transf, and seller receive the right amount
        // - The right amount of reward token by USDC need to be sent
        // - States are correctly updated

        // Enable the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
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
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
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

        // Add the collections to the marketplace contract
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

        // Create profiles
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Put NFTs for sale
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let info2 = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        for (token_number1, token_number2) in (1..20).enumerate() {
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
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
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
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str2.to_string(),
                    sale_price_value: Uint128::new(100_000_000u128 + token_number1 as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok(), "{}", false);

            // Now we list for the 2nd contract
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
                    sale_price_value: Uint128::new(999_999_999u128 + token_number1 as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok(), "{}", false);
        }

        // Check for sale -> only up to token 19 should be for sale
        let query_output = marketplace_test_query_get_tokens_by_coll(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET2.to_string(),
            cw721_base_smart_contract_addr2,
            Some(50),
        )
        .unwrap();
        for token_info_sale in query_output.iter() {
            if token_info_sale.token_id == "Tokenx20" || token_info_sale.token_id == "Tokenx21" {
                assert!(!token_info_sale.for_sale)
            } else {
                assert!(token_info_sale.for_sale)
            }
        }

        // Query the royalty on a sale
        let royalty_tokenx19 = cw2981_multi_test_query_royalty_info(
            &app,
            cw721_base_smart_contract_addr1.clone(),
            "Tokenx19".to_string(),
            Uint128::new(100_000_018u128),
        );
        assert_eq!(royalty_tokenx19[0].address, ROYALTY_RECEIVER1.to_string());
        assert_eq!(
            royalty_tokenx19[0].royalty_amount,
            Uint128::new(1_100_000u128)
        );

        // Balance -> Prior to sales
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        // 57 NFTs for sale -> 6.9 * (38 + 19 -> 2nd coll) = 393.3
        assert_eq!(query_output.amount, Uint128::new(393300000));

        // Balances
        let query_output = query_account_native_denom_balance(&app, OWNER, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999868900000));
        let query_output = query_account_native_denom_balance(&app, WALLET2, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999737800000));

        // Royalties should be at 0
        let query_output =
            query_account_native_denom_balance(&app, ROYALTY_RECEIVER1, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(0));
        let query_output =
            query_account_native_denom_balance(&app, ROYALTY_RECEIVER2, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(0));

        // Owner
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw721_base_smart_contract_addr1.clone(),
            "Tokenx19".to_string(),
        );
        assert_eq!(query_output.owner, WALLET2.to_string());

        // ----->>> Invalid price
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Tokenx19".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidBuyerInformation".to_string()
        );
        // ----->>> Invalid token id
        let info = mock_info(OWNER, &coins(100_000_018u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Tokenx19__".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );
        // ----->>> Invalid denom
        let info = mock_info(OWNER, &coins(100_000_018u128, native_atom.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Tokenx19".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidBuyerInformation".to_string()
        );
        // ----->>> Cant use additional info if not contract
        let info = mock_info(OWNER, &coins(100_000_018u128, native_atom));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Tokenx19".to_string(),
            Some(OWNER.to_string()),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "CantUseAdditionalInfoIfNotContract".to_string()
        );

        // ----->>> Execute a valid purchase: Owner buys Tokenx19 from Wallet2
        let info = mock_info(OWNER, &coins(100_000_018u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Tokenx19".to_string(),
            None,
        );
        let response_output = execute_output.unwrap();
        response_output.events.iter().any(|i| {
            i.attributes.iter().any(|j| {
                match j.key.as_ref() {
                    "Sold by" => j.value == *WALLET2,
                    "Sold to" => j.value == *OWNER,
                    "Token ID" => j.value == *"Tokenx19",
                    "Collection" => j.value == cw721_base_smart_contract_addr1,
                    "Sold for" => j.value == *"100000018",
                    "Sold in denom" => j.value == native_huahua,
                    "Marketplace fees" => j.value == *"4200000", // 100000018 * 4.2% = 4_200_000
                    "Royalty Receiver chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa29" => {
                        j.value == royalty_tokenx19[0].royalty_amount.to_string()
                    }
                    "Royalty Receiver chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa39" => {
                        j.value == royalty_tokenx19[1].royalty_amount.to_string()
                    }
                    _ => false,
                }
            })
        });

        // Sale should not exist anymore
        let query_output = marketplace_test_query_get_nft_for_sale_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            NftCollectionAddress::from(Addr::unchecked(
                cw721_base_smart_contract_addr1.to_string(),
            )),
            "Tokenx19".to_string(),
        );
        assert!(query_output.unwrap().seller == *"");

        // After sale values in contract and royalty receivers
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        // 38 NFTs for sale -> 6.9 * 38 = 262.2
        // Then a sale of 100_000_018u128 at 4.2% ->
        assert_eq!(
            query_output.amount,
            Uint128::new(393300000u128 + 4_200_000u128)
        );

        // Royalties should be increased by the right value -> 1.25% and 1.05%
        let query_output =
            query_account_native_denom_balance(&app, ROYALTY_RECEIVER1, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(1_100_000u128));
        let query_output =
            query_account_native_denom_balance(&app, ROYALTY_RECEIVER2, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(1_500_000u128));

        // Owner of the token is now OWNER
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw721_base_smart_contract_addr1.clone(),
            "Tokenx19".to_string(),
        );
        assert_eq!(query_output.owner, OWNER.to_string());

        // Balances of natives
        let query_output = query_account_native_denom_balance(&app, OWNER, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999768899982));
        let query_output = query_account_native_denom_balance(&app, WALLET2, native_huahua.clone());
        // Which is 100000018-1100000-1500000-(100000018*0.042)-coll 2 fees
        assert_eq!(query_output.amount, Uint128::new(99999999999831000018));

        // Profiles have been updated
        let execute_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET2.to_string(),
        )
        .unwrap();
        assert_eq!(
            execute_output.sell_info,
            Some(vec![TradeInfo {
                denom: native_huahua.clone(),
                volume_value: Uint128::new(100000018u128)
            }])
        );
        assert_eq!(execute_output.buy_info, Some(vec![]));

        let execute_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            OWNER.to_string(),
        )
        .unwrap();
        assert_eq!(
            execute_output.buy_info,
            Some(vec![TradeInfo {
                denom: native_huahua.clone(),
                volume_value: Uint128::new(100000018u128)
            }])
        );
        assert_eq!(execute_output.sell_info, Some(vec![]));

        // We also check the marketplace info
        let query_output =
            marketplace_test_query_get_mkpc_info(&app, nft_marketplace_smart_contract_addr.clone());
        let huahua_output = query_output.unwrap();
        assert_eq!(huahua_output[0].denom, native_huahua);
        assert_eq!(huahua_output[0].nfts_for_sale, 56); // 38 - 1 + 19 = 56
        assert_eq!(huahua_output[0].realized_sales_counter, 1);
        assert_eq!(
            huahua_output[0].total_realized_sales_volume,
            Uint128::new(100_000_018u128)
        );
        assert_eq!(
            huahua_output[0].total_marketplace_fees,
            Uint128::new(397500000u128)
        ); // 57 * 6_900_000 + 4.2% on approx 100
        assert_eq!(
            huahua_output[0].marketplace_fees_to_claim,
            Uint128::new(397500000u128)
        );

        // Check if Tokenx19 is not for sale anymore (different approach)
        let query_output = marketplace_test_query_get_coll_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            None,
            Option::from(100),
        );
        let query_output = query_output.unwrap();
        let in_it = query_output
            .iter()
            .any(|nft_sale| nft_sale.token_id == "Tokenx19");
        assert!(!in_it);

        // USDC volume
        let query_output = marketplace_test_query_get_nft_coll_vol(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        // =100000018*120/1000000 = 12000
        assert_eq!(query_output, Uint128::new(12_000u128));

        // Marketplace USDC volume
        let query_output =
            marketplace_test_query_get_mkpc_vol(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output, Uint128::new(12_000u128));

        // Sale history of the token
        let query_output = marketplace_test_query_get_token_sale_hist(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            "Tokenx19".to_string(),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].seller, WALLET2.to_string());
        assert_eq!(query_output[0].buyer, OWNER.to_string());
        assert_eq!(
            query_output[0].nft_collection_address,
            Addr::unchecked(cw721_base_smart_contract_addr1.clone())
        );
        assert_eq!(query_output[0].token_id, "Tokenx19".to_string());
        assert_eq!(
            query_output[0].sale_price_value,
            Uint128::new(100_000_018u128)
        );
        assert_eq!(query_output[0].sale_price_denom, native_huahua);

        // Try to buy an expired sale
        let previous_block_info = app.block_info();
        app.set_block(BlockInfo {
            height: 999999,
            time: Timestamp::from_seconds(1571797419u64 * 2),
            chain_id: "hello".to_string(),
        });
        // Wallet2 tries to buy Token17 from Owner
        let info = mock_info(WALLET2, &coins(100_000_017u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Token17".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
        app.set_block(previous_block_info);
        // Balances should be the same
        let query_output = query_account_native_denom_balance(&app, OWNER, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999768899982));
        let query_output = query_account_native_denom_balance(&app, WALLET2, native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999831000018));
        // Owner of the token remains OWNER
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw721_base_smart_contract_addr1.clone(),
            "Token17".to_string(),
        );
        assert_eq!(query_output.owner, OWNER.to_string());

        // Wallet2 buy Token18 from Owner
        let info = mock_info(WALLET2, &coins(100_000_018u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Token18".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate NFTs for sale for the collection
        let query_output = marketplace_test_query_get_coll_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            None,
            Option::from(100),
        );
        // There were 19 x 2 tokens -> 38 and 2 were bought and 1 was cancelled -> 35
        assert_eq!(query_output.unwrap().len(), 35);

        // Check the stats
        let query_output =
            marketplace_test_query_get_mkpc_info(&app, nft_marketplace_smart_contract_addr.clone());
        let huahua_output = query_output.unwrap();
        assert_eq!(huahua_output[0].denom, native_huahua);
        assert_eq!(huahua_output[0].nfts_for_sale, 54);
        assert_eq!(huahua_output[0].realized_sales_counter, 2);
        assert_eq!(
            huahua_output[0].total_realized_sales_volume,
            Uint128::new(100_000_018u128 * 2u128)
        );
        assert_eq!(
            huahua_output[0].total_marketplace_fees,
            Uint128::new(401700000u128)
        );
        assert_eq!(
            huahua_output[0].marketplace_fees_to_claim,
            Uint128::new(401700000u128)
        );

        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].denom, native_huahua);
        assert_eq!(query_output[1].nfts_for_sale, 35);
        assert_eq!(query_output[1].realized_trades, 2);
        assert_eq!(
            query_output[1].total_volume,
            Uint128::new(100_000_018u128 * 2u128)
        );
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_000u128));

        let query_output = marketplace_test_query_get_token_sale_hist(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            "Token18".to_string(),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].seller, OWNER.to_string());
        assert_eq!(query_output[0].buyer, WALLET2.to_string());
        assert_eq!(
            query_output[0].nft_collection_address,
            Addr::unchecked(cw721_base_smart_contract_addr1.clone())
        );
        assert_eq!(query_output[0].token_id, "Token18".to_string());
        assert_eq!(
            query_output[0].sale_price_value,
            Uint128::new(100_000_018u128)
        );
        assert_eq!(query_output[0].sale_price_denom, native_huahua);

        let query_output =
            marketplace_test_query_get_mkpc_vol(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(
            query_output,
            Uint128::new(120u128 * 2u128 * 100_000_018u128 / 1_000_000u128)
        );

        let query_output = marketplace_test_query_get_nft_coll_vol(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        assert_eq!(
            query_output,
            Uint128::new(120u128 * 2u128 * 100_000_018u128 / 1_000_000u128)
        );

        // Validate: if floor is updated accurately
        // Check floor
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].denom, native_huahua);
        assert_eq!(query_output[1].nfts_for_sale, 35);
        assert_eq!(query_output[1].realized_trades, 2);
        assert_eq!(
            query_output[1].total_volume,
            Uint128::new(100_000_018u128 * 2u128)
        );
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_000u128));

        // Buy NFT
        let info = mock_info(WALLET2, &coins(100_000_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Token0".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Buy NFT
        let info = mock_info(OWNER, &coins(100_000_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            cw721_base_smart_contract_addr1.to_string(),
            "Tokenx1".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Check floor
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].denom, native_huahua);
        assert_eq!(query_output[1].nfts_for_sale, 33);
        assert_eq!(query_output[1].realized_trades, 4);
        assert_eq!(
            query_output[1].total_volume,
            Uint128::new(100_000_018u128 * 2u128 + 200_000_000u128)
        );
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_001u128));

        // Claim the fees
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_claim_mkpc_fees(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Check that the fees are resetted
        let query_output =
            marketplace_test_query_get_mkpc_info(&app, nft_marketplace_smart_contract_addr.clone());
        let query_output = query_output.unwrap();
        query_output
            .iter()
            .for_each(|by_denom| assert_eq!(by_denom.marketplace_fees_to_claim, Uint128::zero()));

        // Now we check the rewards for a bigger sale
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token20".to_string(),
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
                token_id: "Token20".to_string(),
                sale_price_value: Uint128::new(100_000_000_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Check if the received rewards are accurate as per the sale
        let output_query =
            query_account_cw20_balance(&app, reward_token.clone(), Addr::unchecked(OWNER));
        // initial value
        assert_eq!(output_query.unwrap().balance, Uint128::new(999500000000000));
        let output_query =
            query_account_cw20_balance(&app, reward_token.clone(), Addr::unchecked(WALLET2));
        // initial value
        assert_eq!(output_query.unwrap().balance, Uint128::new(100000000000));

        let info = mock_info(WALLET2, &coins(100_000_000_000_000u128, native_huahua));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr),
            info,
            cw721_base_smart_contract_addr1,
            "Token20".to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Check if the received rewards are accurate as per the sale
        let output_query =
            query_account_cw20_balance(&app, reward_token.clone(), Addr::unchecked(WALLET2));
        // 100_000_000_000_000u128 * 120u128 / 1_000_000u128 = 12000$USDC
        assert_eq!(
            output_query.unwrap().balance,
            Uint128::new(12_000_000_000u128 + 100000000000u128)
        );
        let output_query = query_account_cw20_balance(&app, reward_token, Addr::unchecked(OWNER));
        assert_eq!(
            output_query.unwrap().balance,
            Uint128::new(999500000000000 + 12_000_000_000u128)
        );
    }
}
