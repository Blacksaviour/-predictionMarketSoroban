//! Unit tests for the PriceCurve auction calculator.

extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env, I256};

use shared::types::FileValue;

use crate::{PriceCurve, PriceCurveClient};

fn setup(env: &Env) -> (Address, PriceCurveClient<'_>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let id = env.register(PriceCurve, (admin.clone(),));
    (admin, PriceCurveClient::new(env, &id))
}

#[test]
fn tau_defaults_to_zero() {
    let env = Env::default();
    let (_admin, curve) = setup(&env);

    assert_eq!(curve.tau(), I256::from_i32(&env, 0));
}

#[test]
fn price_is_flat_before_tau_is_set() {
    let env = Env::default();
    let (_admin, curve) = setup(&env);

    // With tau == 0 every elapsed duration is at or beyond the lifetime, so the price is zero.
    let top = I256::from_i128(&env, 1_000);
    assert_eq!(
        curve.price(&top, &I256::from_i32(&env, 0)),
        I256::from_i32(&env, 0)
    );
}

#[test]
fn price_declines_linearly_to_zero() {
    let env = Env::default();
    let (admin, curve) = setup(&env);

    // A 100 second lifetime.
    curve.file(
        &admin,
        &symbol_short!("tau"),
        &FileValue::Num(I256::from_i128(&env, 100)),
    );

    let top = I256::from_i128(&env, 1_000);

    // At t = 0 the price is the full starting price.
    assert_eq!(curve.price(&top, &I256::from_i32(&env, 0)), top);

    // Half way through, half the price remains.
    assert_eq!(
        curve.price(&top, &I256::from_i32(&env, 50)),
        I256::from_i128(&env, 500)
    );

    // Three quarters through.
    assert_eq!(
        curve.price(&top, &I256::from_i32(&env, 75)),
        I256::from_i128(&env, 250)
    );

    // At exactly tau the price reaches zero, and never goes negative.
    assert_eq!(
        curve.price(&top, &I256::from_i128(&env, 100)),
        I256::from_i32(&env, 0)
    );
    assert_eq!(
        curve.price(&top, &I256::from_i128(&env, 1_000)),
        I256::from_i32(&env, 0)
    );
}

#[test]
#[should_panic]
fn unknown_parameters_are_rejected() {
    let env = Env::default();
    let (admin, curve) = setup(&env);

    curve.file(
        &admin,
        &symbol_short!("nope"),
        &FileValue::Num(I256::from_i32(&env, 1)),
    );
}
