#[cfg(test)]
mod testing {
    use anyhow::Result as AnyResult;
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{Addr, Empty, MessageInfo, StdResult, Timestamp, Uint128};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

    use crate::msg::UpdateConfigEnum;
    use general_utils::denominations::{AcceptedDenominations, Denomination};
    use price_oracle_utils::config::Config;
    use price_oracle_utils::oracle::{OraclePrice, OraclePrices};

    pub const OWNER: &str = "chihuahua15eda4nt0sragssfdwenlm0y3maj0qdl8tr23";
    pub const ORACLE_FEEDER: &str = "chihuahua15eda4nt0sragssfdwenlm0y3maj0qdl8tlk403";
    pub const ORACLE_NEW_OWNER: &str = "chihuahua15eda4nt0sragssfdwenlm0y3maj0qdl8tlk404";
    pub const ORACLE_NEW_FEEDER: &str = "chihuahua15eda4nt0sragssfdwenlm0y3maj0qdl8tlk405";
    pub const WALLET2: &str = "chihuahua15eda4nt0sragssfdwenlm0y3maj0qdl8tlk406";

    fn mock_app() -> App {
        App::default()
    }

    fn instantiate_necessary_for_tests() -> (App, String, String, String) {
        let mut app: App = mock_app();
        let native_huahua: String = "uhuahua".to_string();
        let native_atom: String =
            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string();
        let price_oracle_smart_contract_address: Addr =
            instantiate_smart_contract_test_price_oracle(
                &mut app,
                AcceptedDenominations {
                    list_of_denoms: vec![native_huahua.clone(), native_atom.clone()],
                },
            );
        (
            app,
            native_huahua,
            native_atom,
            price_oracle_smart_contract_address.to_string(),
        )
    }

    pub fn smart_contract_def_test_price_oracle() -> Box<dyn Contract<Empty>> {
        let smart_contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(smart_contract)
    }

