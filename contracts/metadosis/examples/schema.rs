use cosmwasm_schema::write_api;
use metadosis::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use metadosis::query::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
