#[cfg(test)]
pub mod tests {
    use anyhow::Result as AnyResult;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{coin, Addr, Coin, Empty, MessageInfo, StdResult, Uint128, to_binary, WasmMsg};
    use cw20::{BalanceResponse, Cw20Coin};
    use cw_multi_test::{App, AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

    use general_utils::denominations::AcceptedDenominations;

    use crate::common::utils::constants::{
        IBC_ATOM, OWNER, UHUAHUA, WALLET2, WALLET3, WALLET4, WALLET5,
    };
    use crate::common::utils::utils_marketplace_contract_test::tests::instantiate_smart_contract_test_nft_marketplace;
    use crate::common::utils::utils_nft_contract_test::tests::{
        instantiate_smart_contract_test_cw2981_multi, smart_contract_def_test_cw2981_multi,
    };
    use crate::common::utils::utils_price_oracle_contract_test::tests::{
        instantiate_smart_contract_test_price_oracle, oracle_test_exec_feed_prices_default,
    };

    pub fn mock_app() -> App {
        App::default()
    }

    #[cw_serde]
    pub struct InitNecessaryOutcome {
        pub native_huahua: String,
        pub native_atom: String,
        pub nft_marketplace_smart_contract_addr: String,
        pub cw2981_nft_contract_addr1: String,
        pub cw2981_nft_contract_addr2: String,
        pub cw2981_nft_code_id: u64,
        pub price_oracle_contract_addr: String,
        pub cw20_reward_token: String,
        pub cw20_invalid_reward_token: String,
    }

    pub fn instantiate_necessary_for_tests() -> (App, InitNecessaryOutcome) {
        let mut app: App = mock_app();
        let oracle_contract_addr = instantiate_smart_contract_test_price_oracle(
            &mut app,
            AcceptedDenominations {
                list_of_denoms: vec![UHUAHUA.to_string(), IBC_ATOM.to_string()],
            },
        );

        let reward_token = instantiate_cw20(&mut app);
        let invalid_reward_token = instantiate_cw20(&mut app);

        let _ = oracle_test_exec_feed_prices_default(
            &mut app,
            oracle_contract_addr.to_string(),
            UHUAHUA.to_string(),
            IBC_ATOM.to_string(),
        );

        let nft_marketplace_smart_contract_address: Addr =
            instantiate_smart_contract_test_nft_marketplace(
                &mut app,
                AcceptedDenominations {
                    list_of_denoms: vec![UHUAHUA.to_string(), IBC_ATOM.to_string()],
                },
                oracle_contract_addr.to_string(),
                reward_token.to_string(),
            );

        let contract_code_id = app.store_code(smart_contract_def_test_cw2981_multi());
        let (cw2981_base_smart_contract_address1, code_id_nft1) =
            instantiate_smart_contract_test_cw2981_multi(&mut app, contract_code_id);
        let (cw2981_base_smart_contract_address2, _code_id_nft2) =
            instantiate_smart_contract_test_cw2981_multi(&mut app, contract_code_id);

        // Mint some coins for purchases
        for i in vec![OWNER, WALLET2, WALLET3, WALLET4, WALLET5].iter() {
            app.sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: i.to_string(),
                    amount: vec![coin(100_000_000_000_000_000_000u128, UHUAHUA.to_string())],
                }
            }))
            .unwrap();
            app.sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: i.to_string(),
                    amount: vec![coin(100_000_000_000_000_000_000u128, IBC_ATOM.to_string())],
                }
            }))
            .unwrap();
        }

        // Send reward tokens to the contract and users
        for i in vec![
            OWNER,
            WALLET2,
            WALLET3,
            WALLET4,
            WALLET5,
            nft_marketplace_smart_contract_address.as_ref(),
        ]
        .iter()
        {
            let info = mock_info(OWNER, &[]);
            let execute_output = execute_function_cw20_test_transfer_cw20(
                &mut app,
                &Addr::unchecked(i.to_string()),
                Addr::unchecked(reward_token.clone()),
                info.clone(),
                Uint128::new(100_000_000_000u128),
            );
            assert!(execute_output.is_ok(), "{}", false);
            let execute_output = execute_function_cw20_test_transfer_cw20(
                &mut app,
                &Addr::unchecked(i.to_string()),
                Addr::unchecked(invalid_reward_token.clone()),
                info,
                Uint128::new(100_000_000_000u128),
            );
            assert!(execute_output.is_ok(), "{}", false);
        }

        (
            app,
            InitNecessaryOutcome {
                native_huahua: UHUAHUA.to_string(),
                native_atom: IBC_ATOM.to_string(),
                nft_marketplace_smart_contract_addr: nft_marketplace_smart_contract_address
                    .to_string(),
                cw2981_nft_contract_addr1: cw2981_base_smart_contract_address1.to_string(),
                cw2981_nft_contract_addr2: cw2981_base_smart_contract_address2.to_string(),
                cw2981_nft_code_id: code_id_nft1,
                price_oracle_contract_addr: oracle_contract_addr.to_string(),
                cw20_reward_token: reward_token.to_string(),
                cw20_invalid_reward_token: invalid_reward_token.to_string(),
            },
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
                amount: Uint128::new(1_000_000_000_000_000u128),
            }],
            mint: None,
            marketing: None,
        };
        app.instantiate_contract(cw20_id, Addr::unchecked(OWNER), &msg, &[], "cw20", None)
            .unwrap()
    }

    // General Execute functions
    pub fn execute_function_cw20_test_transfer_cw20(
        app: &mut App,
        recipient: &Addr,
        cw20_addr: Addr,
        info: MessageInfo,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        let msg = cw20::Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        };
        app.execute_contract(info.sender, cw20_addr, &msg, &[])
    }

    // General Query functions
    pub fn query_account_cw20_balance<T: Into<String>>(
        app: &App,
        contract_addr: T,
        address: Addr,
    ) -> StdResult<BalanceResponse> {
        let msg = cw20_base::msg::QueryMsg::Balance {
            address: address.to_string(),
        };
        let result = app.wrap().query_wasm_smart(contract_addr, &msg);
        result
    }

    pub fn query_account_native_denom_balance<T: Into<String>>(
        app: &App,
        contract_addr: T,
        denom: String,
    ) -> Coin {
        let result = app.wrap().query_balance(contract_addr, denom).unwrap();
        result
    }
}
