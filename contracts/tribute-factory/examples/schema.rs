use cosmwasm_schema::write_api;
use tribute_factory::msg::{ExecuteMsg, InstantiateMsg};
use tribute_factory::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
