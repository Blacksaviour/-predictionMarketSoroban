//! Unit tests for ReserveAccounting.

extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, I256};

use shared::constants::{COMMITTER, RECORDER, WARD};

use crate::{ReserveAccounting, ReserveAccountingClient};

fn wad(env: &Env, n: i128) -> I256 {
    I256::from_i128(env, n * 1_000_000_000_000_000_000)
}

fn setup(env: &Env) -> (Address, ReserveAccountingClient<'_>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let id = env.register(ReserveAccounting, (admin.clone(),));
    (admin, ReserveAccountingClient::new(env, &id))
}

#[test]
fn reserve_starts_empty() {
    let env = Env::default();
    let (_admin, reserve) = setup(&env);

    assert_eq!(reserve.total_reserve(), I256::from_i32(&env, 0));
    assert_eq!(reserve.committed_escrow(), I256::from_i32(&env, 0));
    assert_eq!(reserve.free_slack(), I256::from_i32(&env, 0));
}

#[test]
fn recorder_roles_are_distinct_from_the_committer_role() {
    let env = Env::default();
    let (admin, reserve) = setup(&env);
    let psm = Address::generate(&env);

    // The Peg Stability Module records movements.
    reserve.grant(&admin, &RECORDER, &psm);
    reserve.record_increase(&psm, &wad(&env, 1_000));
    assert_eq!(reserve.total_reserve(), wad(&env, 1_000));

    // But it cannot commit the escrow.
    assert!(!reserve.has(&COMMITTER, &psm));
}

#[test]
fn free_slack_is_the_uncommitted_remainder() {
    let env = Env::default();
    let (admin, reserve) = setup(&env);
    let psm = Address::generate(&env);
    let solvency = Address::generate(&env);

    reserve.grant(&admin, &RECORDER, &psm);
    reserve.grant(&admin, &COMMITTER, &solvency);

    reserve.record_increase(&psm, &wad(&env, 1_000));
    reserve.update_committed_escrow(&solvency, &wad(&env, 400));

    assert_eq!(reserve.total_reserve(), wad(&env, 1_000));
    assert_eq!(reserve.committed_escrow(), wad(&env, 400));
    assert_eq!(reserve.free_slack(), wad(&env, 600));
}

#[test]
#[should_panic]
fn escrow_cannot_exceed_the_reserve() {
    let env = Env::default();
    let (admin, reserve) = setup(&env);
    let solvency = Address::generate(&env);
    reserve.grant(&admin, &COMMITTER, &solvency);

    reserve.update_committed_escrow(&solvency, &wad(&env, 1));
}

#[test]
#[should_panic]
fn reserve_cannot_fall_below_the_committed_escrow() {
    let env = Env::default();
    let (admin, reserve) = setup(&env);
    let psm = Address::generate(&env);
    let solvency = Address::generate(&env);

    reserve.grant(&admin, &RECORDER, &psm);
    reserve.grant(&admin, &COMMITTER, &solvency);

    reserve.record_increase(&psm, &wad(&env, 1_000));
    reserve.update_committed_escrow(&solvency, &wad(&env, 400));

    // Only 600 of slack remains, so removing 601 must revert.
    reserve.record_decrease(&psm, &wad(&env, 601));
}

#[test]
fn ward_role_is_granted_to_the_deployer() {
    let env = Env::default();
    let (admin, reserve) = setup(&env);

    assert!(reserve.has(&WARD, &admin));
}
