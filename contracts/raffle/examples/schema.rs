use cosmwasm_schema::write_api;
use raffle::msg::{ExecuteMsg, InstantiateMsg};
use raffle::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