    fn instantiate_smart_contract_test_price_oracle(
        app: &mut App,
        accepted_ibc_denoms: AcceptedDenominations,
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_price_oracle());
        let init_msg = crate::msg::InstantiateMsg {
            contract_owner: OWNER.to_string(),
            prices_feeder: ORACLE_FEEDER.to_string(),
            max_history_length: 10,
            accepted_ibc_denoms,
        };
        app.instantiate_contract(
            contract_code_id,
            Addr::unchecked(OWNER),
            &init_msg,
            &[],
            "price_oracle",
            None,
        )
        .unwrap()
    }

    // Execute Functions for Price Oracle contract
    fn oracle_test_exec_update_config(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        configs_to_update: Vec<UpdateConfigEnum>,
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::UpdateConfig {
            list_of_updates: configs_to_update,
        };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    fn oracle_test_exec_feed_prices(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        new_prices: OraclePrices,
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::FeedPrices { prices: new_prices };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    // Query Functions: Price Oracle
    fn oracle_test_query_get_config_and_current_prices<T: Into<String>>(
        app: &App,
        contract_addr: T,
    ) -> Config {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetConfigAndCurrentPrices {};
        let result: Config = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    fn oracle_test_query_get_usdc_price_from_amount_and_denom<T: Into<String>>(
        app: &App,
        contract_addr: T,
        amount: Uint128,
        denom: Denomination,
    ) -> Uint128 {
        let msg: crate::msg::QueryMsg =
            crate::msg::QueryMsg::GetUsdcPriceFromAmountAndDenom { amount, denom };
        let result: Uint128 = app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    fn oracle_test_query_get_latest_historical_prices<T: Into<String>>(
        app: &App,
        contract_addr: T,
        length: Option<u32>,
    ) -> StdResult<Vec<(u64, OraclePrices)>> {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetLatestHistoricalPrices { length };
        let result: StdResult<Vec<(u64, OraclePrices)>> =
            app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    #[test]
    fn test_price_oracle_smart_contract_instantiate_contract() {
        // Validate:
        // 1. If instantiation works
        // 2. Initial states are as expected
        let _deps = mock_dependencies();
        let (app, _native_huahua, _native_atom, price_oracle_smart_contract_addr) =
            instantiate_necessary_for_tests();
        let config_info =
            oracle_test_query_get_config_and_current_prices(&app, price_oracle_smart_contract_addr);
        assert_eq!(config_info.contract_owner, OWNER);
        assert_eq!(config_info.prices_feeder, ORACLE_FEEDER);
        assert_eq!(config_info.max_history_length, 10);
        assert_eq!(config_info.oldest_history_id, 0);
        assert_eq!(config_info.next_history_id, 1);
        assert_eq!(
            config_info.accepted_ibc_denoms,
            AcceptedDenominations {
                list_of_denoms: vec![
                    "uhuahua".to_string(),
                    "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                        .to_string()
                ]
            }
        );
        assert_eq!(config_info.current_prices.prices, []);
        assert_eq!(
            config_info.current_prices.at_time,
            Timestamp::from_seconds(0)
        );
    }

    #[test]
    fn test_price_oracle_smart_contract_update_config() {
        // Validate:
        // 1. Only owner can update
        // 2. All update methods works as expected
        // 3. Update states are as expected
        let _deps = mock_dependencies();
        let (mut app, _native_huahua, _native_atom, price_oracle_smart_contract_addr) =
            instantiate_necessary_for_tests();

        let info = mock_info(WALLET2, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::ChangeMaxLength { length: 20 }],
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![
                UpdateConfigEnum::ChangeMaxLength { length: 20 },
                UpdateConfigEnum::AddDenoms {
                    denoms: vec!["random_denom".to_string(), "random_denom2".to_string()],
                },
            ],
        );
        assert!(execute_output.is_ok());
        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::RemoveDenoms {
                denoms: vec!["random_denom".to_string()],
            }],
        );
        assert!(execute_output.is_ok());
        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::ChangePriceFeeder {
                new_feeder: ORACLE_NEW_FEEDER.to_string(),
            }],
        );
        assert!(execute_output.is_ok());
        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::UpdateOwner {
                new_owner: ORACLE_NEW_OWNER.to_string(),
            }],
        );
        assert!(execute_output.is_ok());

        let config_info = oracle_test_query_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
        );
        assert_eq!(config_info.contract_owner, ORACLE_NEW_OWNER);
        assert_eq!(config_info.prices_feeder, ORACLE_NEW_FEEDER);
        assert_eq!(config_info.current_prices.prices, []);
        assert_eq!(
            config_info.current_prices.at_time,
            Timestamp::from_seconds(0)
        );
        assert_eq!(
            config_info.accepted_ibc_denoms.list_of_denoms,
            vec![
                "uhuahua".to_string(),
                "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                "random_denom2".to_string()
            ]
        );
        assert_eq!(config_info.next_history_id, 1);
        assert_eq!(config_info.oldest_history_id, 0);

        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::ChangeMaxLength { length: 20 }],
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        let info = mock_info(ORACLE_NEW_OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr),
            info,
            vec![UpdateConfigEnum::ChangeMaxLength { length: 20 }],
        );
        assert!(execute_output.is_ok());
    }

    #[test]
    fn test_price_oracle_smart_contract_feed_prices() {
        // Validate:
        // 1. FeedPrices can only be used by the price feed address
        // 2. THe fed prices should be all of the those expected by the oracle
        // 3. The timestamp of the prices should be > than the last prices
        // 4. Check the states update are accurate
        // 5. Check if the historical prices are right too
        let _deps = mock_dependencies();
        let (mut app, _native_huahua, native_atom, price_oracle_smart_contract_addr) =
            instantiate_necessary_for_tests();
        let info = mock_info(WALLET2, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589999u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "HELLO".to_string(), // This denom is not accepted
                        name: "HELLO".to_string(),
                        ibc_denom: "HELLO".to_string(),
                        value_usdc_6_decimals: Uint128::new(111u128 + 1_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1_u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidDenominationReceived".to_string()
        );

        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1_u64),
            },
        );
        assert!(execute_output.is_ok());

        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::AddDenoms {
                denoms: vec!["test_denom".to_string()],
            }],
        );
        assert!(execute_output.is_ok());

        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 22_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 22_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1686589235u64 + 22_u64),
            },
        );
        assert!(execute_output.is_ok());

        // the above should work but the new test_denom should have no price and the new one should be updated
        let denom_current_price = oracle_test_query_get_usdc_price_from_amount_and_denom(
            &app,
            price_oracle_smart_contract_addr.clone(),
            Uint128::new(100_000u128),
            native_atom.clone(),
        );
        assert_eq!(denom_current_price, Uint128::new(1355513u128));

        let config_info = oracle_test_query_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
        );
        assert_eq!(
            config_info.current_prices.prices,
            vec![
                OraclePrice {
                    ticker: "HUAHUA".to_string(),
                    name: "Chihuahua Chain".to_string(),
                    ibc_denom: "uhuahua".to_string(),
                    value_usdc_6_decimals: Uint128::new(119u128 + 22_u128),
                },
                OraclePrice {
                    ticker: "ATOM".to_string(),
                    name: "Cosmos Hub".to_string(),
                    ibc_denom:
                        "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                            .to_string(),
                    value_usdc_6_decimals: Uint128::new(13_555_111u128 + 22_u128),
                }
            ]
        );

        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::RemoveDenoms {
                denoms: vec!["test_denom".to_string()],
            }],
        );
        assert!(execute_output.is_ok());

        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1686589235u64 + 23_u64),
            },
        );
        assert!(execute_output.is_ok());

        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589236u64 - 10_u64),
            },
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidTimeForPriceReceived".to_string()
        );

        for i in 0..20 {
            let info = mock_info(ORACLE_FEEDER, &[]);
            let execute_output = oracle_test_exec_feed_prices(
                &mut app,
                &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
                info.clone(),
                OraclePrices {
                    prices: vec![
                        OraclePrice {
                            ticker: "HUAHUA".to_string(),
                            name: "Chihuahua Chain".to_string(),
                            ibc_denom: "uhuahua".to_string(),
                            value_usdc_6_decimals: Uint128::new(119u128 + i as u128),
                        },
                        OraclePrice {
                            ticker: "ATOM".to_string(),
                            name: "Cosmos Hub".to_string(),
                            ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                            value_usdc_6_decimals: Uint128::new(13_555_111u128 + i as u128),
                        },
                    ],
                    at_time: Timestamp::from_seconds(1696589235u64 + i as u64),
                }
            );
            assert!(execute_output.is_ok());
        }

        let histo_prices_query_info = oracle_test_query_get_latest_historical_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
            None,
        );
        assert_eq!(histo_prices_query_info.unwrap().len(), 10);

        let histo_prices_query_info = oracle_test_query_get_latest_historical_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
            None,
        )
        .unwrap();
        assert_eq!(histo_prices_query_info[0].0, 23);
        assert!(histo_prices_query_info[0].1.at_time > histo_prices_query_info[1].1.at_time);

        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(999u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(14_444_444u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1776589999u64),
            },
        );
        assert!(execute_output.is_ok());

        let histo_prices_query_info = oracle_test_query_get_latest_historical_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
            Some(2),
        )
        .unwrap();
        let config_query_info = oracle_test_query_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
        );
        assert_eq!(
            histo_prices_query_info[0].1.at_time,
            config_query_info.current_prices.at_time
        );
        assert_eq!(
            histo_prices_query_info[0].1.prices,
            config_query_info.current_prices.prices
        );
        assert_eq!(
            histo_prices_query_info[0].1.prices,
            vec![
                OraclePrice {
                    ticker: "HUAHUA".to_string(),
                    name: "Chihuahua Chain".to_string(),
                    ibc_denom: "uhuahua".to_string(),
                    value_usdc_6_decimals: Uint128::new(999u128),
                },
                OraclePrice {
                    ticker: "ATOM".to_string(),
                    name: "Cosmos Hub".to_string(),
                    ibc_denom:
                        "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                            .to_string(),
                    value_usdc_6_decimals: Uint128::new(14_444_444u128),
                },
            ]
        );

        let denom_current_price = oracle_test_query_get_usdc_price_from_amount_and_denom(
            &app,
            price_oracle_smart_contract_addr.clone(),
            Uint128::new(100_000u128),
            native_atom,
        );
        assert_eq!(denom_current_price, Uint128::new(1444444u128));

        let config_info = oracle_test_query_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
        );
        assert_eq!(config_info.contract_owner, OWNER);
        assert_eq!(config_info.prices_feeder, ORACLE_FEEDER);
        assert_eq!(
            config_info.current_prices.prices,
            vec![
                OraclePrice {
                    ticker: "HUAHUA".to_string(),
                    name: "Chihuahua Chain".to_string(),
                    ibc_denom: "uhuahua".to_string(),
                    value_usdc_6_decimals: Uint128::new(999u128),
                },
                OraclePrice {
                    ticker: "ATOM".to_string(),
                    name: "Cosmos Hub".to_string(),
                    ibc_denom:
                        "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                            .to_string(),
                    value_usdc_6_decimals: Uint128::new(14_444_444u128),
                }
            ]
        );
        assert_eq!(
            config_info.current_prices.at_time,
            Timestamp::from_seconds(1776589999u64)
        );
        assert_eq!(
            config_info.accepted_ibc_denoms.list_of_denoms,
            vec![
                "uhuahua".to_string(),
                "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string()
            ]
        );
        // Max length of 10
        assert_eq!(config_info.next_history_id, 25);
        assert_eq!(config_info.oldest_history_id, 14);

        // Add a new denom, updates all prices, then miss 1 update
        // Outcome -> Should have the non-updated price remain the same.
        let info = mock_info(OWNER, &[]);
        let execute_output = oracle_test_exec_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            vec![UpdateConfigEnum::AddDenoms {
                denoms: vec!["OSMOSIS".to_string()],
            }],
        );
        assert!(execute_output.is_ok());

        // New prices for all cryptos
        let info = mock_info(ORACLE_FEEDER, &[]);
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(111u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(222u128),
                    },
                    OraclePrice {
                        ticker: "OSMOSIS".to_string(),
                        name: "OSMOSIS".to_string(),
                        ibc_denom: "OSMOSIS".to_string(),
                        value_usdc_6_decimals: Uint128::new(333u128),
                    },
                ],
                at_time: Timestamp::from_seconds(2676589235u64 + 1_u64),
            },
        );
        assert!(execute_output.is_ok());

        // Confirm the updated info
        let config_info = oracle_test_query_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
        );
        assert_eq!(
            config_info.current_prices.prices,
            vec![
                OraclePrice {
                    ticker: "HUAHUA".to_string(),
                    name: "Chihuahua Chain".to_string(),
                    ibc_denom: "uhuahua".to_string(),
                    value_usdc_6_decimals: Uint128::new(111u128),
                },
                OraclePrice {
                    ticker: "ATOM".to_string(),
                    name: "Cosmos Hub".to_string(),
                    ibc_denom:
                        "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                            .to_string(),
                    value_usdc_6_decimals: Uint128::new(222u128),
                },
                OraclePrice {
                    ticker: "OSMOSIS".to_string(),
                    name: "OSMOSIS".to_string(),
                    ibc_denom: "OSMOSIS".to_string(),
                    value_usdc_6_decimals: Uint128::new(333u128),
                },
            ]
        );
        assert_eq!(
            config_info.current_prices.at_time,
            Timestamp::from_seconds(2676589235u64 + 1_u64),
        );

        // Now we only update 2 prices
        let execute_output = oracle_test_exec_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info,
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(444u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom:
                            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                                .to_string(),
                        value_usdc_6_decimals: Uint128::new(555u128),
                    },
                ],
                at_time: Timestamp::from_seconds(3676589235u64 + 1_u64),
            },
        );
        assert!(execute_output.is_ok());

        // And check if the value are as expected
        let config_info =
            oracle_test_query_get_config_and_current_prices(&app, price_oracle_smart_contract_addr);
        assert_eq!(
            config_info.current_prices.prices,
            vec![
                OraclePrice {
                    ticker: "HUAHUA".to_string(),
                    name: "Chihuahua Chain".to_string(),
                    ibc_denom: "uhuahua".to_string(),
                    value_usdc_6_decimals: Uint128::new(444u128),
                },
                OraclePrice {
                    ticker: "ATOM".to_string(),
                    name: "Cosmos Hub".to_string(),
                    ibc_denom:
                        "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2"
                            .to_string(),
                    value_usdc_6_decimals: Uint128::new(555u128),
                },
                OraclePrice {
                    ticker: "OSMOSIS".to_string(),
                    name: "OSMOSIS".to_string(),
                    ibc_denom: "OSMOSIS".to_string(),
                    value_usdc_6_decimals: Uint128::new(333u128), // Unchanged
                },
            ]
        );
        assert_eq!(
            config_info.current_prices.at_time,
            Timestamp::from_seconds(3676589235u64 + 1_u64),
        );
    }
}
