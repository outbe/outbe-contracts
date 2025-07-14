use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::MinterResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub mint: Option<MinterResponse>,
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Burn { amount: Uint128 },
    Mint { recipient: String, amount: Uint128 },
    UpdateMinter { new_minter: Option<String> },
    UpdateAdmin { new_admin: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw20::Balance)]
    Balance { address: String },
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    #[returns(cw20::MinterResponse)]
    Minter {},
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(CheckTicketResponse)]
    CheckTicket { ticket: String },
    #[returns(String)]
    Admin {},
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CheckTicketResponse {
    pub exists: bool,
}
