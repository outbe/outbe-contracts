use cosmwasm_schema::write_api;
use lysis::msg::{ExecuteMsg, InstantiateMsg};
use lysis::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
