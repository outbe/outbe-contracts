use agent_common::msg::InstantiateMsg;
use agent_iba::msg::{ExecuteMsg, MigrateMsg};
use agent_iba::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
