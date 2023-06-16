#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coin, coins, Addr, Timestamp, Uint128};
    use cw_multi_test::{BankSudo, SudoMsg};

    use cw2981_multiroyalties::Royalty;
    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftContractInfo, NftContractType,
    };
    use nft_marketplace_utils::nft_offer::NftOffer;
    use nft_marketplace_utils::nft_sale::NftSale;
    use nft_marketplace_utils::profile::{Profile, ProfileUpdateAction};
    use nft_marketplace_utils::reward_system::VipLevel;

    use crate::common::utils::constants::{
        OWNER, ROYALTY_RECEIVER1, ROYALTY_RECEIVER2, WALLET2, WALLET3, WALLET4,
    };
    use crate::common::utils::utils_common::tests::{
        execute_function_cw20_test_transfer_cw20, instantiate_necessary_for_tests,
        query_account_native_denom_balance,
    };
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_add_new_collection, marketplace_test_exec_add_nft_code_id,
        marketplace_test_exec_answer_offer, marketplace_test_exec_cancel_offer,
        marketplace_test_exec_enable_disable, marketplace_test_exec_offer,
        marketplace_test_exec_sell_nft, marketplace_test_exec_transfer_my_nft,
        marketplace_test_exec_update_my_profile, marketplace_test_query_get_all_offers_address,
        marketplace_test_query_get_all_offers_token, marketplace_test_query_get_profile_info,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        cw2981_multi_test_exec_approve, cw2981_multi_test_exec_mint,
        cw2981_multi_test_query_owner_of,
    };

    #[test]
    fn unit_test_offer_make_cancel_accept_and_transfer_my_nft() {
        // Validations
        // Offer: need to escrow the offer value
        // Offer: Only the owner of the Token ID can accept an offer he received
        // Offer: An offer remains on the NFT even if a sale is cancelled or the token is transferred
        // Offer answer: a new owner of a transferred token can decline offers

        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let native_huahua = necessary.native_huahua;
        let native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let reward_token = necessary.cw20_reward_token;
        let _invalid_reward_token = necessary.cw20_invalid_reward_token;

        let info = mock_info(OWNER, &[]);
        let execute_output = execute_function_cw20_test_transfer_cw20(
            &mut app,
            &Addr::unchecked(WALLET3),
            Addr::unchecked(reward_token),
            info,
            Uint128::new(1_000_000_000u128),
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Enable the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Mint NFTs to make trades offers, etc.
        let royalties = Some(vec![
            Royalty {
                receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()),
                royalty_permille_int: 11,
            },
            Royalty {
                receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()),
                royalty_permille_int: 15,
            },
        ]);
        for nft_to_mint in vec![
            ["Token1", OWNER],
            ["Token2", WALLET4],
            ["Token3", WALLET4],
            ["Token666", WALLET4],
            ["SALE_OFFER", WALLET4],
            ["NON_SALE_OFFER", WALLET4],
            ["DECLINE_SALE_OFFER", WALLET4],
            ["DECLINE_NON_SALE_OFFER", WALLET4],
            ["Token777", WALLET4],
        ] {
            let execute_output = cw2981_multi_test_exec_mint(
                &mut app,
                &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
                mock_info(OWNER, &[]),
                nft_to_mint[0].to_string(),
                nft_to_mint[1].to_string(),
                royalties.clone(),
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

        // Add the collection to the NFT marketplace
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Approve an NFT for a sale
        let info = mock_info(WALLET4, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token3".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Make the sale
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET4.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token3".to_string(),
                sale_price_value: Uint128::new(100_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Approve an NFT for a sale
        let info = mock_info(WALLET4, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token666".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Make the sale
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET4.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token666".to_string(),
                sale_price_value: Uint128::new(100_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Approve an NFT for a sale
        let info = mock_info(WALLET4, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "SALE_OFFER".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Make the sale
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET4.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "SALE_OFFER".to_string(),
                sale_price_value: Uint128::new(100_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Approve an NFT for a sale
        let info = mock_info(WALLET4, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "DECLINE_SALE_OFFER".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Make the sale
        let execute_output = marketplace_test_exec_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftSale {
                seller: WALLET4.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "DECLINE_SALE_OFFER".to_string(),
                sale_price_value: Uint128::new(100_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // TEST-> Make an offer on an NFT but with NO FUNDS -> ERROR
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(200_000_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidFundsForOffer".to_string()
        );

        // TEST-> Make an offer on your NFT -> ERROR
        let info = mock_info(OWNER, &coins(200_000_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token1".to_string(),
                offer_price_value: Uint128::new(200_000_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "CantOfferOnYourOwnNft".to_string()
        );

        // TEST-> Make an offer with NON-ACCEPTED DENOM -> ERROR
        let info = mock_info(OWNER, &coins(100_000u128, native_atom.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidOfferDenom".to_string()
        );

        // TEST-> Make an offer with a different amount being sent -> ERROR
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.to_string(),
                amount: vec![coin(
                    10_000_000_000_000_000_000_000_000_000_000u128,
                    native_huahua.clone(),
                )],
            }
        }))
        .unwrap();
        let info = mock_info(
            OWNER,
            &coins(
                10_000_000_000_000_000_000_000_000_000_000u128,
                native_huahua.clone(),
            ),
        );
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(200u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidOfferValueReceived".to_string()
        );

        // TEST-> Make an offer with an amount exceeding the cap -> ERROR
        let info = mock_info(
            OWNER,
            &coins(
                10_000_000_000_000_000_000_000_000_000_000u128,
                native_huahua.clone(),
            ),
        );
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(10_000_000_000_000_000_000_000_000_000_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidPrice".to_string()
        );

        // TEST-> Send too much on an offer -> ERROR
        let info = mock_info(OWNER, &coins(100_000_000_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(1_000_000_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidOfferValueReceived".to_string()
        );

        // TEST -> Invalid address offerer
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftOffer {
                offerer_address: "HELLO".to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Invalid input: address not normalized".to_string()
        );

        // TEST -> Invalid NFT address
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    "HELLO".to_string(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Invalid input: address not normalized".to_string()
        );

        // TEST -> Invalid collection -> not listed
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    _cw2981_base_smart_contract_addr2,
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "NftCollectionNotListed".to_string()
        );

        // TEST -> Invalid token id
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.to_string(),
                )),
                token_id: "Token2222".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_err());

        // Invalid expiration
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797219u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidExpirationTimeForTheOffer".to_string()
        );

        // TEST-> Valid offer
        let info = mock_info(OWNER, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(100_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // TEST-> Valid offer
        let info = mock_info(OWNER, &coins(50_000_000u128, native_atom.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token3".to_string(),
                offer_price_value: Uint128::new(50_000_000u128),
                offer_price_denom: native_atom,
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // TEST-> Make another offer with a different price on the same Token ID
        let info = mock_info(OWNER, &coins(110_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(110_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Cancel your offer before making a new one".to_string()
        );

        // Make an offer on an NFT - Second person
        let info = mock_info(WALLET2, &coins(110_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token2".to_string(),
                offer_price_value: Uint128::new(110_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // TEST-> Valid offer
        let info = mock_info(WALLET2, &coins(110_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "Token1".to_string(),
                offer_price_value: Uint128::new(110_000u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // TEST-> Valid offer
        let info = mock_info(OWNER, &coins(999_999_999u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "SALE_OFFER".to_string(),
                offer_price_value: Uint128::new(999_999_999u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // TEST-> Valid offer
        let info = mock_info(OWNER, &coins(999_999_999u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "NON_SALE_OFFER".to_string(),
                offer_price_value: Uint128::new(999_999_999u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(2027919998));
        let query_output = query_account_native_denom_balance(&app, OWNER, native_huahua.clone());
        assert_eq!(
            query_output.amount,
            Uint128::new(10000000000099999999997999900002)
        );

        // TEST-> Valid offer
        let info = mock_info(OWNER, &coins(999_999_999u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "DECLINE_SALE_OFFER".to_string(),
                offer_price_value: Uint128::new(999_999_999u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Should receive 999 999 999 to escrow
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(2027919998 + 999999999));
        let query_output = query_account_native_denom_balance(&app, OWNER, native_huahua.clone());
        assert_eq!(
            query_output.amount,
            Uint128::new(10000000000099999999997999900002 - 999999999)
        );

        // Make an offer on an NFT - 3rd person
        let info = mock_info(OWNER, &coins(999_999_999u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftOffer {
                offerer_address: info.sender.to_string(),
                nft_collection_address: NftCollectionAddress::from(Addr::unchecked(
                    cw2981_base_smart_contract_addr1.clone(),
                )),
                token_id: "DECLINE_NON_SALE_OFFER".to_string(),
                offer_price_value: Uint128::new(999_999_999u128),
                offer_price_denom: native_huahua.clone(),
                offer_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Should receive 999 999 999 to escrow
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(4027919996));

        // Validate the contract states -> should have 2 offers
        let output_query = marketplace_test_query_get_all_offers_token(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            "Token2".to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            None,
            None,
        );
        let output_query_address = output_query
            .unwrap()
            .iter()
            .map(|off| off.offerer_address.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            output_query_address,
            vec![
                "chihuahua1ifxixonpu9laitc4w6f17m2jfin3hchp4n85ov",
                "chihuahua1yk8khat1cmofok9yxnt6q06aueocv3pc8f32jo"
            ]
        );

        // Wallet 3 doesnt have any offers
        let output_query = marketplace_test_query_get_all_offers_address(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            WALLET3.to_string(),
            None,
            None,
        );
        assert_eq!(output_query.unwrap(), []);

        // Wallet 2 has 2 offers
        let output_query = marketplace_test_query_get_all_offers_address(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            WALLET2.to_string(),
            None,
            None,
        );
        let output_query_u = output_query.unwrap();
        assert_eq!(output_query_u.len(), 2);
        assert_eq!(output_query_u[0].offerer_address, Addr::unchecked(WALLET2));

        // Owner has 6 offers
        let output_query = marketplace_test_query_get_all_offers_address(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            OWNER.to_string(),
            None,
            None,
        );
        let output_query_u = output_query.unwrap();
        assert_eq!(output_query_u.len(), 6);

        // TEST -> Try to cancel an unexisting offer -> ERROR
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_offer::NftOffer not found".to_string()
        );

        // TEST -> Cancel an offer you dont own -> Not possible as the key is NFT collection + Token ID + Sender
        // Only person that can do it is the contract internally

        // TEST -> Cancel an offer
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_cancel_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token2".to_string(),
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Try to accept unexisting offer
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_answer_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string(),
            WALLET3.to_string(),
            true,
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_offer::NftOffer not found".to_string()
        );

        // Initial amount in escrow + fees
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(4027819996));

        // Accept offer - From an NFT not for sale -> Should work
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_answer_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "SALE_OFFER".to_string(),
            OWNER.to_string(),
            true,
            Some("THANKS BUDDY!".to_string()),
        );
        assert!(execute_output.is_ok(), "{}", false);

        // 4027819996 + 999 999 999 * 4.2% - 999 999 999 = 3069819996
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(3069819996));

        // Decline offer - From an NFT not for sale -> Should work
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_answer_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "DECLINE_SALE_OFFER".to_string(),
            OWNER.to_string(),
            false,
            Some("NEVER BUDDY!".to_string()),
        );
        assert!(execute_output.is_ok(), "{}", false);

        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        // Refunded 999 999 999
        assert_eq!(query_output.amount, Uint128::new(2069819997));

        // Decline offer - From an NFT for sale -> Should work
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_answer_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "DECLINE_NON_SALE_OFFER".to_string(),
            OWNER.to_string(),
            false,
            Some("NEVER BUDDY!!!!".to_string()),
        );
        assert!(execute_output.is_ok(), "{}", false);

        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua,
        );
        // Refunded 999 999 999
        assert_eq!(query_output.amount, Uint128::new(1069819998));

        // Accept offer - From an NFT for sale -> Should work
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_answer_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "NON_SALE_OFFER".to_string(),
            OWNER.to_string(),
            true,
            None,
        );
        // Need an approval before accepting such
        assert!(execute_output.is_err());

        // Make the approval and re-accept
        let info = mock_info(WALLET4, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "NON_SALE_OFFER".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok(), "{}", false);
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_answer_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "NON_SALE_OFFER".to_string(),
            OWNER.to_string(),
            true,
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // We also need to look into "declined offers"

        // We now look into transferring NFTs when they are for sale, not for sale, transfer from a bad account...
        // Transfer a non-existing NFT
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token3xx".to_string(),
            OWNER.to_string(),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Querier contract error: cw721_base::state::TokenInfo<core::option::Option<cw2981_multiroyalties::Metadata>> not found".to_string()
        );

        // Does not own the token
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token3".to_string(),
            WALLET4.to_string(),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "YouDontOwnThisTokenID".to_string()
        );

        // Bad contract
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(_invalid_reward_token)),
            "Token3".to_string(),
            OWNER.to_string(),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "NftCollectionNotListed".to_string()
        );
        // Create profile
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            Profile {
                address: WALLET4.to_string(),
                username: None,
                vip_level: None,
                profile_nft_collection: Some(cw2981_base_smart_contract_addr1.clone()),
                profile_nft_token_id: Some("Token777".to_string()),
                background_nft_collection: None,
                background_nft_token_id: None,
                description: None,
                nft_showcase: None,
                links: None,
                profile_messages: None,
                number_of_trades: None,
                buy_info: None,
                sell_info: None,
                display_trade_info: Some(false),
            },
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Validate the profile's info
        let execute_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET4.to_string(),
        )
        .unwrap();
        assert_eq!(execute_output.address, Addr::unchecked(WALLET4));
        assert_eq!(execute_output.vip_level, Some(VipLevel::Level0));
        assert_eq!(
            execute_output.profile_nft_collection,
            Some(cw2981_base_smart_contract_addr1.to_string())
        );
        assert_eq!(
            execute_output.profile_nft_token_id,
            Some("Token777".to_string())
        );
        // NFT that's not for sale but owned - but no approval
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token777".to_string(),
            OWNER.to_string(),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Querier contract error: Approval not found not found".to_string()
        );

        // NFT that's not for sale but owned - approved
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw2981_base_smart_contract_addr1.clone(),
            "Token777".to_string(),
        );
        assert_eq!(query_output.owner, WALLET4.to_string());
        let info = mock_info(WALLET4, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token777".to_string(),
            1571797419u64 + 600u64,
        );
        assert!(execute_output.is_ok(), "{}", false);

        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token777".to_string(),
            OWNER.to_string(),
        );
        assert!(execute_output.is_ok(), "{}", false);
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw2981_base_smart_contract_addr1.clone(),
            "Token777".to_string(),
        );
        assert_eq!(query_output.owner, OWNER.to_string());

        // Validate the profile's info
        let execute_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET4.to_string(),
        )
        .unwrap();
        assert_eq!(execute_output.address, Addr::unchecked(WALLET4));
        assert_eq!(execute_output.vip_level, Some(VipLevel::Level0));
        assert_eq!(execute_output.profile_nft_collection, None);
        assert_eq!(execute_output.profile_nft_token_id, None);

        // NFT that's for sale but owned - already approved
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw2981_base_smart_contract_addr1.clone(),
            "Token666".to_string(),
        );
        assert_eq!(query_output.owner, WALLET4.to_string());
        let info = mock_info(WALLET4, &[]);
        let execute_output = marketplace_test_exec_transfer_my_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token666".to_string(),
            OWNER.to_string(),
        );
        assert!(execute_output.is_ok(), "{}", false);
        let query_output = cw2981_multi_test_query_owner_of(
            &app,
            cw2981_base_smart_contract_addr1,
            "Token666".to_string(),
        );
        assert_eq!(query_output.owner, OWNER.to_string());
    }
}
