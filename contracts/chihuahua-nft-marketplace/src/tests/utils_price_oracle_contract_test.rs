#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{Addr, Empty, MessageInfo};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use general_utils::denominations::AcceptedDenominations;
    use price_oracle_utils::oracle::OraclePrices;
    use anyhow::Result as AnyResult;
    use crypto_assets_price_oracle::msg::UpdateConfigEnum;
    use crate::tests::tests::{FEEDER, OWNER};

    pub fn smart_contract_def_test_price_oracle() -> Box<dyn Contract<Empty>> {
        let smart_contract = ContractWrapper::new(
            crypto_assets_price_oracle::contract::execute,
            crypto_assets_price_oracle::contract::instantiate,
            crypto_assets_price_oracle::contract::query,
        );
        Box::new(smart_contract)
    }

    pub fn instantiate_smart_contract_test_price_oracle(
        app: &mut App,
        accepted_ibc_denoms: AcceptedDenominations
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_price_oracle());
        let init_msg = crypto_assets_price_oracle::msg::InstantiateMsg {
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

    pub fn execute_function_price_oracle_test_feed_prices(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        new_prices: OraclePrices
    ) -> AnyResult<AppResponse> {
        let msg = crypto_assets_price_oracle::msg::ExecuteMsg::FeedPrices { prices: new_prices };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_price_oracle_test_update_config(
        app: &mut App,
        contract_addr: &Addr,
        info: MessageInfo,
        configs_to_update: Vec<UpdateConfigEnum>
    ) -> AnyResult<AppResponse> {
        let msg = crypto_assets_price_oracle::msg::ExecuteMsg::UpdateConfig { list_of_updates: configs_to_update };
        app.execute_contract(info.sender, contract_addr.clone(), &msg, &[])
    }
}
