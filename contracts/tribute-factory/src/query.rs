use crate::state::CONFIG;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{entry_point, to_json_binary, Addr, Binary, Deps, Env, HexBinary, StdResult};

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
}

#[cw_serde]
pub struct PubkeyResponse {
    pub public_key: HexBinary,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&ConfigResponse {
            tribute_address: config.tribute_address,
        }),
        QueryMsg::Pubkey {} => to_json_binary(&PubkeyResponse {
            public_key: config
                .tee_config
                .map(|it| it.public_key)
                .unwrap_or_default(),
        }),
    }
}
