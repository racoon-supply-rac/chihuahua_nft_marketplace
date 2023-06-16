#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{Addr, Uint128};

    use nft_marketplace_utils::profile::{
        NftShowcase, Profile, ProfileMessage, ProfileMessages, ProfileUpdateAction, Socials,
    };
    use nft_marketplace_utils::reward_system::VipLevel;

    use crate::common::utils::constants::{OWNER, WALLET2, WALLET3};
    use crate::common::utils::utils_common::tests::instantiate_necessary_for_tests;
    use crate::common::utils::utils_marketplace_contract_test::tests::{
        marketplace_test_exec_create_my_profile, marketplace_test_exec_enable_disable,
        marketplace_test_exec_lvl_up_profile, marketplace_test_exec_send_message,
        marketplace_test_exec_update_my_profile, marketplace_test_query_get_profile_info,
    };
    use crate::common::utils::utils_nft_contract_test::tests::cw2981_multi_test_exec_mint;
    use chihuahua_nft_marketplace::msg::ReceiveMsg;

    #[test]
    fn unit_test_execute_function_create_update_upgrade_send_msg_profile_test() {
        let _deps = mock_dependencies();

        let (mut app, necessary) = instantiate_necessary_for_tests();
        let _native_huahua = necessary.native_huahua;
        let _native_atom = necessary.native_atom;
        let nft_marketplace_smart_contract_addr = necessary.nft_marketplace_smart_contract_addr;
        let _cw721_base_smart_contract_addr1 = necessary.cw2981_nft_contract_addr1;
        let _cw721_base_smart_contract_addr2 = necessary.cw2981_nft_contract_addr2;
        let _price_oracle_smart_contract_addr = necessary.price_oracle_contract_addr;
        let reward_token = necessary.cw20_reward_token;
        let _invalid_reward_token = necessary.cw20_invalid_reward_token;

        // Enable contract
        let info = mock_info(OWNER, &[]);
        let output_execute = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(output_execute.is_ok());

        // Create profile -> invalid additional info
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            Some("hello".to_string()),
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "CantUseAdditionalInfoIfNotContract".to_string()
        );

        // Mint NFTs
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_mint(
            &mut app,
            &Addr::unchecked(_cw721_base_smart_contract_addr1.clone()),
            info,
            "Token1".to_string(),
            WALLET3.to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
        let info = mock_info(OWNER, &[]);
        let execute_output = cw2981_multi_test_exec_mint(
            &mut app,
            &Addr::unchecked(_cw721_base_smart_contract_addr1.clone()),
            info,
            "Token2".to_string(),
            WALLET3.to_string(),
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Create profile -> can only error if sends additional_info
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            Some("additional".to_string()),
        );
        assert!(execute_output.is_err());

        // Valid creation
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the profile's info
        let execute_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(execute_output.address, Addr::unchecked(WALLET3));
        assert_eq!(execute_output.vip_level, Some(VipLevel::Level0));
        assert_eq!(execute_output.profile_nft_collection, None);
        assert_eq!(execute_output.profile_nft_token_id, None);
        assert_eq!(execute_output.background_nft_collection, None);
        assert_eq!(execute_output.background_nft_token_id, None);
        assert_eq!(execute_output.description, None);
        assert_eq!(execute_output.nft_showcase, None);
        assert_eq!(execute_output.links, None);
        assert_eq!(execute_output.buy_info, Some(vec![]));

        let mut wallet3_profile = Profile {
            address: "".to_string(),
            username: None,
            vip_level: None,
            profile_nft_collection: None,
            profile_nft_token_id: None,
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
        };

        // Update the profile: wrong sender
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_err());

        // Update the profile: sending invalid nft info
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.profile_nft_collection = Some("AA".to_string());
        wallet3_profile.profile_nft_token_id = Some("BB".to_string());
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_err());

        // Update the profile: sending valid nft info
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.profile_nft_collection = Some(_cw721_base_smart_contract_addr1.to_string());
        wallet3_profile.profile_nft_token_id = Some("Token1".to_string());
        wallet3_profile.description = Some("HELLO".to_string());
        wallet3_profile.profile_messages = Some(ProfileMessages {
            display_on_profile: true,
            messages: vec![ProfileMessage {
                from_address: "HELLO".to_string(),
                from_username: None,
                message: "".to_string(),
            }],
        });
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        // Cannot be updated until profile reaches the right level
        assert_eq!(
            profile_info_output.profile_nft_collection,
            Some(_cw721_base_smart_contract_addr1.to_string())
        );
        assert_eq!(
            profile_info_output.profile_nft_token_id,
            Some("Token1".to_string())
        );
        assert_eq!(profile_info_output.description, None);
        // Messages cannot be sent that way
        assert_eq!(
            profile_info_output.profile_messages,
            Some(ProfileMessages {
                display_on_profile: true,
                messages: vec![]
            })
        );

        // Change the display of p[rofiel back to false
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.profile_messages = Some(ProfileMessages {
            display_on_profile: false,
            messages: vec![ProfileMessage {
                from_address: "HELLO".to_string(),
                from_username: None,
                message: "".to_string(),
            }],
        });
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(
            profile_info_output.profile_messages,
            Some(ProfileMessages {
                display_on_profile: false,
                messages: vec![]
            })
        );

        // Remove the info
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.profile_nft_collection = Some(_cw721_base_smart_contract_addr1.to_string());
        wallet3_profile.profile_nft_token_id = Some("Token1".to_string());
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Remove,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        // Cannot be updated until profile reaches the right level
        assert_eq!(profile_info_output.profile_nft_collection, None);
        assert_eq!(profile_info_output.profile_nft_token_id, None);

        //Now upgrading the profile -> Invalid amount sent for level up
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_lvl_up_profile(
            &mut app,
            info,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(1u128),
            ReceiveMsg::LevelUpProfile {},
        );
        assert_eq!(
            execute_output
                .unwrap_err()
                .source()
                .unwrap()
                .source()
                .expect("_")
                .to_string(),
            "InvalidAmountReceivedForLevelUp".to_string()
        );

        //Now upgrading the profile -> Invalid denom sent for level up
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_lvl_up_profile(
            &mut app,
            info,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            _invalid_reward_token,
            Uint128::new(1_000u128),
            ReceiveMsg::LevelUpProfile {},
        );
        assert_eq!(
            execute_output
                .unwrap_err()
                .source()
                .unwrap()
                .source()
                .expect("_")
                .to_string(),
            "InvalidDenominationReceived".to_string()
        );

        // Valid upgrade
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_lvl_up_profile(
            &mut app,
            info,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(1_000u128),
            ReceiveMsg::LevelUpProfile {},
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Update the profile with description -> should be valid
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.background_nft_token_id = Some("Token1".to_string());
        wallet3_profile.background_nft_collection =
            Some(_cw721_base_smart_contract_addr1.to_string());
        wallet3_profile.description = Some("DESCRIPTION".to_string());
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(profile_info_output.vip_level, Some(VipLevel::Level1));
        assert_eq!(
            profile_info_output.background_nft_collection,
            Some(_cw721_base_smart_contract_addr1.to_string())
        );
        assert_eq!(
            profile_info_output.background_nft_token_id,
            Some("Token1".to_string())
        );
        assert_eq!(profile_info_output.description, None);

        // Make the next 2 level up
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_lvl_up_profile(
            &mut app,
            info,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(10_000u128),
            ReceiveMsg::LevelUpProfile {},
        );
        assert!(execute_output.is_ok(), "{}", false);
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_lvl_up_profile(
            &mut app,
            info,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(50_000u128),
            ReceiveMsg::LevelUpProfile {},
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Try another level up -> should error
        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_lvl_up_profile(
            &mut app,
            info,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token,
            Uint128::new(50_000u128),
            ReceiveMsg::LevelUpProfile {},
        );
        assert!(execute_output.is_err());

        // Update the profile with description -> should be valid
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.username = Some("BITCOIN".to_string());
        wallet3_profile.description = Some("DESCRIPTION".to_string());
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(
            profile_info_output.description,
            Some("DESCRIPTION".to_string())
        );
        assert_eq!(profile_info_output.username, Some("BITCOIN".to_string()));

        // Should be able to remove info -> Description
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.description = Some("REMOVED DESCRIPTION".to_string());
        wallet3_profile.username = None;
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Remove,
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(profile_info_output.description, None);

        // Check if the NFT Showcase works as expected -> add up to a maximum of 4
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.nft_showcase = Some(vec![
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
        ]);
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_err());

        // Check if the NFT Showcase works as expected -> add up to a maximum of 4
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.description = None;
        wallet3_profile.nft_showcase = Some(vec![
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
            NftShowcase {
                collection: _cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
            },
        ]);
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile.clone(),
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(
            profile_info_output.nft_showcase,
            Some(vec![
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                },
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                },
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                },
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                }
            ])
        );

        // If send only 1 to add in showcase, should pop and push
        let info = mock_info(WALLET3, &[]);
        wallet3_profile.description = None;
        wallet3_profile.nft_showcase = Some(vec![NftShowcase {
            collection: _cw721_base_smart_contract_addr1.to_string(),
            token_id: "Token2".to_string(),
        }]);
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            wallet3_profile,
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(
            profile_info_output.nft_showcase,
            Some(vec![
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                },
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                },
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1.to_string(),
                    token_id: "Token1".to_string()
                },
                NftShowcase {
                    collection: _cw721_base_smart_contract_addr1,
                    token_id: "Token2".to_string()
                }
            ])
        );

        // Now try to send messages
        // Need to check when sending to a username so we need to set a username and the other not
        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_send_message(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            WALLET3.to_string(),
            "Hello mom!".to_string(),
        );
        assert!(execute_output.is_ok(), "{}", false);

        let info = mock_info(OWNER, &[]);
        let execute_output = marketplace_test_exec_send_message(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            "BITCOIN".to_string(),
            "Hello dad!".to_string(),
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(profile_info_output.username, Some("BITCOIN".to_string()));
        assert_eq!(profile_info_output.vip_level, Some(VipLevel::Level3));
        assert_eq!(profile_info_output.description, None);
        assert_eq!(
            profile_info_output.profile_messages,
            Some(ProfileMessages {
                display_on_profile: false,
                messages: vec![
                    ProfileMessage {
                        from_address: OWNER.to_string(),
                        from_username: None,
                        message: "Hello mom!".to_string(),
                    },
                    ProfileMessage {
                        from_address: OWNER.to_string(),
                        from_username: None,
                        message: "Hello dad!".to_string(),
                    },
                ],
            })
        );

        let info = mock_info(WALLET3, &[]);
        let execute_output = marketplace_test_exec_send_message(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            OWNER.to_string(),
            "Hey son!".to_string(),
        );
        assert!(execute_output.is_ok(), "{}", false);

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            OWNER.to_string(),
        )
        .unwrap();
        assert_eq!(profile_info_output.username, None);
        assert_eq!(
            profile_info_output.profile_messages,
            Some(ProfileMessages {
                display_on_profile: true,
                messages: vec![ProfileMessage {
                    from_address: WALLET3.to_string(),
                    from_username: Some("BITCOIN".to_string()),
                    message: "Hey son!".to_string(),
                }],
            })
        );

        // THere's 1 msg alreayd and we are adding 11 of them so 2 msgs are popped out and
        //  it should remain from #3 and onward
        for i in 2..=12 {
            let mut msg_to_send: String = "Hey son!".to_string().to_owned();
            msg_to_send.push_str(&i.to_string());
            let info = mock_info(WALLET3, &[]);
            let execute_output = marketplace_test_exec_send_message(
                &mut app,
                nft_marketplace_smart_contract_addr.to_string(),
                info,
                OWNER.to_string(),
                msg_to_send,
            );
            assert!(execute_output.is_ok(), "{}", false);
        }

        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            OWNER.to_string(),
        )
        .unwrap();
        assert_eq!(profile_info_output.username, None);
        assert_eq!(
            profile_info_output.profile_messages.unwrap().messages[0],
            ProfileMessage {
                from_address: WALLET3.to_string(),
                from_username: Some("BITCOIN".to_string()),
                message: "Hey son!3".to_string(),
            }
        );

        // Check if one can query the profile using the username
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            "BITCOIN".to_string(),
        );
        assert!(profile_info_output.is_ok());

        // Update links individually
        let info = mock_info(WALLET3, &[]);
        let new_profile: Profile = Profile {
            address: "".to_string(),
            username: None,
            vip_level: None,
            profile_nft_collection: None,
            profile_nft_token_id: None,
            background_nft_collection: None,
            background_nft_token_id: None,
            description: None,
            nft_showcase: None,
            links: Some(Socials {
                twitter_link: Some("www.twitter_link.com".to_string()),
                discord_link: Some("www.google.com".to_string()),
                telegram_link: None,
                additional_social_link: None,
            }),
            profile_messages: None,
            number_of_trades: None,
            buy_info: None,
            sell_info: None,
            display_trade_info: Some(true),
        };
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            new_profile,
            ProfileUpdateAction::Add,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(
            profile_info_output.links.clone().unwrap().twitter_link,
            Some("www.twitter_link.com".to_string())
        );
        assert_eq!(
            profile_info_output.links.clone().unwrap().discord_link,
            Some("www.google.com".to_string())
        );
        assert_eq!(
            profile_info_output.links.clone().unwrap().telegram_link,
            None
        );
        assert_eq!(profile_info_output.display_trade_info, Some(true));

        // Update links individually -> remove
        let info = mock_info(WALLET3, &[]);
        let new_profile: Profile = Profile {
            address: "".to_string(),
            username: None,
            vip_level: None,
            profile_nft_collection: None,
            profile_nft_token_id: None,
            background_nft_collection: None,
            background_nft_token_id: None,
            description: None,
            nft_showcase: None,
            links: Some(Socials {
                twitter_link: None,
                discord_link: Some("www.google.com".to_string()),
                telegram_link: None,
                additional_social_link: None,
            }),
            profile_messages: None,
            number_of_trades: None,
            buy_info: None,
            sell_info: None,
            display_trade_info: Some(false),
        };
        let execute_output = marketplace_test_exec_update_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            new_profile,
            ProfileUpdateAction::Remove,
        );
        assert!(execute_output.is_ok(), "{}", false);
        // Validate the updated value
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET3.to_string(),
        )
        .unwrap();
        assert_eq!(
            profile_info_output.links.clone().unwrap().twitter_link,
            Some("www.twitter_link.com".to_string())
        );
        assert_eq!(
            profile_info_output.links.clone().unwrap().discord_link,
            None
        );
        assert_eq!(
            profile_info_output.links.clone().unwrap().telegram_link,
            None
        );
        assert_eq!(profile_info_output.display_trade_info, None);

        // Valid creation
        let info = mock_info(WALLET2, &[]);
        let execute_output = marketplace_test_exec_create_my_profile(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
            None,
        );
        assert!(execute_output.is_ok(), "{}", false);
        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr.to_string(),
            WALLET2.to_string(),
        )
        .unwrap();
        println!("{:#?}", profile_info_output.vip_level);

        // Enable contract
        let info = mock_info(OWNER, &[]);
        let output_execute = marketplace_test_exec_enable_disable(
            &mut app,
            nft_marketplace_smart_contract_addr.to_string(),
            info,
        );
        assert!(output_execute.is_ok());

        let profile_info_output = marketplace_test_query_get_profile_info(
            &app,
            nft_marketplace_smart_contract_addr,
            WALLET2.to_string(),
        )
        .unwrap();
        println!("{:#?}", profile_info_output.vip_level);
    }
}
