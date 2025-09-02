use crate::types::{AccountInput, AccountStatus, AgentInput};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub threshold: Option<u8>,
    pub paused: Option<bool>,
}

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateAgent {
        agent: AgentInput,
    },
    UpdateAgent {
        id: String,
        agent: AgentInput,
    },
    VoteAgent {
        id: String,
        approve: bool,
        reason: Option<String>,
    },

    UpdateAccount {
        account: AccountInput,
    },
    ChangeAccountStatus {
        address: Addr,
        status: AccountStatus,
        reason: Option<String>,
    },
}
