use agent_common::msg::InstantiateMsg;
use agent_cra::msg::{ExecuteMsg, MigrateMsg};
use agent_cra::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
