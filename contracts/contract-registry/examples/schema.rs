use cosmwasm_schema::write_api;

use contract_registry::msg::{ExecuteMsg, InstantiateMsg};
use contract_registry::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
