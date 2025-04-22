use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::App;

pub const NATIVE_DENOM: &str = "qnc";

pub const INITIAL_BALANCE: Uint128 = Uint128::new(10_000_000_u128);

#[allow(dead_code)]
pub struct AppConfig {
    pub owner_addr: Addr,
    pub user_addr: Addr,
}

#[allow(dead_code)]
pub struct DeployedContract {
    pub address: Addr,
    pub code_id: u64,
}

pub fn setup_test_env() -> (App, AppConfig) {
    let mut app = App::default();

    let owner_addr = app.api().addr_make("OWNER");
    let user_addr = app.api().addr_make("USER");

    println!("💰 Add native balance to the owner");
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(
                storage,
                &owner_addr,
                vec![Coin::new(INITIAL_BALANCE, NATIVE_DENOM)],
            )
            .unwrap();
    });

    (
        app,
        AppConfig {
            owner_addr,
            user_addr,
        },
    )
}
