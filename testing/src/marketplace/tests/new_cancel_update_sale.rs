#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128};

    use cw2981_multiroyalties::Royalty;
    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftContractInfo, NftContractType,
    };
    use nft_marketplace_utils::nft_sale::NftSale;

    use crate::common::utils::constants::{OWNER, ROYALTY_RECEIVER1, ROYALTY_RECEIVER2, WALLET2};
    use crate::common::utils::utils_common::tests::instantiate_necessary_for_tests;
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_add_new_collection, marketplace_test_exec_add_nft_code_id,
        marketplace_test_exec_cancel_sale, marketplace_test_exec_enable_disable,
        marketplace_test_exec_rm_exp_sale, marketplace_test_exec_sell_nft,
        marketplace_test_exec_transfer_my_nft, marketplace_test_exec_update_sale,
        marketplace_test_query_get_coll_all_nfts_for_sale, marketplace_test_query_get_mkpc_info,
        marketplace_test_query_get_mkpc_vol, marketplace_test_query_get_nft_coll_info,
        marketplace_test_query_get_nft_coll_vol, marketplace_test_query_get_nft_for_sale_info,
        marketplace_test_query_get_profile_info, marketplace_test_query_get_seller_nfts_for_sale,
        marketplace_test_query_get_tokens_by_coll,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        cw2981_multi_test_exec_approve, cw2981_multi_test_exec_mint, cw2981_multi_test_exec_revoke,
        cw2981_multi_test_exec_transfer_nft,
    };

    #[test]
    fn test_marketplace_smart_contract_sell_cancel_update_nft_function() {
        // Sell NFT validations:
        // - Only owner of an NFT can sell an NFT [ok]
        // - Only valid collection and token id can be sold [ok]
        // - need to have an approval of the nft [ok]
        // - in case someone transfers an nft  (without the interface) we need to make it possible for the new owner to cancel/update a sale
        // - States need to be well updated: nfts for sale, marketplace stats by denom (nft for sale), [ok]
        // - need to send the right amount and denom for the listing [ok]

        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let native_huahua = necessary.native_huahua;
        let _native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let _reward_token = necessary.reward_token;
        let _invalid_reward_token = necessary.invalid_reward_token;

        // Enable the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info.clone(),
        );
        assert!(execute_output.is_ok());

        // Mint a few tokens
        for i in 1..=7 {
            let mut token_id_str: String = "Token".to_string().to_owned();
            token_id_str.push_str(&i.to_string());
            let execute_output = cw2981_multi_test_exec_mint(
                &mut app,
                &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
                info.clone(),
                token_id_str,
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
            assert!(execute_output.is_ok());
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

        // Add collection to trade it
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert!(execute_output.is_ok());

        // Mint into another contract
        let execute_output = cw2981_multi_test_exec_mint(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr2.clone()),
            info,
            "Token33".to_string(),
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
        assert!(execute_output.is_ok());

        // Collection has not been added to the marketplace
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(100_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "NftCollectionNotListed".to_string()
        );

        // The seller did not "Approve" the NFT Marketplace to transfer it
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Querier contract error: Approval not found not found".to_string()
        );

        // Need to approve the NFT to the marketplace before selling it
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token1".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok());

        // Sell the NFT - wrong denom for listing fee
        let info = mock_info(
            OWNER,
            &coins(
                6_900_000u128,
                "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
            ),
        );
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidDenomOrValueReceivedForListingFee".to_string()
        );

        // Sell the NFT - invalid amount for listing fee
        let info = mock_info(OWNER, &coins(690_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidDenomOrValueReceivedForListingFee".to_string()
        );

        // Sell the NFT - invalid seller - not owner
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidSellerInformation".to_string()
        );

        // Sell the NFT - invalid seller - not owner
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidSellerInformation".to_string()
        );

        // Sell the NFT - invalid expiration
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(10_001u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidExpirationTimeForTheSale".to_string()
        );

        // Sell the NFT - invalid expiration
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(10_0001u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 8700000000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidExpirationTimeForTheSale".to_string()
        );

        // Sell the NFT - invalid price
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(
                    100_000_000_000_000_000_000_000_000_000_000_000_000u128,
                ),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidPriceForTheSale".to_string()
        );

        // Sell the NFT - invalid price
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(0u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidPriceForTheSale".to_string()
        );

        // Sell the NFT - Token ID does not exists
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token11".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Querier contract error: cw721_base::state::TokenInfo<core::option::Option<cw2981_multiroyalties::Metadata>> not found".to_string()
        );

        // Sell the NFT - collection not listed
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_err());

        // Sell the NFT - invalid denom
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: "AAA".to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidSellerInformation".to_string()
        );

        // Sell NFT no error
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(99_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok());

        // Profile should exist
        let query_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            OWNER.to_string(),
        );
        assert!(query_output.is_ok());

        // Sell NFT - resell the same nft
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "SaleAlreadyExists".to_string()
        );

        // --- Add multiple for sale
        for i in 3..=5 {
            let mut token_id_str: String = "Token".to_string();
            token_id_str.push_str(&i.to_string());
            let info = mock_info(OWNER, &[]);
            let execute_output = cw2981_multi_test_exec_approve(
                &mut app,
                &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info,
                token_id_str.clone(),
                1571797419u64 + 600u64,
            );
            assert!(execute_output.is_ok());
            let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
            let execute_output = marketplace_test_exec_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info,
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                    token_id: token_id_str.clone(),
                    sale_price_value: Uint128::new(200_000u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
                },
            );
            assert!(execute_output.is_ok());
        }

        // Sell the NFT - invalid owner
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token2".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "YouDontOwnThisTokenID".to_string()
        );

        // Add accepted code id and stuff
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_nft_code_id(
            &mut app,
            nft_marketplace_smart_contract_addr.clone(),
            vec![NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            }],
            info,
        );
        assert!(execute_output.is_ok());

        // Add accepted code id and stuff
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_nft_code_id(
            &mut app,
            nft_marketplace_smart_contract_addr.clone(),
            vec![NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            }],
            info,
        );
        assert!(execute_output.is_ok());

        // Add collection to trade it
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr2.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert!(execute_output.is_ok());

        // List from other wallet
        let info = mock_info(WALLET2, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr2.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token33".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok());
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(400_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok());

        // GetNftForSaleInfo -> Validate the info of the added NFT
        let query_output = marketplace_test_query_get_nft_for_sale_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            NftCollectionAddress::from(Addr::unchecked(
                cw2981_base_smart_contract_addr1.to_string(),
            )),
            "Token1".to_string(),
        )
        .unwrap();
        assert_eq!(query_output.seller, OWNER);
        assert_eq!(
            query_output.nft_collection_address,
            cw2981_base_smart_contract_addr1
        );
        assert_eq!(query_output.token_id, "Token1".to_string());
        assert_eq!(query_output.sale_price_value, Uint128::new(99_000u128));
        assert_eq!(query_output.sale_price_denom, native_huahua);
        assert_eq!(
            query_output.sale_expiration,
            Timestamp::from_seconds(1571797419u64 + 87000u64)
        );

        // GetSellerAllNftsForSale -> Validate the NFTs for sale by seller
        let query_output = marketplace_test_query_get_seller_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            OWNER.to_string(),
            Option::None,
            Option::None,
        );
        let tokens_for_sale_owner: Vec<_> = query_output
            .unwrap()
            .iter()
            .map(|info| info.token_id.clone())
            .collect();
        assert_eq!(
            tokens_for_sale_owner,
            vec!["Token1", "Token3", "Token4", "Token5"]
        );

        // GetCollectionAllNftsForSale -> Validate the NFTs for sale by collection
        let query_output = marketplace_test_query_get_coll_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            None,
            None,
        );
        let tokens_for_sale_collection: Vec<_> = query_output
            .unwrap()
            .iter()
            .map(|info| info.token_id.clone())
            .collect();
        assert_eq!(
            tokens_for_sale_collection,
            vec!["Token1", "Token3", "Token4", "Token5"]
        );

        // GetMarketplaceInfo -> Validate marketplace stats by denom
        let query_output =
            marketplace_test_query_get_mkpc_info(&app, nft_marketplace_smart_contract_addr.clone());
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].nfts_for_sale, 5);
        assert_eq!(query_output[1].nfts_for_sale, 0);

        // GetAllTokensByCollAndIfForSale -> Check for sale
        let query_output = marketplace_test_query_get_tokens_by_coll(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            OWNER.to_string(),
            cw2981_base_smart_contract_addr1.clone(),
            Some(50),
        )
        .unwrap();
        for token_info_sale in query_output.iter() {
            let _ = !matches!(
                token_info_sale.token_id.as_str(),
                "Token2" | "Token6" | "Token7"
            );
        }

        // Sale cancellation
        // Try to cancel with additional_info
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
            Some(OWNER.to_string()),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "CantUseAdditionalInfoIfNotContract".to_string()
        );

        // Transfer an NFT (from marketplace) that is for sale and make the old owner cancel it -> should not work
        // Because the sale gets cancelled by transferring from the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token5".to_string(),
            WALLET2.to_string(),
        );
        assert!(execute_output.is_ok());
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token5".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );

        // Put it back for sale than make a classic transfer to see if the behaviour is ok
        let info = mock_info(WALLET2, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token5".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok());
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token5".to_string(),
                sale_price_value: Uint128::new(400_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok());

        let info = mock_info(WALLET2, &[]);
        let execute_output = cw2981_multi_test_exec_transfer_nft(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token5".to_string(),
            OWNER.to_string(),
        );
        assert!(execute_output.is_ok());

        // Old owner should not be able to cancel the sale
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token5".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "YouDontOwnThisTokenID".to_string()
        );

        // New owner should be able to cancel the sale -> even if he didnt trigger it
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token5".to_string(),
            None,
        );
        assert!(execute_output.is_ok());

        // Try to cancel a sale someone does not own
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "YouDontOwnThisTokenID".to_string()
        );

        // Transfer then try to cancel from old owner
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_transfer_nft(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token4".to_string(),
            WALLET2.to_string(),
        );
        assert!(execute_output.is_ok());
        // Old owner
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token4".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "YouDontOwnThisTokenID".to_string()
        );
        // New owner can cancel the sale -> in the UI, we cancel the sale then transfer.
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token4".to_string(),
            None,
        );
        assert!(execute_output.is_ok());

        // Try to cancel a sale that is not listed
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token7".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );

        // Cancel without revoking
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "RevokeYourApprovalBeforeCancellingSale".to_string()
        );

        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].nfts_for_sale, 2);
        assert_eq!(query_output[1].current_floor, Uint128::new(99_000u128));

        // Real cancel
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_revoke(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token1".to_string(),
        );
        assert!(execute_output.is_ok());
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
            None,
        );
        assert!(execute_output.is_ok());

        // Should be cancelled
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].nfts_for_sale, 1);
        // Cancelled the floor
        assert_eq!(query_output[1].current_floor, Uint128::new(200000u128));

        // Try to re-cancel
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );

        // Validate the NFTs for sale by collection
        let query_output = marketplace_test_query_get_coll_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            None,
            None,
        );
        let tokens_for_sale_collection: Vec<_> = query_output
            .unwrap()
            .iter()
            .map(|info| info.token_id.clone())
            .collect();
        // Transferred 5 earlier
        assert_eq!(tokens_for_sale_collection, vec!["Token3"]);

        // Check other states and make sure they didn't change
        let query_output =
            marketplace_test_query_get_mkpc_vol(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output, Uint128::zero());

        let query_output = marketplace_test_query_get_nft_coll_vol(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
        );
        assert_eq!(query_output, Uint128::zero());

        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].realized_trades, 0);
        assert_eq!(query_output[0].total_volume, Uint128::zero());

        // Update a sale: Sale does not exists
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token6".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );

        // Update a sale: Wrong sender
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2,
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "YouDontOwnThisTokenID".to_string()
        );

        // Update a sale: Wrong seller update
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token2".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_err());

        // Update a sale: Wrong seller update
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token2".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_err());

        // Update a sale: Valid info
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token3".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua,
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok());

        app.set_block(BlockInfo {
            height: 999999,
            time: Timestamp::from_seconds(1571797419u64 * 5),
            chain_id: "hello".to_string(),
        });

        // Validate the updated value
        let query_output = marketplace_test_query_get_nft_for_sale_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token3".to_string(),
        );
        assert_eq!(
            query_output.unwrap().sale_price_value,
            Uint128::new(200_000_000u128)
        );

        let execute_output = marketplace_test_exec_rm_exp_sale(
            &mut app,
            nft_marketplace_smart_contract_addr.clone(),
            mock_info(OWNER, &[]),
        );
        assert!(execute_output.is_ok());

        // Validate the updated value
        let query_output = marketplace_test_query_get_nft_for_sale_info(
            &app,
            nft_marketplace_smart_contract_addr,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1)),
            "Token3".to_string(),
        );
        assert!(query_output.unwrap().seller == *"");
    }
}
