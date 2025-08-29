use agent_registry::msg::{ExecuteMsg, InstantiateMsg};
use agent_registry::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
