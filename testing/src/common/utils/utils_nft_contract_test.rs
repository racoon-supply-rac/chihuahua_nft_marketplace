#[cfg(test)]
pub mod tests {
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{Addr, Empty, MessageInfo, Timestamp, Uint128};
    use cw721_base::MintMsg;
    use cw721_metadata_onchain;
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use cw_utils::Expiration;

    use cw2981_multiroyalties::msg::Cw2981QueryMsg;
    use cw2981_multiroyalties::{Metadata, Royalty};
    use nft_marketplace_utils::nft_collection::TokenId;

    use crate::common::utils::constants::OWNER;

    pub fn smart_contract_def_test_cw2981_multi() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw2981_multiroyalties::entry::execute,
            cw2981_multiroyalties::entry::instantiate,
            cw2981_multiroyalties::entry::query,
        );
        Box::new(contract)
    }

    pub fn smart_contract_def_test_cw721_metadata_onchain() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_metadata_onchain::entry::execute,
            cw721_metadata_onchain::entry::instantiate,
            cw721_metadata_onchain::entry::query,
        );
        Box::new(contract)
    }

    pub fn instantiate_smart_contract_test_cw2981_multi(
        app: &mut App,
        contract_code_id: u64,
    ) -> (Addr, u64) {
        let msg = cw721_base::msg::InstantiateMsg {
            name: "NftContractTest".to_string(),
            symbol: "NFT".to_string(),
            minter: OWNER.to_string(),
        };
        (
            app.instantiate_contract(
                contract_code_id,
                Addr::unchecked(OWNER),
                &msg,
                &[],
                "cw2981_base_code",
                None,
            )
            .unwrap(),
            contract_code_id,
        )
    }

    pub fn instantiate_smart_contract_test_cw721_metadata_onchain(app: &mut App) -> (Addr, u64) {
        let contract_code_id = app.store_code(smart_contract_def_test_cw721_metadata_onchain());
        let msg = cw721_base::msg::InstantiateMsg {
            name: "CW721_NFT_Contract".to_string(),
            symbol: "CW721".to_string(),
            minter: OWNER.to_string(),
        };
        (
            app.instantiate_contract(
                contract_code_id,
                Addr::unchecked(OWNER),
                &msg,
                &[],
                "cw_721",
                None,
            )
            .unwrap(),
            contract_code_id,
        )
    }

    // Execute Functions for CW2981 NFT contract
    pub fn cw2981_multi_test_exec_mint(
        app: &mut App,
        nft_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
        mint_to: String,
        royalty: Option<Vec<Royalty>>,
    ) -> AnyResult<AppResponse> {
        #[allow(clippy::init_numbered_fields)]
        let msg = cw2981_multiroyalties::ExecuteMsg::Mint {
            0: MintMsg {
                token_id,
                owner: mint_to,
                token_uri: None,
                extension: Some(Metadata {
                    description: Some("Description".into()),
                    name: Some("My Name Is".to_string()),
                    royalties: royalty,
                    ..Metadata::default()
                }),
            },
        };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    pub fn cw721_onchain_meta_test_exec_mint(
        app: &mut App,
        nft_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
        mint_to: String,
        _royalty: Option<Vec<Royalty>>,
    ) -> AnyResult<AppResponse> {
        let msg = cw721_base::msg::ExecuteMsg::<cw721_metadata_onchain::Metadata, Empty>::Mint(
            MintMsg::<cw721_metadata_onchain::Metadata> {
                token_id,
                owner: mint_to,
                token_uri: None,
                extension: cw721_metadata_onchain::Metadata {
                    description: Some("Description".into()),
                    name: Some("My Name Is".to_string()),
                    ..cw721_metadata_onchain::Metadata::default()
                },
            },
        );
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    pub fn cw2981_multi_test_exec_approve(
        app: &mut App,
        nft_contract_addr: &Addr,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
        expiration_seconds: u64,
    ) -> AnyResult<AppResponse> {
        pub type Extension = Option<Empty>;
        let msg = cw721_base::ExecuteMsg::<Extension, ()>::Approve {
            spender: nft_marketplace_contract_addr.to_string(),
            token_id,
            expires: Option::from(Expiration::AtTime(Timestamp::from_seconds(
                expiration_seconds,
            ))),
        };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    pub fn cw2981_multi_test_exec_revoke(
        app: &mut App,
        nft_contract_addr: &Addr,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
    ) -> AnyResult<AppResponse> {
        pub type Extension = Option<Empty>;
        let msg = cw721_base::ExecuteMsg::<Extension, ()>::Revoke {
            spender: nft_marketplace_contract_addr.to_string(),
            token_id,
        };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    pub fn cw2981_multi_test_exec_transfer_nft(
        app: &mut App,
        nft_contract_addr: &Addr,
        _nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
        recipient: String,
    ) -> AnyResult<AppResponse> {
        pub type Extension = Option<Empty>;
        let msg = cw721_base::ExecuteMsg::<Extension, ()>::TransferNft {
            recipient,
            token_id,
        };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    // Query Function NFT contract
    pub fn cw2981_multi_test_query_nft_info<T: Into<String> + cosmwasm_schema::serde::Serialize>(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
    ) -> cw721::NftInfoResponse<Metadata> {
        let msg: cw721::Cw721QueryMsg = cw721::Cw721QueryMsg::NftInfo { token_id };
        let result: cw721::NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(nft_collection_address, &msg)
            .unwrap();
        result
    }

    pub fn cw2981_multi_test_query_owner_of<T: Into<String> + cosmwasm_schema::serde::Serialize>(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
    ) -> cw721::OwnerOfResponse {
        let msg: cw721::Cw721QueryMsg = cw721::Cw721QueryMsg::OwnerOf {
            token_id,
            include_expired: None,
        };
        let result: cw721::OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(nft_collection_address, &msg)
            .unwrap();
        result
    }

    pub fn cw2981_multi_test_query_approvals<
        T: Into<String> + cosmwasm_schema::serde::Serialize,
    >(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
    ) -> cw721::ApprovalsResponse {
        let msg: cw721::Cw721QueryMsg = cw721::Cw721QueryMsg::Approvals {
            token_id,
            include_expired: None,
        };
        let result: cw721::ApprovalsResponse = app
            .wrap()
            .query_wasm_smart(nft_collection_address, &msg)
            .unwrap();
        result
    }

    pub fn cw2981_multi_test_query_royalty_info<
        T: Into<String> + cosmwasm_schema::serde::Serialize,
    >(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
        sale_price: Uint128,
    ) -> Vec<cw2981_multiroyalties::msg::RoyaltiesInfoResponse> {
        let msg: cw2981_multiroyalties::QueryMsg = cw2981_multiroyalties::QueryMsg::Extension {
            msg: Cw2981QueryMsg::RoyaltyInfo {
                token_id,
                sale_price,
            },
        };
        let result = app.wrap().query_wasm_smart(nft_collection_address, &msg);
        result.unwrap()
    }
}
