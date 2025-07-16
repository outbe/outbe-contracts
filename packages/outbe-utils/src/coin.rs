use cosmwasm_std::Coin;

pub const NATIVE_DENOM: &str = "coen";
pub const NATIVE_DENOM_UNIT: &str = "unit";

pub fn coens(amount: u128) -> Vec<Coin> {
    vec![coen(amount)]
}

pub fn coen(amount: u128) -> Coin {
    Coin::new(amount, NATIVE_DENOM)
}

pub fn units(amount: u128) -> Vec<Coin> {
    vec![unit(amount)]
}

pub fn unit(amount: u128) -> Coin {
    Coin::new(amount, NATIVE_DENOM_UNIT)
}
