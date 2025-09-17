use agent_nra::query::QueryMsg;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        query: QueryMsg,
    }
}
