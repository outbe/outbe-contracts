use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub creator: Option<String>,
    pub vector: Option<Addr>,
    pub tribute: Option<Addr>,
    pub nod: Option<Addr>,
    pub token_allocator: Option<Addr>,
    pub price_oracle: Option<Addr>,
    pub random_oracle: Option<Addr>,
    /// Deficit config where 1 mean 100%
    pub deficit: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    Execute { run_date: Option<Uint64> },
    // todo remove after demo
    BurnAll {},
}
