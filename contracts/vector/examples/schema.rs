use cosmwasm_schema::write_api;
use vector::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use vector::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
