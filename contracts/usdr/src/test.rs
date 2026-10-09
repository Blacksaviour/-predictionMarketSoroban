//! Unit tests for the USDR token.
//!
//! The Foundry suite under `tests/` in the original repository is reproduced here as Soroban unit tests that run
//! against the in-process host (`soroban_sdk::testutils`).

extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, I256};

use shared::constants::{BURNER, WARD};

use crate::{Usdr, UsdrClient};

fn wad(env: &Env, n: i128) -> I256 {
    I256::from_i128(env, n * 1_000_000_000_000_000_000)
}

/// Deploys USDR with a fresh admin and returns the client.
fn setup(env: &Env) -> (Address, UsdrClient<'_>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register(Usdr, (admin.clone(),));
    (admin, UsdrClient::new(env, &contract_id))
}

#[test]
fn metadata_matches_the_solidity_token() {
    let env = Env::default();
    let (_admin, usdr) = setup(&env);

    assert_eq!(usdr.symbol(), soroban_sdk::String::from_str(&env, "USDR"));
    assert_eq!(
        usdr.name(),
        soroban_sdk::String::from_str(&env, "Rain Dollar")
    );
    assert_eq!(usdr.decimals(), 18);
    assert_eq!(usdr.total_supply(), I256::from_i32(&env, 0));
}

#[test]
fn ward_can_mint_and_burn() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);
    let user = Address::generate(&env);

    usdr.mint(&admin, &user, &wad(&env, 100));
    assert_eq!(usdr.balance_of(&user), wad(&env, 100));
    assert_eq!(usdr.total_supply(), wad(&env, 100));

    // A ward is not a burner, so burning from another account spends the caller's allowance.
    usdr.approve(&user, &admin, &wad(&env, 40));
    usdr.burn(&admin, &user, &wad(&env, 40));
    assert_eq!(usdr.balance_of(&user), wad(&env, 60));
    assert_eq!(usdr.total_supply(), wad(&env, 60));
}

#[test]
fn transfer_moves_balances() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    usdr.mint(&admin, &alice, &wad(&env, 10));
    usdr.transfer(&alice, &bob, &wad(&env, 3));

    assert_eq!(usdr.balance_of(&alice), wad(&env, 7));
    assert_eq!(usdr.balance_of(&bob), wad(&env, 3));
}

#[test]
fn transfer_from_spends_allowance() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    usdr.mint(&admin, &alice, &wad(&env, 10));
    usdr.approve(&alice, &bob, &wad(&env, 4));

    usdr.transfer_from(&bob, &alice, &bob, &wad(&env, 4));

    assert_eq!(usdr.balance_of(&bob), wad(&env, 4));
    assert_eq!(usdr.allowance(&alice, &bob), I256::from_i32(&env, 0));
}

#[test]
#[should_panic]
fn mint_requires_the_ward_role() {
    let env = Env::default();
    let (_admin, usdr) = setup(&env);
    let rando = Address::generate(&env);
    let user = Address::generate(&env);

    // No role granted to `rando`: the call must trap with `NotAuthorized`.
    usdr.mint(&rando, &user, &wad(&env, 1));
}

#[test]
#[should_panic]
fn mint_rejects_zero_amount() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);
    let user = Address::generate(&env);

    usdr.mint(&admin, &user, &I256::from_i32(&env, 0));
}

#[test]
#[should_panic]
fn burn_rejects_zero_amount() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);

    usdr.burn(&admin, &admin, &I256::from_i32(&env, 0));
}

#[test]
fn ward_role_is_granted_to_the_deployer() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);

    // WARD administers itself and BURNER; the deployer starts with WARD.
    assert!(usdr.has(&WARD, &admin));
    assert!(!usdr.has(&BURNER, &admin));
}

#[test]
fn a_burner_may_burn_without_an_allowance() {
    let env = Env::default();
    let (admin, usdr) = setup(&env);
    let burner = Address::generate(&env);
    let user = Address::generate(&env);

    usdr.grant(&admin, &BURNER, &burner);
    usdr.mint(&admin, &user, &wad(&env, 10));

    // No allowance needed: the burner holds the dedicated role.
    usdr.burn(&burner, &user, &wad(&env, 4));

    assert_eq!(usdr.balance_of(&user), wad(&env, 6));
}
