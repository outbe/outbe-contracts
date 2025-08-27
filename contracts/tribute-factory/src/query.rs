use crate::state::CONFIG;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{entry_point, to_json_binary, Addr, Binary, Deps, Env, StdResult};
use outbe_utils::Base58Binary;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns config
    #[returns(ConfigResponse)]
    GetConfig {},
    /// Returns TEE encryption info
    #[returns(EncryptionInfoResponse)]
    EncryptionInfo {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub tribute_address: Option<Addr>,
}

#[cw_serde]
pub struct EncryptionInfoResponse {
    pub public_key: Base58Binary,
    pub salt: Base58Binary,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&ConfigResponse {
            tribute_address: config.tribute_address,
        }),
        QueryMsg::EncryptionInfo {} => {
            let result = config
                .tee_config
                .map(|it| EncryptionInfoResponse {
                    public_key: it.public_key,
                    salt: it.salt,
                })
                .unwrap_or(EncryptionInfoResponse {
                    public_key: Base58Binary::default(),
                    salt: Base58Binary::default(),
                });

            to_json_binary(&result)
        }
    }
}
