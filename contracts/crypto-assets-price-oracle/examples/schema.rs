use cosmwasm_schema::write_api;
use crypto_assets_price_oracle::msg::{InstantiateMsg, ExecuteMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
