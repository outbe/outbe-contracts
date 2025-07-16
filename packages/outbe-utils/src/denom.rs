use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub enum Denom {
    Native(String),
    Cw20(Addr),
    // TODO add native
}
