use cosmwasm_schema::write_api;

use token_allocator::msg::{ExecuteMsg, InstantiateMsg};
use token_allocator::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
