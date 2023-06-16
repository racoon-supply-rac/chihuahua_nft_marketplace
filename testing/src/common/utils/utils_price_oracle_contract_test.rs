#[cfg(test)]
pub mod tests {
    use anyhow::Result as AnyResult;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{Addr, Empty, MessageInfo, Timestamp, Uint128};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

    use general_utils::denominations::AcceptedDenominations;
    use oracle::msg::UpdateConfigEnum;
    use price_oracle_utils::oracle::{OraclePrice, OraclePrices};

    use crate::common::utils::constants::{FEEDER, OWNER};

    pub fn smart_contract_def_test_price_oracle() -> Box<dyn Contract<Empty>> {
        let smart_contract = ContractWrapper::new(
            oracle::contract::execute,
            oracle::contract::instantiate,
            oracle::contract::query,
        );
        Box::new(smart_contract)
    }

    pub fn instantiate_smart_contract_test_price_oracle(
        app: &mut App,
        accepted_ibc_denoms: AcceptedDenominations,
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_price_oracle());
        let init_msg = oracle::msg::InstantiateMsg {
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
        )
        .unwrap()
    }

    pub fn oracle_test_exec_feed_prices(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        new_prices: OraclePrices,
    ) -> AnyResult<AppResponse> {
        let msg = oracle::msg::ExecuteMsg::FeedPrices { prices: new_prices };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn oracle_test_exec_feed_prices_default(
        app: &mut App,
        price_oracle_smart_contract_addr: String,
        native_huahua: String,
        native_atom: String,
    ) -> AnyResult<AppResponse> {
        oracle_test_exec_feed_prices(
            app,
            &Addr::unchecked(price_oracle_smart_contract_addr),
            mock_info(FEEDER, &[]),
            OraclePrices {
                prices: vec![
                    OraclePrice {
                        ticker: "HUAHUA".to_string(),
                        name: "Chihuahua Chain".to_string(),
                        ibc_denom: native_huahua,
                        value_usdc_6_decimals: Uint128::new(119u128 + 1_u128),
                    },
                    OraclePrice {
                        ticker: "ATOM".to_string(),
                        name: "Cosmos Hub".to_string(),
                        ibc_denom: native_atom,
                        value_usdc_6_decimals: Uint128::new(13_555_111u128 + 1_u128),
                    },
                ],
                at_time: Timestamp::from_seconds(1676589235u64 + 1_u64),
            },
        )
    }

    pub fn oracle_test_exec_update_config(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        configs_to_update: Vec<UpdateConfigEnum>,
    ) -> AnyResult<AppResponse> {
        let msg = oracle::msg::ExecuteMsg::UpdateConfig {
            list_of_updates: configs_to_update,
        };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }
}
