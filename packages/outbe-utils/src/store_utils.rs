use cosmwasm_std::{StdError, StdResult};

/// This is a helper function that we are using when
/// deserializing a data serialized by the `PrimaryKey` trait.
pub fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}
