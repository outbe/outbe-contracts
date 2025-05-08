use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// This is an exact copy of `CustomMsg`, since implementing a trait for a type from another crate is not possible.
///
/// Possible:
/// `impl<T> Cw721CustomMsg for Option<T> where T: Cw721CustomMsg {}`
///
/// Not possible:
/// `impl<T> CustomMsg for Option<T> where T: CustomMsg {}`
///
/// This will be removed once the `CustomMsg` trait is moved to the `cosmwasm_std` crate: https://github.com/CosmWasm/cosmwasm/issues/2056
pub trait Cw721CustomMsg: Serialize + Clone + Debug + PartialEq + JsonSchema {}

impl Cw721CustomMsg for Empty {}
impl<T> Cw721CustomMsg for Option<T> where T: Cw721CustomMsg {}

pub trait Cw721State: Serialize + DeserializeOwned + Clone + Debug {}

impl Cw721State for Empty {}
impl<T> Cw721State for Option<T> where T: Cw721State {}

pub trait Cw721CollectionConfig: Serialize + DeserializeOwned + Clone + Debug {}

impl Cw721CollectionConfig for Empty {}

impl<T> Cw721CollectionConfig for Option<T> where T: Cw721CollectionConfig {}
