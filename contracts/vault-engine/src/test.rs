//! Unit tests for the VaultEngine core ledger.

extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol, I256};

use shared::constants::{ilk_id, ray as ray_unit, wad as wad_unit, WARD};
use shared::types::FileValue;

use crate::{VaultEngine, VaultEngineClient};

fn zero(env: &Env) -> I256 {
    I256::from_i32(env, 0)
}

fn wad(env: &Env, n: i128) -> I256 {
    wad_unit(env).mul(&I256::from_i128(env, n))
}

/// Deploys the ledger with a fresh ward and registers a RAIN ilk with a 400% ratio and 1:1 price factor.
fn setup(env: &Env) -> (Address, VaultEngineClient<'_>, BytesN<32>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register(VaultEngine, (admin.clone(),));
    let engine = VaultEngineClient::new(env, &contract_id);

    let rain = ilk_id(env, "RAIN");
    engine.init(&admin, &rain);

    // spot = 1 / 4 = 0.25 (the PriceConverter would normally compute this).
    let spot = ray_unit(env).div(&I256::from_i32(env, 4));
    engine.file_ilk(
        &admin,
        &rain,
        &symbol_short!("spot"),
        &FileValue::Num(spot.clone()),
    );
    assert_eq!(engine.ilks(&rain).spot, spot);

    // `line` and `globalLine` are expressed in rad (wad * ray), so 1,000,000 USDR of headroom is
    // 1e24 * 1e27. Both ceilings default to zero in the Solidity original and must be set at deployment.
    let headroom = wad_unit(env)
        .mul(&I256::from_i128(env, 1_000_000))
        .mul(&ray_unit(env));
    engine.file_ilk(
        &admin,
        &rain,
        &symbol_short!("line"),
        &FileValue::Num(headroom.clone()),
    );
    engine.file(
        &admin,
        &Symbol::new(env, "globalLine"),
        &FileValue::Num(headroom),
    );
    engine.file_ilk(
        &admin,
        &rain,
        &symbol_short!("dust"),
        &FileValue::Num(zero(env)),
    );

    (admin, engine, rain)
}

#[test]
fn new_ilk_starts_at_ray() {
    let env = Env::default();
    let (_admin, engine, rain) = setup(&env);

    assert_eq!(engine.ilks(&rain).rate, ray_unit(&env));
    assert_eq!(engine.live(), 1);
}

#[test]
fn open_assigns_sequential_ids_and_ownership() {
    let env = Env::default();
    let (_admin, engine, rain) = setup(&env);
    let user = Address::generate(&env);

    let first = engine.open(&rain, &user);
    let second = engine.open(&rain, &user);

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(engine.vault_count(), 2);
    assert_eq!(engine.owner_of(&first), Some(user.clone()));
    assert_eq!(engine.ilk_of(&first), rain);
}

#[test]
fn frob_locks_collateral_and_draws_debt_within_the_ratio() {
    let env = Env::default();
    let (admin, engine, rain) = setup(&env);
    let user = Address::generate(&env);
    let vault_id = engine.open(&rain, &user);

    // Fund the user's free collateral through the adapter path (slip is ward gated).
    engine.slip(&admin, &rain, &user, &wad(&env, 100));

    // Lock 100 RAIN and draw 20 USDR: at a 25% factor that is exactly safe (100 * 0.25 = 25 >= 20).
    let collateral = wad(&env, 100);
    let amount = wad(&env, 20);
    engine.frob(&user, &vault_id, &user, &user, &collateral, &amount);

    let urn = engine.urns(&vault_id);
    assert_eq!(urn.ink, wad(&env, 100));
    assert_eq!(urn.art, wad(&env, 20));
    assert_eq!(engine.debt(), amount.mul(&ray_unit(&env)));
}

#[test]
#[should_panic]
fn frob_reverts_when_the_vault_is_unsafe() {
    let env = Env::default();
    let (admin, engine, rain) = setup(&env);
    let user = Address::generate(&env);
    let vault_id = engine.open(&rain, &user);

    engine.slip(&admin, &rain, &user, &wad(&env, 100));

    // 40 USDR against 100 RAIN needs a 160% ratio: above the 25% factor, so `tab > ink * spot`.
    let amount = wad(&env, 40);
    engine.frob(&user, &vault_id, &user, &user, &amount, &amount);
}

#[test]
#[should_panic]
fn frob_reverts_after_cage() {
    let env = Env::default();
    let (admin, engine, rain) = setup(&env);
    let user = Address::generate(&env);
    let vault_id = engine.open(&rain, &user);

    engine.slip(&admin, &rain, &user, &wad(&env, 100));
    engine.cage(&admin);

    let amount = wad(&env, 20);
    engine.frob(&user, &vault_id, &user, &user, &amount, &amount);
}

#[test]
fn grab_seizes_positions_and_books_bad_debt() {
    let env = Env::default();
    let (admin, engine, rain) = setup(&env);
    let user = Address::generate(&env);
    let vault_id = engine.open(&rain, &user);

    engine.slip(&admin, &rain, &user, &wad(&env, 100));
    let collateral = wad(&env, 100);
    let amount = wad(&env, 20);
    engine.frob(&user, &vault_id, &user, &user, &collateral, &amount);

    let sink = Address::generate(&env);
    let neg_collateral = shared::math::neg(&env, &collateral);
    let negative = shared::math::neg(&env, &amount);
    engine.grab(&admin, &vault_id, &user, &sink, &neg_collateral, &negative);

    // The collateral and the normalized debt are both removed from the vault.
    let urn = engine.urns(&vault_id);
    assert_eq!(urn.ink, zero(&env));
    assert_eq!(urn.art, zero(&env));

    // The seized debt is moved to the balance sheet as bad debt: `sin -= dtab` with a negative `dtab`,
    // so the sink's bad debt grows by the full tab amount.
    assert_eq!(engine.sin(&sink), amount.mul(&ray_unit(&env)));
}

#[test]
fn ward_role_is_granted_to_the_deployer() {
    let env = Env::default();
    let (admin, engine, _rain) = setup(&env);

    assert!(engine.has(&WARD, &admin));
}

#[test]
fn hope_and_nope_toggle_operator_permission() {
    let env = Env::default();
    let (_admin, engine, _rain) = setup(&env);
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    assert_eq!(engine.can(&owner, &operator), 0);
    engine.hope(&owner, &operator);
    assert_eq!(engine.can(&owner, &operator), 1);
    engine.nope(&owner, &operator);
    assert_eq!(engine.can(&owner, &operator), 0);
}

#[test]
fn parameters_are_settable_through_file() {
    let env = Env::default();
    let (admin, engine, rain) = setup(&env);

    let global_line = wad(&env, 500);
    engine.file(
        &admin,
        &Symbol::new(&env, "globalLine"),
        &FileValue::Num(global_line.clone()),
    );

    assert_eq!(engine.global_line(), global_line);
    assert_eq!(engine.ilks(&rain).rate, ray_unit(&env));
}
