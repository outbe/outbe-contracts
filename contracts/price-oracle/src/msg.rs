use crate::types::{DayType};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use outbe_utils::denom::Denom;

#[cw_serde]
pub struct InstantiateMsg {
    pub creator: Option<String>
}

#[cw_serde]
pub enum ExecuteMsg {

    // New methods
    AddTokenPair {
        token1: Denom,
        token2: Denom,
    },
    RemoveTokenPair {
        token1: Denom,
        token2: Denom,
    },
    UpdatePrice {
        token1: Denom,
        token2: Denom,
        price: Decimal,
        open: Option<Decimal>,
        high: Option<Decimal>,
        low: Option<Decimal>,
        close: Option<Decimal>,
    },
    SetDayType {
        token1: Denom,
        token2: Denom,
        day_type: DayType,
    },
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}
