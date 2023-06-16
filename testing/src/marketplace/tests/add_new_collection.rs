#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{Addr, Uint128};

    use nft_marketplace_utils::nft_collection::{
        NftCollectionAddress, NftContractInfo, NftContractType,
    };

    use crate::common::utils::constants::{OWNER, WALLET2};
    use crate::common::utils::utils_common::tests::instantiate_necessary_for_tests;
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_add_new_collection, marketplace_test_exec_add_nft_code_id,
        marketplace_test_exec_enable_disable, marketplace_test_query_get_config,
        marketplace_test_query_get_nft_coll_info, marketplace_test_query_get_nft_coll_vol,
    };
    use crate::common::utils::utils_nft_contract_test::tests::{
        cw2981_multi_test_exec_mint, cw721_onchain_meta_test_exec_mint,
        instantiate_smart_contract_test_cw721_metadata_onchain,
    };

    #[test]
    fn test_marketplace_smart_contract_add_new_nft_collection_function() {
        // Validations:
        // - Only admin can add NFT collections for now [ok]
        // - Cannot add more than once [ok]
        // - Last Added collection state is updated [ok]
        // - Cannot add any type of contract [ok]
        // - It is only possible to add the Cw2981MultiRoyalties & Cw721OnChainMetadata (no royalties) type except for the two legacy collections [ok]
        // - NFT collection with no nfts cannot be added [ok]
        // - Currently checks if royalties are working on the given collection [ok]
        // - Validate all state changes are ok [ok]

        let _deps = mock_dependencies();
        let (mut app, necessary) = instantiate_necessary_for_tests();

        let _native_huahua = necessary.native_huahua;
        let native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr =
            necessary.nft_marketplace_smart_contract_addr.clone();
        let cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let _reward_token = necessary.cw20_reward_token;
        let _invalid_reward_token = necessary.cw20_invalid_reward_token;

        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
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

        // ----->>> Only admin can add NFT collections for now [ok]
        let info = mock_info(WALLET2, &[]);
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
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        // ----->>> NFT collection with no nfts cannot be added [ok]
        let info = mock_info(OWNER, &[]);
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
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "NoNftsMintedForThisContract".to_string()
        );

        // ----->>> Cannot add any type of contract [ok]
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(OWNER)),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidInput".to_string()
        );

        // ----->>> Cannot add any type of contract [ok]
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(_reward_token)),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert!(execute_output.is_err());

        // Mint NFT
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_mint(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            info,
            "Token1".to_string(),
            OWNER.to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // ----->>> It is only possible to add the Cw2981MultiRoyalties & Cw721OnChainMetadata (no royalties) type except for the two legacy collections [ok]
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MadHuahua,
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidNftCollection".to_string()
        );

        // Creating a Cw721OnChainMetadata
        let (no_royalty_nft_collection, _code_id_nft) =
            instantiate_smart_contract_test_cw721_metadata_onchain(&mut app);

        // Mint NFT
        let info = mock_info(OWNER, &[]);
        let execute_output = cw721_onchain_meta_test_exec_mint(
            &mut app,
            &no_royalty_nft_collection,
            info,
            "Token1".to_string(),
            OWNER.to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // ----->>> It is only possible to add the Cw2981MultiRoyalties & Cw721OnChainMetadata (no royalties) type except for the two legacy collections [ok]
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(no_royalty_nft_collection)),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::MarketplaceInfo,
            },
        );
        assert!(execute_output.is_err());

        // ----->>> It is only possible to add the Cw2981MultiRoyalties & Cw721OnChainMetadata (no royalties) type except for the two legacy collections [ok]
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::MarketplaceInfo,
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidNftCollection".to_string()
        );

        // Valid addition
        let info = mock_info(OWNER, &[]);
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
        assert!(execute_output.is_ok());

        // ----->>> Validate all state changes are ok [ok]
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            cw2981_base_smart_contract_addr1.to_string(),
        )
        .unwrap();

        // ----->>> Validate all state changes are ok [ok]
        assert_eq!(query_output.len(), 2);
        assert_eq!(
            query_output[0].nft_collection_address,
            cw2981_base_smart_contract_addr1
        );
        assert_eq!(query_output[0].denom, native_atom);
        assert_eq!(
            query_output[0].collection_name,
            "NftContractTest".to_string()
        );
        assert_eq!(query_output[0].nfts_for_sale, 0);
        assert_eq!(query_output[0].realized_trades, 0);
        assert_eq!(query_output[0].total_volume, Uint128::zero());
        assert_eq!(query_output[0].current_floor, Uint128::zero());

        // ----->>> Validate all state changes are ok [ok]
        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(
            query_output.general_stats.last_collection_added,
            cw2981_base_smart_contract_addr1
        );

        // ----->>> Validate all state changes are ok [ok]
        let query_output = marketplace_test_query_get_nft_coll_vol(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
        );
        assert_eq!(query_output, Uint128::zero());

        // ----->>> Cannot add more than once [ok]
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1)),
            NftContractInfo {
                code_id: necessary.cw2981_nft_code_id,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "NftCollectionAlreadyExists".to_string()
        );

        // ----->>> Validate all state changes are ok [ok]
        let query_output = marketplace_test_query_get_nft_coll_info(
            &app,
            nft_marketplace_smart_contract_addr,
            NftCollectionAddress::from(Addr::unchecked("a".to_string())),
        );
        assert_eq!(query_output, Ok(vec![]))
    }
}
