use crate::error::Cw721ContractError;
use crate::state::{BURNER, CREATOR, MINTER};
use cosmwasm_std::{Addr, Api, DepsMut, Env, MessageInfo, Response, StdResult, Storage};
use cw_ownable::{Action, Ownership};

pub fn assert_minter(storage: &dyn Storage, sender: &Addr) -> Result<(), Cw721ContractError> {
    if MINTER.assert_owner(storage, sender).is_err() {
        return Err(Cw721ContractError::NotMinter {});
    }
    Ok(())
}

pub fn assert_burner(storage: &dyn Storage, sender: &Addr) -> Result<(), Cw721ContractError> {
    if BURNER.assert_owner(storage, sender).is_err() {
        return Err(Cw721ContractError::NotMinter {});
    }
    Ok(())
}

pub fn assert_creator(storage: &dyn Storage, sender: &Addr) -> Result<(), Cw721ContractError> {
    if CREATOR.assert_owner(storage, sender).is_err() {
        return Err(Cw721ContractError::NotCreator {});
    }
    Ok(())
}

// ------- helper cw721 functions -------
pub fn initialize_creator(
    storage: &mut dyn Storage,
    api: &dyn Api,
    creator: Option<&str>,
) -> StdResult<Ownership<Addr>> {
    CREATOR.initialize_owner(storage, api, creator)
}

pub fn initialize_minter(
    storage: &mut dyn Storage,
    api: &dyn Api,
    minter: Option<&str>,
) -> StdResult<Ownership<Addr>> {
    MINTER.initialize_owner(storage, api, minter)
}

pub fn initialize_burner(
    storage: &mut dyn Storage,
    api: &dyn Api,
    minter: Option<&str>,
) -> StdResult<Ownership<Addr>> {
    BURNER.initialize_owner(storage, api, minter)
}

pub fn update_minter_ownership(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    action: Action,
) -> Result<Response, Cw721ContractError> {
    let ownership = MINTER.update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::new()
        .add_attribute("update_minter_ownership", info.sender.to_string())
        .add_attributes(ownership.into_attributes()))
}

pub fn update_burner_ownership(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    action: Action,
) -> Result<Response, Cw721ContractError> {
    let ownership = BURNER.update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::new()
        .add_attribute("update_burner_ownership", info.sender.to_string())
        .add_attributes(ownership.into_attributes()))
}

pub fn update_creator_ownership(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    action: Action,
) -> Result<Response, Cw721ContractError> {
    let ownership = CREATOR.update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::new()
        .add_attribute("update_creator_ownership", info.sender.to_string())
        .add_attributes(ownership.into_attributes()))
}
