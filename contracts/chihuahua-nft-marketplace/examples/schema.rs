use cosmwasm_schema::write_api;
use chihuahua_nft_marketplace::msg::{InstantiateMsg, ExecuteMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
