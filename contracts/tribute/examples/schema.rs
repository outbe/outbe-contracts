use cosmwasm_schema::write_api;
use tribute::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use tribute::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
