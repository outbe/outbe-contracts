use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub mint: Option<cw20::MinterResponse>,
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Burn {
        amount: Uint128,
    },
    Mint {
        recipient: String,
        amount: Uint128,
    },
    UpdateMinter {
        new_minter: Option<String>,
    },
    UpdateAdmin {
        new_admin: String,
    },

    #[cfg(feature = "demo")]
    MintNative {
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw20::BalanceResponse)]
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

#[cw_serde]
pub struct CheckTicketResponse {
    pub exists: bool,
}
