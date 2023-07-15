#[cfg(test)]
pub mod tests {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{coin, Addr, Coin};
    use cw_multi_test::{App, BankSudo, SudoMsg};

    use general_utils::denominations::AcceptedDenominations;

    use crate::common::utils::constants::{IBC_ATOM, INVALID_REWARD_TOKEN, OWNER, REWARD_TOKEN, UHUAHUA, WALLET2, WALLET3, WALLET4, WALLET5};
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
        pub reward_token: String,
        pub invalid_reward_token: String,
    }

    pub fn instantiate_necessary_for_tests() -> (App, InitNecessaryOutcome) {
        let mut app: App = mock_app();
        let oracle_contract_addr = instantiate_smart_contract_test_price_oracle(
            &mut app,
            AcceptedDenominations {
                list_of_denoms: vec![UHUAHUA.to_string(), IBC_ATOM.to_string()],
            },
        );

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
                REWARD_TOKEN.to_string(),
            );

        let contract_code_id = app.store_code(smart_contract_def_test_cw2981_multi());
        let (cw2981_base_smart_contract_address1, code_id_nft1) =
            instantiate_smart_contract_test_cw2981_multi(&mut app, contract_code_id.clone());
        let (cw2981_base_smart_contract_address2, _code_id_nft2) =
            instantiate_smart_contract_test_cw2981_multi(&mut app, contract_code_id);

        // Send reward tokens to the marketplace to distribute
        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: nft_marketplace_smart_contract_address.to_string(),
                amount: vec![coin(100_000_000_000_000_000_000u128, REWARD_TOKEN.to_string())],
            }
        }))
            .unwrap();

        // Mint some coins for purchases and reward tokens
        for i in vec![OWNER, WALLET2, WALLET3, WALLET4, WALLET5].iter() {
            for tok in vec![UHUAHUA, IBC_ATOM, REWARD_TOKEN, INVALID_REWARD_TOKEN].iter() {
                app.sudo(SudoMsg::Bank({
                    BankSudo::Mint {
                        to_address: i.to_string(),
                        amount: vec![coin(100_000_000_000_000_000_000u128, tok.to_string())],
                    }
                }))
                    .unwrap();
            }
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
                reward_token: REWARD_TOKEN.to_string(),
                invalid_reward_token: INVALID_REWARD_TOKEN.to_string(),
            },
        )
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
