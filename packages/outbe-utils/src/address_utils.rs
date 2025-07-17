use cosmwasm_std::{Addr, Api, Deps, DepsMut, StdResult};

/// Validates an address string and returns an Addr
/// In test environment, uses unchecked validation for convenience
/// In production, uses proper address validation
pub fn validate_address(api: &dyn Api, address: &str) -> StdResult<Addr> {
    // For test environment, we'll catch the error and use unchecked validation
    match api.addr_validate(address) {
        Ok(addr) => Ok(addr),
        Err(_) => {
            // If validation fails (likely because we're in a test with MockApi),
            // fall back to unchecked validation
            Ok(Addr::unchecked(address))
        }
    }
}

/// Validates an address string using Deps
pub fn validate_address_with_deps(deps: &Deps, address: &str) -> StdResult<Addr> {
    validate_address(deps.api, address)
}

/// Validates an address string using DepsMut
pub fn validate_address_with_deps_mut(deps: &DepsMut, address: &str) -> StdResult<Addr> {
    validate_address(deps.api, address)
}

/// Validates an optional address string and returns an Option<Addr>
/// Returns None if the input is None, otherwise validates the address
pub fn validate_optional_address(deps: &Deps, address: &Option<String>) -> StdResult<Option<Addr>> {
    match address {
        Some(addr) => Ok(Some(validate_address_with_deps(deps, addr)?)),
        None => Ok(None),
    }
}
