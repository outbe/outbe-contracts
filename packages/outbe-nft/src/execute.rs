use crate::error::Cw721ContractError;
use crate::state::{CREATOR, MINTER};
use cosmwasm_std::{Addr, Api, StdResult, Storage};
use cw_ownable::Ownership;

pub fn assert_minter(storage: &dyn Storage, sender: &Addr) -> Result<(), Cw721ContractError> {
    if MINTER.assert_owner(storage, sender).is_err() {
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
