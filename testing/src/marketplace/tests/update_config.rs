#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{Addr, Decimal, Uint128};

    use general_utils::denominations::AcceptedDenominations;
    use nft_marketplace_utils::reward_system::{RewardSystem, VipLevel, VipPerk};

    use crate::common::utils::constants::{OWNER, WALLET2, WALLET3};
    use crate::common::utils::utils_common::tests::instantiate_necessary_for_tests;
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_create_my_profile, marketplace_test_exec_enable_disable,
        marketplace_test_exec_update_config, marketplace_test_query_get_config,
    };
    use chihuahua_nft_marketplace::msg::UpdateConfigEnum;

    #[test]
    fn test_marketplace_smart_contract_enable_disable_function() {
        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let _native_huahua = necessary.native_huahua;
        let _native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let _cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let _reward_token = necessary.reward_token;
        let _invalid_reward_token = necessary.invalid_reward_token;

        // Validation:
        // - Contract should be disabled after instantiation [ok]
        // - Only the owner can enable/disable [ok]
        // - When disabled, should become enabled [ok]
        // - When disabled, users cant interact with the contract but admin functions work [ok]

        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            None,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "ContractDisabled".to_string()
        );

        // Enable the contract
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(execute_output.is_ok());
        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert!(query_output.contract_enabled);

        // Should work given the contract is enabled
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            None,
        );
        assert!(execute_output.is_ok());

        // Only the Admin can use this execute function
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr,
            info,
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );
    }

    #[test]
    fn test_marketplace_smart_contract_update_config_function() {
        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let native_huahua = necessary.native_huahua;
        let native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let _cw2981_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw2981_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let _reward_token = necessary.reward_token;
        let _invalid_reward_token = necessary.invalid_reward_token;

        // Validation:
        // - EnableDisable is tested in the previous test [ok]
        // - UpdateConfig should work even if the contract is disabled given its an admin function [ok]
        // - UpdateConfig should only be used by the contract owner [ok]
        // - Test: add and remove denom, update owner and update the reward system [ok]
        // - Test: multiple change at the same time [ok]

        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::AddDenoms {
                denoms: vec!["utoken".to_string(), "utoken2".to_string()],
            }],
        );
        assert!(execute_output.is_ok());

        // Check if the change is reflected
        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(
            query_output.accepted_ibc_denominations,
            AcceptedDenominations {
                list_of_denoms: vec![
                    native_huahua,
                    native_atom.clone(),
                    "utoken".to_string(),
                    "utoken2".to_string()
                ]
            }
        );

        // Admin function - only admin can execute
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::UpdateOwner {
                address: WALLET2.to_string(),
            }],
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        // Make another change and check if it works
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::UpdateOwner {
                address: WALLET2.to_string(),
            }],
        );
        assert!(execute_output.is_ok());
        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output.contract_owner, WALLET2.to_string());

        // Old owner should not work
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::RemoveDenoms {
                denoms: vec!["uhuahua".to_string()],
            }],
        );
        assert!(execute_output.is_err());

        // Test new admin and two changes
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            vec![
                UpdateConfigEnum::RemoveDenoms {
                    denoms: vec!["uhuahua".to_string()],
                },
                UpdateConfigEnum::UpdateOwner {
                    address: OWNER.to_string(),
                },
            ],
        );
        assert!(execute_output.is_ok());
        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(
            query_output.accepted_ibc_denominations,
            AcceptedDenominations {
                list_of_denoms: vec![native_atom, "utoken".to_string(), "utoken2".to_string()]
            }
        );
        assert_eq!(query_output.contract_owner, OWNER.to_string());

        // Then we modify the Vip perks
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::UpdateRewardSystem {
                reward_system: RewardSystem {
                    reward_token_address: "ModifiedToken".to_string(),
                    reward_token_per_1usdc_volume: Uint128::new(1u128),
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
                            marketplace_fees_discount: Decimal::from_str("0.07").unwrap(),
                            level_up_price_in_reward_tokens: Uint128::new(20_000u128),
                        },
                        VipPerk {
                            vip_level: VipLevel::Level3,
                            profile_background: true,
                            profile_nft_showcase: true,
                            profile_description: true,
                            profile_links: true,
                            marketplace_fees_discount: Decimal::from_str("0.15").unwrap(),
                            level_up_price_in_reward_tokens: Uint128::new(100_000u128),
                        },
                    ],
                },
            }],
        );
        assert!(execute_output.is_ok());
        let query_output =
            marketplace_test_query_get_config(&app, nft_marketplace_smart_contract_addr);
        assert_eq!(
            query_output.reward_system.reward_token_address,
            "ModifiedToken".to_string()
        );
        assert_eq!(
            query_output.reward_system.reward_token_per_1usdc_volume,
            Uint128::new(1u128)
        );
        assert_eq!(
            query_output.reward_system.vip_perks[2].level_up_price_in_reward_tokens,
            Uint128::new(100_000u128)
        );
    }
}
