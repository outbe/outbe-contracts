use cosmwasm_schema::write_api;
use nod::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use nod::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
