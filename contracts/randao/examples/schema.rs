use cosmwasm_schema::write_api;

use randao::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use randao::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
