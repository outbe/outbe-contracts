use cosmwasm_schema::write_api;

use price_oracle::msg::{ExecuteMsg, InstantiateMsg};
use price_oracle::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
