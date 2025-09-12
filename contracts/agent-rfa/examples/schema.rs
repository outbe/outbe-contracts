use agent_rfa::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use agent_rfa::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
