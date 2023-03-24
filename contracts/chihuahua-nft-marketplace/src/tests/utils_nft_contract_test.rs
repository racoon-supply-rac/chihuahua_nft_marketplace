#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{Addr, Empty, MessageInfo, Timestamp, Uint128};
    use cw721_base::MintMsg;
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use cw_utils::Expiration;
    use cw2981_multiroyalties::{Metadata, Royalty};
    use cw2981_multiroyalties::msg::Cw2981QueryMsg;
    use nft_marketplace_utils::nft_collection::TokenId;
    use anyhow::Result as AnyResult;
    use crate::tests::tests::{OWNER};

    pub fn smart_contract_def_test_cw2981_multi() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw2981_multiroyalties::entry::execute,
            cw2981_multiroyalties::entry::instantiate,
            cw2981_multiroyalties::entry::query,
        );
        Box::new(contract)
    }

    pub fn instantiate_smart_contract_test_cw2981_multi(
        app: &mut App,
    ) -> Addr {
        let contract_code_id = app.store_code(smart_contract_def_test_cw2981_multi());
        let msg = cw721_base::msg::InstantiateMsg {
            name: "NftContractTest".to_string(),
            symbol: "NFT".to_string(),
            minter: OWNER.clone().to_string(),
        };
        app.instantiate_contract(
            contract_code_id,
            Addr::unchecked(OWNER),
            &msg,
            &[],
            "cw2981_base_code",
            None,
        )
            .unwrap()
    }

    // Execute Functions for CW2981 NFT contract
    pub fn execute_function_cw2981_multi_test_mint(
        app: &mut App,
        nft_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
        mint_to: String,
        royalty: Option<Vec<Royalty>>
    ) -> AnyResult<AppResponse> {
        let msg =  cw2981_multiroyalties::ExecuteMsg::Mint {
            0: MintMsg {
                token_id,
                owner: mint_to.clone().to_string(),
                token_uri: None,
                extension: Some(Metadata {
                    description: Some("Description".into()),
                    name: Some("My Name Is".to_string()),
                    royalties: royalty,
                    ..Metadata::default()
                })
            } };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_cw2981_multi_test_approve(
        app: &mut App,
        nft_contract_addr: &Addr,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
        expiration_seconds: u64
    ) -> AnyResult<AppResponse> {
        pub type Extension = Option<Empty>;
        let msg = cw721_base::ExecuteMsg::<Extension, ()>::Approve {
            spender: nft_marketplace_contract_addr.to_string(),
            token_id,
            expires: Option::from(Expiration::AtTime(Timestamp::from_seconds(expiration_seconds)))
        };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    pub fn execute_function_cw2981_multi_test_revoke(
        app: &mut App,
        nft_contract_addr: &Addr,
        nft_marketplace_contract_addr: &Addr,
        info: MessageInfo,
        token_id: String,
    ) -> AnyResult<AppResponse> {
        pub type Extension = Option<Empty>;
        let msg = cw721_base::ExecuteMsg::<Extension, ()>::Revoke {
            spender: nft_marketplace_contract_addr.to_string(),
            token_id
        };
        app.execute_contract(info.sender, nft_contract_addr.clone(), &msg, &[])
    }

    // Query Function NFT contract
    pub fn query_function_cw2981_multi_test_nft_info<T: Into<String> + cosmwasm_schema::serde::Serialize>(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
    ) -> cw721::NftInfoResponse<Metadata> {
        let msg: cw721::Cw721QueryMsg = cw721::Cw721QueryMsg::NftInfo { token_id: token_id.clone().to_string() };
        let result: cw721::NftInfoResponse<Metadata> =
            app.wrap().query_wasm_smart(nft_collection_address, &msg).unwrap();
        result
    }

    pub fn query_function_cw2981_multi_test_owner_of<T: Into<String> + cosmwasm_schema::serde::Serialize>(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
    ) -> cw721::OwnerOfResponse {
        let msg: cw721::Cw721QueryMsg = cw721::Cw721QueryMsg::OwnerOf { token_id: token_id.clone().to_string(), include_expired: None };
        let result: cw721::OwnerOfResponse =
            app.wrap().query_wasm_smart(nft_collection_address, &msg).unwrap();
        result
    }

    pub fn query_function_cw2981_multi_test_approvals<T: Into<String> + cosmwasm_schema::serde::Serialize>(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
    ) -> cw721::ApprovalsResponse {
        let msg: cw721::Cw721QueryMsg = cw721::Cw721QueryMsg::Approvals { token_id: token_id.clone().to_string(), include_expired: None };
        let result: cw721::ApprovalsResponse =
            app.wrap().query_wasm_smart(nft_collection_address, &msg).unwrap();
        result
    }

    pub fn query_function_cw2981_multi_test_royalty_info<T: Into<String> + cosmwasm_schema::serde::Serialize>(
        app: &App,
        nft_collection_address: T,
        token_id: TokenId,
        sale_price: Uint128
    ) -> Vec<cw2981_multiroyalties::msg::RoyaltiesInfoResponse> {
        let msg: cw2981_multiroyalties::QueryMsg = cw2981_multiroyalties::QueryMsg::Extension {
            msg: Cw2981QueryMsg::RoyaltyInfo {
                token_id,
                sale_price
            } };
        let result =
            app.wrap().query_wasm_smart(nft_collection_address, &msg);
        result.unwrap()
    }
}
