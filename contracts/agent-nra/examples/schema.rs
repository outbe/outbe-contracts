use agent_nra::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use agent_nra::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
