pub const OWNER: &str = "chihuahua1ifxixonpu9laitc4w6f17m2jfin3hchp4n85ov";
pub const FEEDER: &str = "chihuahua15xlwzvtblxvx9gem11p1pkotq1xo2a5452vf5d";
pub const WALLET2: &str = "chihuahua1yk8khat1cmofok9yxnt6q06aueocv3pc8f32jo";
pub const WALLET3: &str = "chihuahua1sdn8tr8d86boe49ifuk11ibsuncpnp8fx6u9tn";
pub const WALLET4: &str = "chihuahua1m8dlu9gd9ztamqehba0r7trfse6fs6k9nrzyx8";
pub const WALLET5: &str = "chihuahua128g3f8bp24mj4p9nqm9kqdo1urufo5wkqf65t3";
pub const ROYALTY_RECEIVER1: &str = "chihuahua1l6pm0qgul73h5m943moxfwf1mx534wjtqsi7y9";
pub const ROYALTY_RECEIVER2: &str = "chihuahua1gm6afmyfhldd2eentuskw3h0rq296nlt92vmxz";

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{Addr, coin, Coin, coins, Decimal, Empty, MessageInfo, StdResult, Timestamp, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cw20::{BalanceResponse, Cw20Coin};
    use cw_multi_test::{App, AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use cw2981_multiroyalties::Royalty;
    use general_utils::denominations::AcceptedDenominations;
    use nft_marketplace_utils::nft_collection::{NftCollectionAddress, TokenId};
    use nft_marketplace_utils::nft_offer::NftOffer;
    use nft_marketplace_utils::nft_sale::NftSale;
    use nft_marketplace_utils::profile::{Profile, TradeInfo};
    use nft_marketplace_utils::reward_system::{RewardSystem, VipLevel, VipPerk};
    use price_oracle_utils::oracle::{OraclePrice, OraclePrices};
    use crate::msg::{ReceiveMsg, UpdateConfigEnum};
    use crate::tests::utils_marketplace_contract_test::tests::{execute_function_marketplace_test_add_new_collection, execute_function_marketplace_test_buy_nft, execute_function_marketplace_test_cancel_sale, execute_function_marketplace_test_create_my_profile, execute_function_marketplace_test_enable_disable, execute_function_marketplace_test_sell_nft, execute_function_marketplace_test_update_config, execute_function_marketplace_test_update_my_profile, instantiate_smart_contract_test_nft_marketplace, query_function_marketplace_test_get_config, query_function_marketplace_test_get_marketplace_info, query_function_marketplace_test_get_marketplace_volume, query_function_marketplace_test_get_nft_collection_info, query_function_marketplace_test_get_nft_collection_volume, query_function_marketplace_test_get_collection_all_nfts_for_sale, query_function_marketplace_test_get_seller_all_nfts_for_sale, query_function_marketplace_test_get_profile_info, query_function_marketplace_test_get_nft_for_sale_info, query_function_marketplace_test_get_token_id_sale_history, execute_function_marketplace_test_update_sale, execute_function_marketplace_test_claim_marketplace_fees, execute_function_marketplace_test_level_up_my_profile, execute_function_marketplace_test_offer, execute_function_marketplace_test_cancel_offer, execute_function_marketplace_test_accept_offer};
    use crate::tests::utils_nft_contract_test::tests::{execute_function_cw2981_multi_test_approve, execute_function_cw2981_multi_test_mint, execute_function_cw2981_multi_test_revoke, instantiate_smart_contract_test_cw2981_multi, query_function_cw2981_multi_test_owner_of, query_function_cw2981_multi_test_royalty_info};
    use crate::tests::utils_price_oracle_contract_test::tests::{execute_function_price_oracle_test_feed_prices, execute_function_price_oracle_test_update_config, instantiate_smart_contract_test_price_oracle};
    use crate::tests::tests::{OWNER, ROYALTY_RECEIVER1, ROYALTY_RECEIVER2, WALLET2, WALLET3};

    fn mock_app() -> App {
        App::default()
    }

    fn instantiate_necessary_for_tests() -> (App, String, String, String, String, String, String, String, String) {
        let mut app: App = mock_app();
        let native_huahua: String = "uhuahua".to_string();
        let native_atom: String = "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string();
        let price_oracle_smart_contract_address: Addr = instantiate_smart_contract_test_price_oracle(
            &mut app,
            AcceptedDenominations {
                list_of_denoms: vec![native_huahua.clone(), native_atom.clone()]
            }
        );

        let reward_token = instantiate_cw20(&mut app);
        let invalid_reward_token = instantiate_cw20(&mut app);

        let nft_marketplace_smart_contract_address: Addr = instantiate_smart_contract_test_nft_marketplace(
            &mut app,
            AcceptedDenominations {
                list_of_denoms: vec![native_huahua.clone(), native_atom.clone()]
            },
            price_oracle_smart_contract_address.to_string(),
            reward_token.to_string()
        );
        let cw2981_base_smart_contract_address1: Addr = instantiate_smart_contract_test_cw2981_multi(&mut app);
        let cw2981_base_smart_contract_address2: Addr = instantiate_smart_contract_test_cw2981_multi(&mut app);
        return (
            app,
            native_huahua,
            native_atom,
            nft_marketplace_smart_contract_address.to_string(),
            cw2981_base_smart_contract_address1.to_string(),
            cw2981_base_smart_contract_address2.to_string(),
            price_oracle_smart_contract_address.clone().to_string(),
            reward_token.clone().to_string(),
            invalid_reward_token.clone().to_string()
        )
    }

    pub fn smart_contract_def_test_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    pub fn instantiate_cw20(app: &mut App) -> Addr {
        let cw20_id = app.store_code(smart_contract_def_test_cw20());
        let msg = cw20_base::msg::InstantiateMsg {
            name: String::from("RewardToken"),
            symbol: String::from("REW"),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: OWNER.to_string(),
                amount: Uint128::new(1_000_000_000_000_000u128)
            }],
            mint: None,
            marketing: None,
        };
        app.instantiate_contract(
            cw20_id,
            Addr::unchecked(OWNER),
            &msg,
            &[],
            "cw20",
            None
        )
            .unwrap()
    }

    // General Execute functions
    fn execute_function_cw20_test_transfer_cw20(
        app: &mut App,
        recipient: &Addr,
        cw20_addr: Addr,
        info: MessageInfo,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        let msg = cw20::Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount
        };
        app.execute_contract(info.sender, cw20_addr.clone(), &msg, &[])
    }

    // General Query functions
    fn query_account_cw20_balance<T: Into<String>>(
        app: &App,
        contract_addr: T,
        address: Addr,
    ) -> StdResult<BalanceResponse> {
        let msg = cw20_base::msg::QueryMsg::Balance { address: address.to_string() };
        let result =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    fn query_account_native_denom_balance<T: Into<String>>(
        app: &App,
        contract_addr: T,
        denom: String
    ) -> Coin {
        let result =
            app.wrap().query_balance(contract_addr, denom.clone()).unwrap();
        result
    }

    #[test]
    fn test_marketplace_smart_contract_instantiate_contract() {
        let _deps = mock_dependencies();
        let (
            app,
            native_huahua,
            native_atom,
            nft_marketplace_smart_contract_addr,
            _cw2981_base_smart_contract_addr1,
            _cw2981_base_smart_contract_addr2,
            price_oracle_smart_contract_addr,
            reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();

        // Validation: Are all contract states as expected:
        // - CONFIG [ok]
        // - REWARD_SYSTEM [ok]
        // - MARKETPLACE_STATS_BY_DENOM [ok]
        // - NFT_COLLECTION_VOLUME_USDC [ok]
        // - GENERAL_STATS [ok]
        let query_output = query_function_marketplace_test_get_config(
            &app,
            nft_marketplace_smart_contract_addr.clone()
        );
        assert_eq!(query_output.contract_enabled, false);
        assert_eq!(query_output.contract_owner, OWNER.to_string());
        assert_eq!(
            query_output.accepted_ibc_denominations,
            AcceptedDenominations { list_of_denoms: vec![native_huahua.clone(), native_atom.clone()] }
        );
        assert_eq!(query_output.marketplace_pct_fees, Decimal::from_str("0.042").unwrap());
        assert_eq!(query_output.marketplace_listing_fee_value, Uint128::new(6_900_000u128));
        assert_eq!(query_output.marketplace_listing_fee_denom, native_huahua.clone());
        assert_eq!(query_output.oracle_contract_address, price_oracle_smart_contract_addr);
        assert_eq!(query_output.reward_system.reward_token_address, reward_token);
        assert_eq!(query_output.reward_system.total_reward_tokens_distributed, Uint128::zero());
        assert_eq!(
            query_output.reward_system.reward_token_per_1usdc_volume,
            Uint128::new(1_000_000u128)
        );
        assert_eq!(query_output.reward_system.vip_perks.len(), 3);
        assert_eq!(query_output.general_stats.top_10_volume_usdc, []);
        assert_eq!(query_output.general_stats.last_collection_added, "".to_string());
        assert_eq!(query_output.general_stats.lowest_volume_usdc, Uint128::zero());

        let query_output = query_function_marketplace_test_get_marketplace_info(&app, nft_marketplace_smart_contract_addr.clone()).unwrap();
        assert_eq!(query_output.len(), 2);
        let huahua_output = query_output[0].clone();
        assert_eq!(huahua_output.denom, native_huahua.clone().to_string());
        assert_eq!(huahua_output.nfts_for_sale, 0);
        assert_eq!(huahua_output.realized_sales_counter, 0);
        assert_eq!(huahua_output.total_realized_sales_volume, Uint128::zero());
        assert_eq!(huahua_output.total_marketplace_fees, Uint128::zero());
        assert_eq!(huahua_output.marketplace_fees_to_claim, Uint128::zero());

        let query_output = query_function_marketplace_test_get_marketplace_volume(
            &app,
            nft_marketplace_smart_contract_addr.clone()
        );
        assert_eq!(query_output, Uint128::zero());
    }

    #[test]
    fn test_marketplace_smart_contract_enable_disable_function() {
        let _deps = mock_dependencies();
        let (
            mut app,
            _native_huahua,
            _native_atom,
            nft_marketplace_smart_contract_addr,
            _cw2981_base_smart_contract_addr1,
            _cw2981_base_smart_contract_addr2,
            _price_oracle_smart_contract_addr,
            _reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();

        // Validation:
        // - Contract should be disabled after instantiation [ok]
        // - Only the owner can enable/disable [ok]
        // - When disabled, should become enabled [ok]
        // - When disabled, users cant interact with the contract but admin functions work [ok]

        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_create_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            None
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "ContractDisabled".to_string());

        // Enable the contract
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info
        );
        assert_eq!(execute_output.is_err(), false);
        let query_output = query_function_marketplace_test_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output.contract_enabled, true);

        // Should work given the contract is enabled
        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_create_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            None
        );
        assert_eq!(execute_output.is_err(), false);

        // Only the Admin can use this execute function
        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "Unauthorized".to_string());
    }

    #[test]
    fn test_marketplace_smart_contract_update_config_function() {
        let _deps = mock_dependencies();
        let (
            mut app,
            native_huahua,
            native_atom,
            nft_marketplace_smart_contract_addr,
            _cw2981_base_smart_contract_addr1,
            _cw2981_base_smart_contract_addr2,
            _price_oracle_smart_contract_addr,
            _reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();

        // Validation:
        // - UpdateConfig should work even if the contract is disabled given its an admin function [ok]
        // - UpdateConfig should only be used by the contract owner [ok]
        // - Test: add and remove denom, update owner and update the reward system [ok]
        // - Test: multiple change at the same time [ok]

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::AddDenoms { denoms: vec!["utoken".to_string(), "utoken2".to_string()] }
            ]
        );
        assert_eq!(execute_output.is_err(), false);

        // Check if the change is reflected
        let query_output = query_function_marketplace_test_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(
            query_output.accepted_ibc_denominations,
            AcceptedDenominations { list_of_denoms: vec![native_huahua.clone(), native_atom.clone(),
                                                         "utoken".to_string(), "utoken2".to_string()] }
        );

        // Admin function - only admin can execute
        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_marketplace_test_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::UpdateOwner { address: WALLET2.to_string() }
            ]
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "Unauthorized".to_string());

        // Make another change and check if it works
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::UpdateOwner { address: WALLET2.to_string() }
            ]
        );
        assert_eq!(execute_output.is_err(), false);
        let query_output = query_function_marketplace_test_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output.contract_owner, WALLET2.to_string());

        // Old owner should not work
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            vec![UpdateConfigEnum::RemoveDenoms { denoms: vec!["uhuahua".to_string()] }]
        );
        assert_eq!(execute_output.is_err(), true);

        // Test new admin and two changes
        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_marketplace_test_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::RemoveDenoms { denoms: vec!["uhuahua".to_string()] },
                 UpdateConfigEnum::UpdateOwner { address: OWNER.to_string()}]
        );
        assert_eq!(execute_output.is_err(), false);
        let query_output = query_function_marketplace_test_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(
            query_output.accepted_ibc_denominations,
            AcceptedDenominations {
                list_of_denoms: vec![native_atom.clone(), "utoken".to_string(), "utoken2".to_string()] }
        );
        assert_eq!(query_output.contract_owner, OWNER.to_string());

        // Then we modify the Vip perks
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_config(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
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
                            price_in_reward_tokens: Uint128::new(1_000u128),
                        },
                        VipPerk {
                            vip_level: VipLevel::Level2,
                            profile_background: true,
                            profile_nft_showcase: false,
                            profile_description: true,
                            profile_links: false,
                            marketplace_fees_discount: Decimal::from_str("0.07").unwrap(),
                            price_in_reward_tokens: Uint128::new(20_000u128)
                        },
                        VipPerk {
                            vip_level: VipLevel::Level3,
                            profile_background: true,
                            profile_nft_showcase: true,
                            profile_description: true,
                            profile_links: true,
                            marketplace_fees_discount: Decimal::from_str("0.15").unwrap(),
                            price_in_reward_tokens: Uint128::new(100_000u128)
                        }
                    ],
                },
            }]
        );
        assert_eq!(execute_output.is_err(), false);
        let query_output = query_function_marketplace_test_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output.reward_system.reward_token_address, "ModifiedToken".to_string());
        assert_eq!(query_output.reward_system.reward_token_per_1usdc_volume, Uint128::new(1u128));
        assert_eq!(
            query_output.reward_system.vip_perks[2].price_in_reward_tokens,
            Uint128::new(100_000u128)
        );
    }

    #[test]
    fn test_marketplace_smart_contract_add_new_nft_collection_function() {

        // Validations:
        // - Only admin can add NFT collections [ok]
        // - NFT collection with no nft cannot be added [ok]
        // - Cannot add more than once [ok]
        // - Validate state changes are ok [ok]

        let _deps = mock_dependencies();
        let (
            mut app,
            _native_huahua,
            native_atom,
            nft_marketplace_smart_contract_addr,
            cw2981_base_smart_contract_addr1,
            _cw2981_base_smart_contract_addr2,
            _price_oracle_smart_contract_addr,
            _reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(execute_output.is_err(), false);

        // No one except Admin
        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone()))
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "Unauthorized".to_string());

        // Add a collection with no minted NFTs
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone()))
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "NoNftsMintedForThisContract".to_string()
        );

        // Mint NFT
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw2981_multi_test_mint(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            info.clone(),
            "Token1".to_string(),
            OWNER.to_string(),
            None
        );
        assert_eq!(execute_output.is_err(), false);

        // Add a collection
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone()))
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the information about the collection
        let query_output = query_function_marketplace_test_get_nft_collection_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(cw2981_base_smart_contract_addr1.to_string())
        ).unwrap();

        // Should have 2 denoms
        assert_eq!(query_output.len(), 2);
        assert_eq!(query_output[0].nft_collection_address, cw2981_base_smart_contract_addr1.to_string());
        assert_eq!(query_output[0].denom, native_atom.clone());
        assert_eq!(query_output[0].collection_name, "NftContractTest".to_string());
        assert_eq!(query_output[0].nfts_for_sale, 0);
        assert_eq!(query_output[0].realized_trades, 0);
        assert_eq!(query_output[0].total_volume, Uint128::zero());
        assert_eq!(query_output[0].current_floor, Uint128::zero());

        let query_output = query_function_marketplace_test_get_config(&app, nft_marketplace_smart_contract_addr.clone());
        assert_eq!(query_output.general_stats.last_collection_added, cw2981_base_smart_contract_addr1.clone().to_string());

        let query_output = query_function_marketplace_test_get_nft_collection_volume(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())));
        assert_eq!(query_output, Uint128::zero());

        // Adding the same collection
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone()))
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "NftCollectionAlreadyExists".to_string());

        // Adding a bad contract
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked("not_a_contract".to_string()))
        );
        assert_eq!(execute_output.is_err(), true);
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Querier contract error: cw_multi_test::wasm::ContractData not found".to_string()
        );

        // Trying to add another type of contract
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(nft_marketplace_smart_contract_addr.clone()))
        );
        assert_eq!(execute_output.is_err(), true);

        // Query a wrong contract should return nothing
        let query_output = query_function_marketplace_test_get_nft_collection_info(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked("a".to_string()))
        );
        assert_eq!(query_output, Ok(vec![]))
    }

    #[test]
    fn test_marketplace_smart_contract_sell_cancel_update_nft_function() {

        // Sell NFT validations:
        // - Only owner of an NFT can sell and NFT [ok]
        // - Only valid collection and token id can be sold [ok]
        // - need to have an approval of the nft [ok]
        // - States need to be well updated: nfts for sale, marketplace stats by denom (nft for sale), [ok]
        // - need to send the right amount for the listing [ok]

        let _deps = mock_dependencies();
        let (
            mut app,
            native_huahua,
            native_atom,
            nft_marketplace_smart_contract_addr,
            cw2981_base_smart_contract_addr1,
            cw2981_base_smart_contract_addr2,
            _price_oracle_smart_contract_addr,
            _reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();
        // Enable the contract
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint NFT
        let execute_output = execute_function_cw2981_multi_test_mint(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            info.clone(),
            "Token1".to_string(),
            OWNER.to_string(),
            Some(vec![
                Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
            ])
        );
        assert_eq!(execute_output.is_err(), false);

        // Add collection to trade it
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone()))
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint a few tokens
        for i in 2..=7 {
            let mut token_id_str: String = "Token".to_string().to_owned();
            token_id_str.push_str(&*i.to_string());
            let execute_output = execute_function_cw2981_multi_test_mint(
                &mut app,
                &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
                info.clone(),
                token_id_str,
                OWNER.to_string(),
                Some(vec![
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
                ])
            );
            assert_eq!(execute_output.is_err(), false);
        }

        // Mint into another contract
        let execute_output = execute_function_cw2981_multi_test_mint(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr2.clone()),
            info.clone(),
            "Token33".to_string(),
            WALLET2.to_string(),
            Some(vec![
                Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
            ])
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint some coins
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000u128, native_huahua.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: WALLET2.clone().to_string(),
                amount: vec![coin(100_000_000u128, native_huahua.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000u128, native_atom.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000u128, "random_denom".to_string())],
            }
        }))
            .unwrap();

        // Need to approve the NFT to the marketplace before selling it
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw2981_multi_test_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token1".to_string(),
            1571797419u64 + 600u64
        );
        assert_eq!(execute_output.is_err(), false);

        // Sell the NFT - wrong denom for listing fee
        let info = mock_info(OWNER, &coins(6_900_000u128, "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidDenomOrValueReceivedForListingFee".to_string()
        );

        // Sell the NFT - invalid amount for listing fee
        let info = mock_info(OWNER, &coins(6_900_00u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidDenomOrValueReceivedForListingFee".to_string()
        );

        // Sell the NFT - invalid seller - not owner
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "InvalidSellerInformation".to_string());

        // Sell the NFT - invalid seller - not owner
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "InvalidSellerInformation".to_string());

        // Sell the NFT - invalid expiration
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(10_001u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "InvalidExpirationTimeForTheSale".to_string());

        // Sell the NFT - invalid expiration
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(10_0001u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 8700000000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "InvalidExpirationTimeForTheSale".to_string());

        // Sell the NFT - invalid price
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000_000_000_000_000_000_000_000_000_000_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "InvalidPriceForTheSale".to_string());

        // Sell the NFT - invalid price
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(0u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "InvalidPriceForTheSale".to_string());

        // Sell the NFT - invalid NFT
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token11".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Generic error: Querier contract error: cw721_base::state::TokenInfo<core::option::Option<cw2981_multiroyalties::Metadata>> not found".to_string()
        );

        // Sell the NFT - collection not listed
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), true);

        // Sell NFT no error
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Sell NFT - resell the same nft
        let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token1".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "SaleAlreadyExists".to_string());

        // --- Add multiple for sale
        for i in 3..=5 {
            let mut token_id_str: String = "Token".to_string();
            token_id_str.push_str(&*i.to_string());
            let info = mock_info(OWNER, &vec![]);
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                token_id_str.clone(),
                1571797419u64 + 600u64
            );
            assert_eq!(execute_output.is_err(), false);
            let info = mock_info(OWNER, &coins(6_900_000u128, "uhuahua".to_string()));
            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                    token_id: token_id_str.clone(),
                    sale_price_value: Uint128::new(200_000u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);
        }

        // Sell the NFT - invalid owner
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token2".to_string(),
                sale_price_value: Uint128::new(100_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.unwrap_err().source().unwrap().to_string(), "YouDontOwnThisTokenID".to_string());

        // Add collection to trade it
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr2.clone()))
        );
        assert_eq!(execute_output.is_err(), false);


        // List from other wallet
        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_cw2981_multi_test_approve(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr2.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token33".to_string(),
            1571797419u64 + 600u64
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(WALLET2, &coins(6_900_000u128, "uhuahua".to_string()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(400_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the info of the added NFT
        let query_output = query_function_marketplace_test_get_nft_for_sale_info(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone().to_string())),
            "Token1".to_string()
        ).unwrap();
        assert_eq!(query_output.seller, OWNER);
        assert_eq!(query_output.nft_collection_address, cw2981_base_smart_contract_addr1.clone().to_string());
        assert_eq!(query_output.token_id, "Token1".to_string());
        assert_eq!(query_output.sale_price_value, Uint128::new(100_000u128));
        assert_eq!(query_output.sale_price_denom, native_huahua.clone().to_string());
        assert_eq!(query_output.sale_expiration, Timestamp::from_seconds(1571797419u64 + 87000u64));

        // Validate the NFTs for sale by seller
        let query_output = query_function_marketplace_test_get_seller_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            OWNER.to_string(),
            Option::None, Option::None
        );
        let tokens_for_sale_owner: Vec<_> = query_output.unwrap().iter().map(|info| info.token_id.clone()).collect();
        assert_eq!(tokens_for_sale_owner, vec!["Token1", "Token3", "Token4", "Token5"]);

        // Validate the NFTs for sale by collection
        let query_output = query_function_marketplace_test_get_collection_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            None, None
        );
        let tokens_for_sale_collection: Vec<_> = query_output.unwrap().iter().map(|info| info.token_id.clone()).collect();
        assert_eq!(tokens_for_sale_collection, vec!["Token1", "Token3", "Token4", "Token5"]);

        // Validate marketplace stats
        let query_output = query_function_marketplace_test_get_marketplace_info(&app, nft_marketplace_smart_contract_addr.clone());
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].nfts_for_sale, 5);
        assert_eq!(query_output[1].nfts_for_sale, 0);

        // Sale cancellation
        // Try to cancel a sale someone does not own
        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_marketplace_test_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string()
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "CantCancelASaleYouDontOwn".to_string()
        );

        // Try to cancel a sale that is not listed
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token7".to_string()
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );

        // Cancel without revoking
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string()
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "RevokeYourApprovalBeforeCancellingSale".to_string()
        );

        // Real cancel
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw2981_multi_test_revoke(
            &mut app,
            &Addr::unchecked(cw2981_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token1".to_string()
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string()
        );
        assert_eq!(execute_output.is_err(), false);

        // Try to re-cancel
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_cancel_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info,
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            "Token1".to_string()
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "nft_marketplace_utils::nft_sale::NftSale not found".to_string()
        );
        // Validate the NFTs for sale by collection
        let query_output = query_function_marketplace_test_get_collection_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone())),
            None, None
        );
        let tokens_for_sale_collection: Vec<_> = query_output.unwrap().iter().map(|info| info.token_id.clone()).collect();
        assert_eq!(tokens_for_sale_collection, vec!["Token3", "Token4", "Token5"]);

        // Update a sale: Wrong sender
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr2.to_string(),
                token_id: "Token33".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "CantCancelASaleYouDontOwn".to_string()
        );

        // Update a sale: Wrong seller update
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token2".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), true);

        // Update a sale: Wrong seller update
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token2".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), true);

        // Update a sale: Valid info
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_sale(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw2981_base_smart_contract_addr1.to_string(),
                token_id: "Token3".to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the updated value
        let query_output = query_function_marketplace_test_get_nft_for_sale_info(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw2981_base_smart_contract_addr1.clone().to_string())),
            "Token3".to_string()
        );
        assert_eq!(query_output.unwrap().sale_price_value, Uint128::new(200_000_000u128));

    }

    #[test]
    fn test_marketplace_smart_contract_buy_nft_function() {
        let _deps = mock_dependencies();
        let (
            mut app,
            native_huahua,
            native_atom,
            nft_marketplace_smart_contract_addr,
            cw721_base_smart_contract_addr1,
            _cw721_base_smart_contract_addr2,
            price_oracle_smart_contract_addr,
            reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();

        // - Cant buy own NFT
        // - States need to be updated
        // - The right values are sent to royalty, marketplace, nft transfered, and seller receive the right amount

        // Enable the contract
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(execute_output.is_err(), false);

        // Need to add Denom in the Price Oracle
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                crypto_assets_price_oracle::msg::UpdateConfigEnum::AddDenoms {
                    denoms: vec![native_huahua.clone(), native_atom.clone()]
                }]
        );
        assert_eq!(execute_output.is_err(), false);

        // Price Oracle: Add prices
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: native_huahua.clone().to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: native_atom.clone().to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1 as u64),
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint several NFTs
        let info = mock_info(OWNER, &vec![]);
        for (token_number1, token_number2) in (1..22).enumerate() {
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&*token_number1.to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&*token_number2.to_string());
            let execute_output = execute_function_cw2981_multi_test_mint(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                info.clone(),
                token_id_iter_str.to_string(),
                OWNER.to_string(),
                Some(vec![
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
                ])
            );
            assert_eq!(execute_output.is_err(), false);
            let execute_output = execute_function_cw2981_multi_test_mint(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                info.clone(),
                token_id_iter_str2.to_string(),
                WALLET2.to_string(),
                Some(vec![
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
                ])
            );
            assert_eq!(execute_output.is_err(), false);
        }

        // Add the collection to the marketplace contract
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint some coins for purchases
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_huahua.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_atom.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: WALLET2.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_huahua.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: WALLET2.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_atom.clone())],
            }
        }))
            .unwrap();

        // Send Reward tokens to the contract to distribute
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw20_test_transfer_cw20(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            Addr::unchecked(reward_token.clone()),
            info.clone(),
            Uint128::new(100_000_000_000u128)
        );
        assert_eq!(execute_output.is_err(), false);

        let info = mock_info(OWNER, &&coins(6_900_000u128, "uhuahua".to_string()));
        let info2 = mock_info(WALLET2, &&coins(6_900_000u128, "uhuahua".to_string()));
        // Put up for sale several NFTs
        for (token_number1, token_number2) in (1..20).enumerate() {
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&*token_number1.clone().to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&*token_number2.clone().to_string());
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                token_id_iter_str.clone().to_string(),
                1571797419u64 + 87000u64
            );
            assert_eq!(execute_output.is_err(), false);
            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str.clone().to_string(),
                    sale_price_value: Uint128::new(100_000_000u128 + token_number1.clone() as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);

            // Wallet 2
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                token_id_iter_str2.clone().to_string(),
                1571797419u64 + 87000u64
            );
            assert_eq!(execute_output.is_err(), false);

            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                NftSale {
                    seller: WALLET2.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str2.clone().to_string(),
                    sale_price_value: Uint128::new(100_000_000u128 + token_number1.clone() as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);
        }

        // Query the cw2981
        let royalty_tokenx19 = query_function_cw2981_multi_test_royalty_info(
            &app,
            cw721_base_smart_contract_addr1.clone(),
            "Tokenx19".to_string(),
            Uint128::new(100_000_018u128)
        );
        assert_eq!(royalty_tokenx19[0].address, ROYALTY_RECEIVER1.to_string());
        assert_eq!(royalty_tokenx19[0].royalty_amount, Uint128::new(1_100_000u128));

        // Prior to sale values in contract and royalty receivers
        let query_output = query_account_native_denom_balance(&app, nft_marketplace_smart_contract_addr.clone(), native_huahua.clone());
        // 38 NFTs for sale -> 6.9 * 38 = 262.2
        assert_eq!(query_output.amount, Uint128::new(262200000));

        // Balances
        let query_output = query_account_native_denom_balance(&app, OWNER.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999868900000));
        let query_output = query_account_native_denom_balance(&app, WALLET2.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999868900000));

        // Royalties should be at 0
        let query_output = query_account_native_denom_balance(&app, ROYALTY_RECEIVER1.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(0));
        let query_output = query_account_native_denom_balance(&app, ROYALTY_RECEIVER2.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(0));
        // Owner
        let query_output = query_function_cw2981_multi_test_owner_of(&app, cw721_base_smart_contract_addr1.clone(), "Tokenx19".to_string());
        assert_eq!(query_output.owner, WALLET2.to_string());

        // Owner buy Tokenx19 from Wallet2
        let info = mock_info(OWNER, &coins(100_000_018u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr1.to_string()),
            "Tokenx19".to_string(),
        );
        let response_output = execute_output.unwrap();
        response_output.events.iter().any(|i| {
            i.attributes.iter().any(|j| {
                match j.key.as_ref() {
                    "Sold by" => j.value == WALLET2.to_string(),
                    "Sold to" => j.value == OWNER.to_string(),
                    "Token ID" => j.value == "Tokenx19".to_string(),
                    "Collection" => j.value == cw721_base_smart_contract_addr1.clone().to_string(),
                    "Sold for" => j.value == "100000018".to_string(),
                    "Sold in denom" => j.value == native_huahua.clone().to_string(),
                    "Marketplace fees" => j.value == "4200000".to_string(), // 100000018 * 4.2% = 4_200_000
                    "Royalty Receiver chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa29" => j.value == royalty_tokenx19[0].royalty_amount.to_string(),
                    "Royalty Receiver chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa39" => j.value == royalty_tokenx19[1].royalty_amount.to_string(),
                    _ => false,
                }
            })
        });
        // After sale values in contract and royalty receivers
        let query_output = query_account_native_denom_balance(&app, nft_marketplace_smart_contract_addr.clone(), native_huahua.clone());
        // 38 NFTs for sale -> 6.9 * 38 = 262.2
        // Then a sale of 100_000_018u128 at 4.2% ->
        assert_eq!(query_output.amount, Uint128::new(262200000u128 + 4_200_000u128));

        // Royalties should be increased by the right value -> 1.25% and 1.05%
        let query_output = query_account_native_denom_balance(&app, ROYALTY_RECEIVER1.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(1_100_000u128));
        let query_output = query_account_native_denom_balance(&app, ROYALTY_RECEIVER2.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(1_500_000u128));

        // Owner
        let query_output = query_function_cw2981_multi_test_owner_of(&app, cw721_base_smart_contract_addr1.clone(), "Tokenx19".to_string());
        assert_eq!(query_output.owner, OWNER.to_string());

        // Balances
        let query_output = query_account_native_denom_balance(&app, OWNER.clone(), native_huahua.clone());
        assert_eq!(query_output.amount, Uint128::new(99999999999768899982));
        let query_output = query_account_native_denom_balance(&app, WALLET2.clone(), native_huahua.clone());
        // Which is 100000018-1100000-1500000-(100000018*0.042)
        assert_eq!(query_output.amount, Uint128::new(99999999999962100018));

        // Profile updated
        let execute_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            WALLET2.to_string()
        ).unwrap();
        assert_eq!(
            execute_output.sell_info,
            Some(vec![TradeInfo {denom: native_huahua.clone(), volume_value: Uint128::new(100000018u128)}])
        );
        assert_eq!(
            execute_output.buy_info,
            Some(vec![])
        );

        let execute_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            OWNER.to_string()
        ).unwrap();
        assert_eq!(
            execute_output.buy_info,
            Some(vec![TradeInfo {denom: native_huahua.clone(), volume_value: Uint128::new(100000018u128)}])
        );
        assert_eq!(
            execute_output.sell_info,
            Some(vec![])
        );

        // We also check the marketplace info
        //QAZX
        let query_output = query_function_marketplace_test_get_marketplace_info(&app, nft_marketplace_smart_contract_addr.clone());
        let huahua_output = query_output.unwrap();
        assert_eq!(huahua_output[0].denom, native_huahua.clone().to_string());
        assert_eq!(huahua_output[0].nfts_for_sale, 37);
        assert_eq!(huahua_output[0].realized_sales_counter, 1);
        assert_eq!(huahua_output[0].total_realized_sales_volume, Uint128::new(100_000_018u128));
        assert_eq!(huahua_output[0].total_marketplace_fees, Uint128::new(4_200_000u128));
        assert_eq!(huahua_output[0].marketplace_fees_to_claim, Uint128::new(4_200_000u128));

        let query_output = query_function_marketplace_test_get_collection_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            None,
            Option::from(100)
        );
        let query_output = query_output.unwrap();
        let in_it = query_output.iter().any(|nft_sale| nft_sale.token_id == "Tokenx19");
        assert_eq!(in_it, false);

        let query_output = query_function_marketplace_test_get_nft_collection_volume(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())));
        // =100000018*120/1000000 = 12000
        assert_eq!(query_output, Uint128::new(12_000u128));

        let query_output = query_function_marketplace_test_get_marketplace_volume(
            &app,
            nft_marketplace_smart_contract_addr.clone()
        );
        assert_eq!(query_output, Uint128::new(12_000u128));

        let query_output = query_function_marketplace_test_get_token_id_sale_history(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            TokenId::from("Tokenx19".to_string())
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].seller, WALLET2.to_string());
        assert_eq!(query_output[0].buyer, OWNER.to_string());
        assert_eq!(query_output[0].nft_collection_address, Addr::unchecked(cw721_base_smart_contract_addr1.clone()));
        assert_eq!(query_output[0].token_id, "Tokenx19".to_string());
        assert_eq!(query_output[0].sale_price_value, Uint128::new(100_000_018u128));
        assert_eq!(query_output[0].sale_price_denom, native_huahua.clone());

        // Wallet2 buy Token19 from Owner
        let info = mock_info(WALLET2, &coins(100_000_018u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr1.to_string()),
            "Token18".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate NFTs for sale for the collection
        let query_output = query_function_marketplace_test_get_collection_all_nfts_for_sale(
            &app,
            nft_marketplace_smart_contract_addr.clone().to_string(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            None,
            Option::from(100)
        );
        assert_eq!(query_output.unwrap().len(), 36);

        // Check the stats
        let query_output = query_function_marketplace_test_get_marketplace_info(&app, nft_marketplace_smart_contract_addr.clone());
        let huahua_output = query_output.unwrap();
        assert_eq!(huahua_output[0].denom, native_huahua.clone().to_string());
        assert_eq!(huahua_output[0].nfts_for_sale, 36);
        assert_eq!(huahua_output[0].realized_sales_counter, 2);
        assert_eq!(huahua_output[0].total_realized_sales_volume, Uint128::new(100_000_018u128 * 2u128));
        assert_eq!(huahua_output[0].total_marketplace_fees, Uint128::new(8_400_000u128));
        assert_eq!(huahua_output[0].marketplace_fees_to_claim, Uint128::new(8_400_000u128));

        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].denom, native_huahua.clone().to_string());
        assert_eq!(query_output[1].nfts_for_sale, 36);
        assert_eq!(query_output[1].realized_trades, 2);
        assert_eq!(query_output[1].total_volume, Uint128::new(100_000_018u128 * 2u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_000u128));

        let query_output = query_function_marketplace_test_get_token_id_sale_history(
            &app,
            nft_marketplace_smart_contract_addr.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
            TokenId::from("Token18".to_string())
        );
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].seller, OWNER.to_string());
        assert_eq!(query_output[0].buyer, WALLET2.to_string());
        assert_eq!(query_output[0].nft_collection_address, Addr::unchecked(cw721_base_smart_contract_addr1.clone()));
        assert_eq!(query_output[0].token_id, "Token18".to_string());
        assert_eq!(query_output[0].sale_price_value, Uint128::new(100_000_018u128));
        assert_eq!(query_output[0].sale_price_denom, native_huahua.clone());

        let query_output = query_function_marketplace_test_get_marketplace_volume(
            &app,
            nft_marketplace_smart_contract_addr.clone()
        );
        assert_eq!(query_output, Uint128::new(120u128 * 2u128 * 100_000_018u128 / 1_000_000u128));

        let query_output = query_function_marketplace_test_get_nft_collection_volume(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())));
        assert_eq!(query_output, Uint128::new(120u128 * 2u128 * 100_000_018u128 / 1_000_000u128));

        // Validate: if floor is updated accurately
        let info = mock_info(WALLET2, &coins(100_000_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr1.to_string()),
            "Token0".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &coins(100_000_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr1.to_string()),
            "Tokenx1".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);

        // Check floor
        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[1].denom, native_huahua.clone().to_string());
        assert_eq!(query_output[1].nfts_for_sale, 34);
        assert_eq!(query_output[1].realized_trades, 4);
        assert_eq!(query_output[1].total_volume, Uint128::new(100_000_018u128 * 2u128 + 200_000_000u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_001u128));

        // Claim the fees
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_claim_marketplace_fees(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(execute_output.is_err(), false);

        // Check that the fees are resetted
        let query_output = query_function_marketplace_test_get_marketplace_info(&app, nft_marketplace_smart_contract_addr.clone());
        let query_output = query_output.unwrap();
        query_output
            .iter()
            .for_each(|by_denom| assert_eq!(by_denom.marketplace_fees_to_claim, Uint128::zero()));

        // Now we check the rewards for a bigger sale
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw2981_multi_test_approve(
            &mut app,
            &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token20".to_string(),
            1571797419u64 + 87000u64
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &coins(6_900_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token20".clone().to_string(),
                sale_price_value: Uint128::new(100_000_000_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), false);


        let info = mock_info(WALLET2, &coins(100_000_000_000_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr1.to_string()),
            "Token20".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);

        // Check if the received reward is accurate as per the sale
        let output_query = query_account_cw20_balance(&app, reward_token.clone(), Addr::unchecked(WALLET2));
        // 100_000_000_000_000u128 * 120u128 / 1_000_000u128 = 12000$USDC  ->
        assert_eq!(output_query.unwrap().balance, Uint128::new(12_000_000_000u128));
        let output_query = query_account_cw20_balance(&app, reward_token.clone(), Addr::unchecked(OWNER));
        // 100_000_000_000_000u128 * 120u128 / 1_000_000u128 = 12000$USDC  ->
        assert_eq!(output_query.unwrap().balance, Uint128::new(1_000_000_000_000_000u128 - 100_000_000_000u128 + 12_000_000_000u128));
    }

    #[test]
    fn unit_test_execute_function_create_update_profile_test() {
        let _deps = mock_dependencies();
        let (
            mut app,
            _native_huahua,
            _native_atom,
            nft_marketplace_smart_contract_addr,
            _cw721_base_smart_contract_addr1,
            _cw721_base_smart_contract_addr2,
            _price_oracle_smart_contract_addr,
            reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();

        // Enable contract
        let info = mock_info(OWNER, &vec![]);
        let output_execute = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(output_execute.is_err(), false);

        // Query non-existing profile
        let query_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            WALLET2.to_string()
        );
        assert_eq!(
            query_output.unwrap_err().to_string(),
            "Generic error: Querier contract error: nft_marketplace_utils::profile::Profile not found".to_string()
        );

        // Create profile
        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_create_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            None
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the profile's info
        let execute_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            WALLET3.to_string()
        ).unwrap();
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

        // Update the profile: wonrg address
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_update_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            Profile {
                address: WALLET3.to_string(),
                vip_level: None,
                profile_nft_collection: None,
                profile_nft_token_id: None,
                background_nft_collection: None,
                background_nft_token_id: None,
                description: Some("Hello this is my description".to_string()),
                nft_showcase: None,
                links: None,
                extended_profile: None,
                number_of_trades: None,
                buy_info: None,
                sell_info: None,
            }
        );
        assert_eq!(execute_output.is_err(), true);

        // Update the profile: valid
        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_update_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            Profile {
                address: WALLET3.to_string(),
                vip_level: None,
                profile_nft_collection: None,
                profile_nft_token_id: None,
                background_nft_collection: None,
                background_nft_token_id: None,
                description: Some("Hello this is my description".to_string()),
                nft_showcase: None,
                links: None,
                extended_profile: None,
                number_of_trades: None,
                buy_info: None,
                sell_info: None,
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the updated value
        let profile_info_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            WALLET3.to_string()
        ).unwrap();
        // Cannot be updated until profile reaches the right level
        assert_eq!(profile_info_output.description, None);

        // Now upgrading the profile
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw20_test_transfer_cw20(
            &mut app,
            &Addr::unchecked(WALLET3),
            Addr::unchecked(reward_token.clone()),
            info.clone(),
            Uint128::new(1_000_000_000u128)
        );
        assert_eq!(execute_output.is_err(), false);

        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_level_up_my_profile(
            &mut app,
            info.clone(),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(1u128),
            ReceiveMsg::LevelUpProfile {}
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().source().expect("_").to_string(),
            "InvalidAmountReceivedForLevelUp".to_string()
        );

        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_level_up_my_profile(
            &mut app,
            info.clone(),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(1_000u128),
            ReceiveMsg::LevelUpProfile {}
        );
        assert_eq!(execute_output.is_err(), false);

        // Update the profile: valid
        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_update_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            Profile {
                address: WALLET3.to_string(),
                vip_level: None,
                profile_nft_collection: None,
                profile_nft_token_id: None,
                background_nft_collection: None,
                background_nft_token_id: None,
                description: Some("Hello this is my description".to_string()),
                nft_showcase: None,
                links: None,
                extended_profile: None,
                number_of_trades: None,
                buy_info: None,
                sell_info: None,
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the updated value
        let profile_info_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            WALLET3.to_string()
        ).unwrap();
        assert_eq!(profile_info_output.vip_level, Some(VipLevel::Level1));
        assert_eq!(profile_info_output.description, None);


        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_level_up_my_profile(
            &mut app,
            info.clone(),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            reward_token.clone(),
            Uint128::new(10_000u128),
            ReceiveMsg::LevelUpProfile {}
        );
        assert_eq!(execute_output.is_err(), false);

        // Update the profile: valid
        let info = mock_info(WALLET3, &vec![]);
        let execute_output = execute_function_marketplace_test_update_my_profile(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            Profile {
                address: WALLET3.to_string(),
                vip_level: None,
                profile_nft_collection: None,
                profile_nft_token_id: None,
                background_nft_collection: None,
                background_nft_token_id: None,
                description: Some("Hello this is my description".to_string()),
                nft_showcase: None,
                links: None,
                extended_profile: None,
                number_of_trades: None,
                buy_info: None,
                sell_info: None,
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Validate the updated value
        let profile_info_output = query_function_marketplace_test_get_profile_info(
            &app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            WALLET3.to_string()
        ).unwrap();
        assert_eq!(profile_info_output.vip_level, Some(VipLevel::Level2));
        assert_eq!(profile_info_output.description, Some("Hello this is my description".to_string()));

    }

    #[test]
    fn test_marketplace_smart_contract_buy_nft_function_floor_denoms_collections() {
        let _deps = mock_dependencies();
        let (
            mut app,
            native_huahua,
            native_atom,
            nft_marketplace_smart_contract_addr,
            cw721_base_smart_contract_addr1,
            cw721_base_smart_contract_addr2,
            price_oracle_smart_contract_addr,
            reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();
        // Enable the contract
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(execute_output.is_err(), false);

        // Need to add Denom in the Price Oracle
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                crypto_assets_price_oracle::msg::UpdateConfigEnum::AddDenoms {
                    denoms: vec![native_huahua.clone(), native_atom.clone()]
                }]
        );
        assert_eq!(execute_output.is_err(), false);

        // Price Oracle: Add prices
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: native_huahua.clone().to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: native_atom.clone().to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1 as u64),
            }
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint several NFTs
        let info = mock_info(OWNER, &vec![]);
        for (token_number1, token_number2) in (1..22).enumerate() {
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&*token_number1.to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&*token_number2.to_string());
            let execute_output = execute_function_cw2981_multi_test_mint(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                info.clone(),
                token_id_iter_str.to_string(),
                OWNER.to_string(),
                Some(vec![
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
                ])
            );
            assert_eq!(execute_output.is_err(), false);
            let execute_output = execute_function_cw2981_multi_test_mint(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
                info.clone(),
                token_id_iter_str2.to_string(),
                WALLET2.to_string(),
                Some(vec![
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER1.to_string()), royalty_permille_int: 11 },
                    Royalty { receiver: Addr::unchecked(ROYALTY_RECEIVER2.to_string()), royalty_permille_int: 15 }
                ])
            );
            assert_eq!(execute_output.is_err(), false);
        }

        // Add the collection to the marketplace contract
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())),
        );
        assert_eq!(execute_output.is_err(), false);
        // Add the collection to the marketplace contract
        let execute_output = execute_function_marketplace_test_add_new_collection(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())),
        );
        assert_eq!(execute_output.is_err(), false);

        // Mint some coins for purchases
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_huahua.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: OWNER.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_atom.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: WALLET2.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_huahua.clone())],
            }
        }))
            .unwrap();
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: WALLET2.clone().to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, native_atom.clone())],
            }
        }))
            .unwrap();

        // Send Reward tokens to the contract to distribute
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw20_test_transfer_cw20(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            Addr::unchecked(reward_token.clone()),
            info.clone(),
            Uint128::new(100_000_000_000u128)
        );
        assert_eq!(execute_output.is_err(), false);

        // Put some NFTs for sale to check if the floor is accurate by collection and denom
        let info = mock_info(OWNER, &&coins(6_900_000u128, "uhuahua".to_string()));
        let info2 = mock_info(WALLET2, &&coins(6_900_000u128, "uhuahua".to_string()));
        // Put up for sale several NFTs
        for (token_number1, token_number2) in (1..5).enumerate() {
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&*token_number1.clone().to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&*token_number2.clone().to_string());
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                token_id_iter_str.clone().to_string(),
                1571797419u64 + 87000u64
            );
            assert_eq!(execute_output.is_err(), false);
            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str.clone().to_string(),
                    sale_price_value: Uint128::new(100_000_000u128 + token_number1.clone() as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);

            // Wallet 2
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                token_id_iter_str2.clone().to_string(),
                1571797419u64 + 87000u64
            );
            assert_eq!(execute_output.is_err(), false);

            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                NftSale {
                    seller: WALLET2.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr2.to_string(),
                    token_id: token_id_iter_str2.clone().to_string(),
                    sale_price_value: Uint128::new(200_000_000u128 + token_number1.clone() as u128),
                    sale_price_denom: native_huahua.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);
        }

        for token_number1 in 6..8 {
            let token_number2 = token_number1.clone();
            let mut token_id_iter_str: String = "Token".to_string().to_owned();
            token_id_iter_str.push_str(&*token_number1.clone().to_string());
            let mut token_id_iter_str2: String = "Tokenx".to_string().to_owned();
            token_id_iter_str2.push_str(&*token_number2.clone().to_string());
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                token_id_iter_str.clone().to_string(),
                1571797419u64 + 87000u64
            );
            assert_eq!(execute_output.is_err(), false);
            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info.clone(),
                NftSale {
                    seller: OWNER.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                    token_id: token_id_iter_str.clone().to_string(),
                    sale_price_value: Uint128::new(10_000u128 + token_number1.clone() as u128),
                    sale_price_denom: native_atom.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);

            // Wallet 2
            let execute_output = execute_function_cw2981_multi_test_approve(
                &mut app,
                &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                token_id_iter_str2.clone().to_string(),
                1571797419u64 + 87000u64
            );
            assert_eq!(execute_output.is_err(), false);

            let execute_output = execute_function_marketplace_test_sell_nft(
                &mut app,
                &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
                info2.clone(),
                NftSale {
                    seller: WALLET2.to_string(),
                    nft_collection_address: cw721_base_smart_contract_addr2.to_string(),
                    token_id: token_id_iter_str2.clone().to_string(),
                    sale_price_value: Uint128::new(20_000u128 + token_number1.clone() as u128),
                    sale_price_denom: native_atom.to_string(),
                    sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
                }
            );
            assert_eq!(execute_output.is_err(), false);
        }
        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr1.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(10_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(100_000_000u128));
        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(200_000_000u128));

        // Owner buy Tokenx1 from Wallet2
        let info = mock_info(OWNER, &coins(200_000_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr2.to_string()),
            "Tokenx1".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);

        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(200_000_001u128));

        // Test until there is nothing left
        let info = mock_info(OWNER, &coins(200_000_001u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr2.to_string()),
            "Tokenx2".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &coins(200_000_002u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr2.to_string()),
            "Tokenx3".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &coins(200_000_003u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr2.to_string()),
            "Tokenx4".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);

        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(1_000_000_000_000_000_000u128));

        // Wallet 2
        let execute_output = execute_function_cw2981_multi_test_approve(
            &mut app,
            &Addr::unchecked(cw721_base_smart_contract_addr2.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info2.clone(),
            "Tokenx5".clone().to_string(),
            1571797419u64 + 87000u64
        );
        assert_eq!(execute_output.is_err(), false);

        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info2.clone(),
            NftSale {
                seller: WALLET2.to_string(),
                nft_collection_address: cw721_base_smart_contract_addr2.to_string(),
                token_id: "Tokenx5".clone().to_string(),
                sale_price_value: Uint128::new(20_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), false);

        let query_output = query_function_marketplace_test_get_nft_collection_info(&app, nft_marketplace_smart_contract_addr.clone(), NftCollectionAddress::from(Addr::unchecked(cw721_base_smart_contract_addr2.clone())));
        let query_output = query_output.unwrap();
        assert_eq!(query_output[0].current_floor, Uint128::new(20_006u128));
        assert_eq!(query_output[1].current_floor, Uint128::new(20_000u128));

        // Need to test the General Stats
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_cw2981_multi_test_approve(
            &mut app,
            &Addr::unchecked(cw721_base_smart_contract_addr1.clone()),
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            "Token5".clone().to_string(),
            1571797419u64 + 87000u64
        );
        assert_eq!(execute_output.is_err(), false);

        let info = mock_info(OWNER, &coins(6_900_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_sell_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftSale {
                seller: OWNER.to_string(),
                nft_collection_address: cw721_base_smart_contract_addr1.to_string(),
                token_id: "Token5".clone().to_string(),
                sale_price_value: Uint128::new(200_000_000u128),
                sale_price_denom: native_huahua.to_string(),
                sale_expiration: Timestamp::from_seconds(1571797419u64 + 87000u64)
            }
        );
        assert_eq!(execute_output.is_err(), false);

        let info = mock_info(WALLET2, &coins(200_000_000u128, native_huahua.clone()));
        let execute_output = execute_function_marketplace_test_buy_nft(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone()),
            info.clone(),
            NftCollectionAddress::from(cw721_base_smart_contract_addr1.to_string()),
            "Token5".to_string(),
        );
        assert_eq!(execute_output.is_err(), false);

    }

    #[test]
    fn unit_test_offer_make_cancel_accept() {
        let _deps = mock_dependencies();
        let (
            mut app,
            _native_huahua,
            _native_atom,
            nft_marketplace_smart_contract_addr,
            _cw721_base_smart_contract_addr1,
            _cw721_base_smart_contract_addr2,
            _price_oracle_smart_contract_addr,
            _reward_token,
            _invalid_reward_token
        ) = instantiate_necessary_for_tests();
        // Enable the contract
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_enable_disable(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone()
        );
        assert_eq!(execute_output.is_err(), false);

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_marketplace_test_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            NftOffer {
                offerer_address: "".to_string(),
                nft_collection_address: "".to_string(),
                token_id: "".to_string(),
                offer_price_value: Default::default(),
                offer_price_denom: "".to_string(),
                offer_expiration: Default::default(),
            }
        );
        assert_eq!(execute_output.is_err(), true);

        let execute_output = execute_function_marketplace_test_cancel_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            "".to_string(),
            "".to_string()
        );
        assert_eq!(execute_output.is_err(), true);

        let execute_output = execute_function_marketplace_test_accept_offer(
            &mut app,
            &Addr::unchecked(nft_marketplace_smart_contract_addr.clone().to_string()),
            info.clone(),
            "".to_string(),
            "".to_string(),
            "".to_string()
        );
        assert_eq!(execute_output.is_err(), true);
    }

}
