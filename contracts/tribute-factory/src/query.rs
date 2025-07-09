use crate::state::TeeConfig;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, HexBinary};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns config
    #[returns(ConfigResponse)]
    GetConfig {},
    /// Returns TEE Ed25519 public key
    #[returns(PubkeyResponse)]
    Pubkey {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub tribute_address: Option<Addr>,
    pub tee_config: Option<TeeConfig>,
}

#[cw_serde]
pub struct PubkeyResponse {
    pub public_key: HexBinary,
}
