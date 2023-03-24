#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Empty, MessageInfo, StdResult, Timestamp, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use general_utils::denominations::{AcceptedDenominations, Denomination};
    use price_oracle_utils::config::Config;
    use price_oracle_utils::oracle::{OraclePrice, OraclePrices};
    use crate::msg::UpdateConfigEnum;
    use anyhow::Result as AnyResult;

    const OWNER: &str = "chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa23";
    const FEEDER: &str = "chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa25";
    const NEW_OWNER: &str = "chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa11";
    const NEW_FEEDER: &str = "chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa22";
    const WALLET2: &str = "chihuahua1wwwtnl8hdjajmq87psg3d3h7jjn9g2h9fdewa24";


    fn mock_app() -> App {
        App::default()
    }

    fn instantiate_necessary_for_tests() -> (App, String, String, String) {
        let mut app: App = mock_app();
        let native_huahua: String = "uhuahua".to_string();
        let native_atom: String = "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string();
        let price_oracle_smart_contract_address: Addr = instantiate_smart_contract_test_price_oracle(
            &mut app,
            AcceptedDenominations { list_of_denoms: vec![native_huahua.clone(), native_atom.clone()] },
        );
        return (
            app,
            native_huahua.clone(),
            native_atom.clone(),
            price_oracle_smart_contract_address.to_string()
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
        accepted_ibc_denoms: AcceptedDenominations
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_price_oracle());
        let init_msg = crate::msg::InstantiateMsg {
            contract_owner: OWNER.to_string(),
            prices_feeder: FEEDER.to_string(),
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
        ).unwrap()
    }

    // Execute Functions for Price Oracle contract
    fn execute_function_price_oracle_test_update_config(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        configs_to_update: Vec<UpdateConfigEnum>
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::UpdateConfig { list_of_updates: configs_to_update };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    fn execute_function_price_oracle_test_feed_prices(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        new_prices: OraclePrices
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::FeedPrices { prices: new_prices };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    // Query Functions: Price Oracle
    fn query_function_price_oracle_test_get_config_and_current_prices<T: Into<String>>(
        app: &App,
        contract_addr: T
    ) -> Config {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetConfigAndCurrentPrices {};
        let result: Config =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    fn query_function_price_oracle_test_get_usdc_price_from_amount_and_denom<T: Into<String>>(
        app: &App,
        contract_addr: T,
        amount: Uint128,
        denom: Denomination
    ) -> Uint128 {
        let msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetUsdcPriceFromAmountAndDenom { amount, denom };
        let result: Uint128 =
            app.wrap().query_wasm_smart(contract_addr, &msg).unwrap();
        result
    }

    fn query_function_price_oracle_test_get_latest_historical_prices<T: Into<String>>(
        app: &App,
        contract_addr: T,
        length: Option<u32>
    ) -> StdResult<Vec<(u64, OraclePrices)>>  {
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
        let (
            app,
            _native_huahua,
            _native_atom,
            price_oracle_smart_contract_addr
        ) = instantiate_necessary_for_tests();
        let config_info = query_function_price_oracle_test_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone()
        );
        assert_eq!(config_info.contract_owner, OWNER);
        assert_eq!(config_info.prices_feeder, FEEDER);
        assert_eq!(config_info.max_history_length, 10);
        assert_eq!(config_info.oldest_history_id, 0);
        assert_eq!(config_info.next_history_id, 1);
        assert_eq!(
            config_info.accepted_ibc_denoms,
            AcceptedDenominations { list_of_denoms: vec![
                "uhuahua".to_string(),
                "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string()
            ] }
        );
        assert_eq!(config_info.current_prices.prices, vec![]);
        assert_eq!(config_info.current_prices.at_time, Timestamp::from_seconds(0));
    }

    #[test]
    fn test_price_oracle_smart_contract_update_config() {
        // Validate:
        // 1. Only owner can update
        // 2. All update methods works as expected
        // 3. Update states are as expected
        let _deps = mock_dependencies();
        let (
            mut app,
            _native_huahua,
            _native_atom,
            price_oracle_smart_contract_addr
        ) = instantiate_necessary_for_tests();

        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::ChangeMaxLength { length: 20 }
            ]
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::ChangeMaxLength { length: 20 },
                UpdateConfigEnum::AddDenoms { denoms: vec![
                    "random_denom".to_string(), "random_denom2".to_string()
                ] }
            ]
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::RemoveDenoms { denoms: vec!["random_denom".to_string()] }
            ]
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::ChangePriceFeeder { new_feeder: NEW_FEEDER.to_string() }
            ]
        );
        assert_eq!(execute_output.is_err(), false);
        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::UpdateOwner { new_owner: NEW_OWNER.to_string() }
            ]
        );
        assert_eq!(execute_output.is_err(), false);

        let config_info = query_function_price_oracle_test_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone()
        );
        assert_eq!(config_info.contract_owner, NEW_OWNER);
        assert_eq!(config_info.prices_feeder, NEW_FEEDER);
        assert_eq!(config_info.current_prices.prices, vec![]);
        assert_eq!(config_info.current_prices.at_time, Timestamp::from_seconds(0));
        assert_eq!(config_info.accepted_ibc_denoms.list_of_denoms, vec![
            "uhuahua".to_string(),
            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
            "random_denom2".to_string()
        ]);
        assert_eq!(config_info.next_history_id, 1);
        assert_eq!(config_info.oldest_history_id, 0);

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::ChangeMaxLength { length: 20 }
            ]
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        let info = mock_info(NEW_OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::ChangeMaxLength { length: 20 }
            ]
        );
        assert_eq!(execute_output.is_err(), false);

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
        let (
            mut app,
            _native_huahua,
            native_atom,
            price_oracle_smart_contract_addr
        ) = instantiate_necessary_for_tests();
        let info = mock_info(WALLET2, &vec![]);
        let execute_output = execute_function_price_oracle_test_feed_prices(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
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
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589999u64),
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

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
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "HELLO".to_string(),
                        name: "HELLO".to_string(),
                        ibc_denom: "HELLO".to_string(),
                        value_usdc_6_decimals: Uint128::new(111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1 as u64),
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidDenominationReceived".to_string()
        );

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
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1 as u64),
            }
        );
        assert_eq!(execute_output.is_err(), false);

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::AddDenoms { denoms: vec!["test_denom".to_string()] }
            ]
        );
        assert_eq!(execute_output.is_err(), false);

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
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1686589235u64 + 1 as u64),
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "SomeDenomsAreMissingInYourUpdate".to_string()
        );

        let info = mock_info(OWNER, &vec![]);
        let execute_output = execute_function_price_oracle_test_update_config(
            &mut app,
            &Addr::unchecked(price_oracle_smart_contract_addr.clone()),
            info.clone(),
            vec![
                UpdateConfigEnum::RemoveDenoms { denoms: vec!["test_denom".to_string()] }
            ]
        );
        assert_eq!(execute_output.is_err(), false);

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
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589236u64 + 1 as u64),
            }
        );
        assert_eq!(execute_output.is_err(), false);

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
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(119u128 + 1 as u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1 as u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589236u64 - 10 as u64),
            }
        );
        assert_eq!(
            execute_output.unwrap_err().source().unwrap().to_string(),
            "InvalidTimeForPriceReceived".to_string()
        );

        for i in 0..20 {
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
                    at_time: Timestamp::from_seconds(1676599999u64 + i as u64),
                }
            );
            assert_eq!(
                execute_output.is_err(),
                false
            );
        }

        let histo_prices_query_info = query_function_price_oracle_test_get_latest_historical_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
            None
        );
        assert_eq!(histo_prices_query_info.unwrap().len(), 10);

        let histo_prices_query_info = query_function_price_oracle_test_get_latest_historical_prices(
            &app,
            price_oracle_smart_contract_addr.clone(),
            None
        ).unwrap();
        assert_eq!(histo_prices_query_info[0].0, 22);
        assert!(histo_prices_query_info[0].1.at_time > histo_prices_query_info[1].1.at_time);

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
                        ibc_denom: "uhuahua".to_string(),
                        value_usdc_6_decimals: Uint128::new(999u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                        value_usdc_6_decimals: Uint128::new(14_444_444u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1776589999u64),
            }
        );
        assert_eq!(execute_output.is_err(), false);

        let histo_prices_query_info = query_function_price_oracle_test_get_latest_historical_prices(&app, price_oracle_smart_contract_addr.clone(), Some(2)).unwrap();
        let config_query_info = query_function_price_oracle_test_get_config_and_current_prices(&app, price_oracle_smart_contract_addr.clone());
        assert_eq!(histo_prices_query_info[0].1.at_time, config_query_info.current_prices.at_time);
        assert_eq!(histo_prices_query_info[0].1.prices, config_query_info.current_prices.prices);
        assert_eq!(histo_prices_query_info[0].1.prices, vec![
            OraclePrice {
                ticker: "HUAHUA".to_string(),
                name: "Chihuahua Chain".to_string(),
                ibc_denom: "uhuahua".to_string(),
                value_usdc_6_decimals: Uint128::new(999u128),
            },
            OraclePrice {
                ticker: "ATOM".to_string(),
                name: "Cosmos Hub".to_string(),
                ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                value_usdc_6_decimals: Uint128::new(14_444_444u128),
            },
        ]);

        let denom_current_price = query_function_price_oracle_test_get_usdc_price_from_amount_and_denom(&app, price_oracle_smart_contract_addr.clone(), Uint128::new(100_000u128), native_atom.clone());
        assert_eq!(denom_current_price, Uint128::new(1444444u128));

        let config_info = query_function_price_oracle_test_get_config_and_current_prices(
            &app,
            price_oracle_smart_contract_addr.clone()
        );
        assert_eq!(config_info.contract_owner, OWNER);
        assert_eq!(config_info.prices_feeder, FEEDER);
        assert_eq!(config_info.current_prices.prices, vec![
            OraclePrice {
                ticker: "HUAHUA".to_string(),
                name: "Chihuahua Chain".to_string(),
                ibc_denom: "uhuahua".to_string(),
                value_usdc_6_decimals: Uint128::new(999u128),
            },
            OraclePrice {
                ticker: "ATOM".to_string(),
                name: "Cosmos Hub".to_string(),
                ibc_denom: "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string(),
                value_usdc_6_decimals: Uint128::new(14_444_444u128),
            }
        ]);
        assert_eq!(config_info.current_prices.at_time, Timestamp::from_seconds(1776589999u64));
        assert_eq!(config_info.accepted_ibc_denoms.list_of_denoms, vec![
            "uhuahua".to_string(),
            "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2".to_string()
        ]);
        // Max length of 10
        assert_eq!(config_info.next_history_id, 24);
        assert_eq!(config_info.oldest_history_id, 13);

    }
}
