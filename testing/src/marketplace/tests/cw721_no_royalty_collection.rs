#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coins, Addr, Timestamp, Uint128};

    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftContractInfo, NftContractType,
    };
    use nft_marketplace_utils::nft_sale::NftSale;

    use crate::common::utils::constants::{OWNER, WALLET2};
    use crate::common::utils::utils_common::tests::{
        instantiate_necessary_for_tests, query_account_native_denom_balance,
    };
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_add_new_collection, marketplace_test_exec_add_nft_code_id,
        marketplace_test_exec_buy_nft, marketplace_test_exec_enable_disable,
        marketplace_test_exec_sell_nft,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        cw2981_multi_test_exec_approve, cw721_onchain_meta_test_exec_mint,
        instantiate_smart_contract_test_cw721_metadata_onchain,
    };

    #[test]
    fn unit_test_cw721_metadata_onchain_collection() {
        // There are two NFT collections on HUAHUA which one has no royalty and the other has custom royalties
        // The one with no royalty is tested below
        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let native_huahua = necessary.native_huahua;
        let _native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let _cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let _reward_token = necessary.reward_token;
        let _invalid_reward_token = necessary.invalid_reward_token;

        let (no_royalty_nft_collection, code_id_nft) =
            instantiate_smart_contract_test_cw721_metadata_onchain(&mut app);

        // Mint NFT
        let info = mock_info(OWNER, &[]);
        let execute_output = cw721_onchain_meta_test_exec_mint(
            &mut app,
            &Addr::unchecked(no_royalty_nft_collection.clone()),
            info,
            "Token1".to_string(),
            OWNER.to_string(),
            None,
        );
        assert!(execute_output.is_ok());

        // Add accepted code id and stuff
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_nft_code_id(
            &mut app,
            nft_marketplace_smart_contract_addr.clone(),
            vec![NftContractInfo {
                code_id: code_id_nft,
                nft_contract_type: NftContractType::Cw721OnChainMetadata,
            }],
            info,
        );
        assert!(execute_output.is_ok());

        // Add a collection
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(no_royalty_nft_collection.clone())),
            NftContractInfo {
                code_id: code_id_nft,
                nft_contract_type: NftContractType::Cw721OnChainMetadata,
            },
        );
        assert!(execute_output.is_ok());

        // Enable the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(execute_output.is_ok());

        // Add the sale
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_approve(
            &mut app,
            &Addr::unchecked(no_royalty_nft_collection.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            "Token1".to_string(),
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
                nft_collection_address: no_royalty_nft_collection.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64),
            },
        );
        assert!(execute_output.is_ok());

        // Prior to sale values -> 6900000 for listing
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(6900000));

        // Now Wallet 2 buys the NFT
        let info = mock_info(WALLET2, &coins(100_000u128, native_huahua.clone()));
        let execute_output = marketplace_test_exec_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(no_royalty_nft_collection),
            "Token1".to_string(),
            None,
        );
        assert!(execute_output.is_ok());

        // After the sale -> 6900000 + 0.042 * 100_000 for listing (and the seller should receive the rest because no royalties)
        let query_output = query_account_native_denom_balance(
            &app,
            nft_marketplace_smart_contract_addr,
            native_huahua.clone(),
        );
        assert_eq!(query_output.amount, Uint128::new(6900000 + 4_200));
        let query_output = query_account_native_denom_balance(&app, OWNER, native_huahua);
        assert_eq!(
            query_output.amount,
            Uint128::new(100_000_000_000_000_000_000u128 - 6900000 - 4_200 + 100_000)
        );
    }
}
