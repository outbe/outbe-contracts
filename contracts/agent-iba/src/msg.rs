use agent_common::types::{AgentInput, ExternalWallet};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum MigrateMsg {
    Migrate {},
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitAgent {
        id: String,
    },
    EditAgent {
        agent: Box<AgentInput>,
    },
    HoldAgent {
        address: String,
    },
    BanAgent {
        address: String,
    },
    ActivateAgent {
        address: String,
    },
    ResignAgent {},

    EditAdditionalWallets {
        additional_wallets: Option<Vec<ExternalWallet>>,
    },
}
