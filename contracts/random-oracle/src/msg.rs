use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub random_value: Option<u64>,
}
#[cw_serde]
pub enum ExecuteMsg {
    SetRandom { random_value: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns a pseudo random value either previously supplied by `SetRandom`
    /// or depending on the current block number.
    ///  
    #[returns(RandomResponse)]
    RandomValue {
        from_range: u64,
        to_range: u64,
        /// Number of random values to return
        count_values: u64,
    },
}

#[cw_serde]
pub struct RandomResponse {
    pub random_values: Vec<u64>,
}
