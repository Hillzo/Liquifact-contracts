use crate::{
    CollateralClearedEvt, CollateralRecordedEvt, EscrowError, LiquifactEscrow,
    LiquifactEscrowClient,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    Address, Env, Event,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const AMOUNT: i128 = 10_000_0000000;
const PLEDGE: i128 = 5_000_0000000;

fn setup(env: &Env) -> (Address, Address, LiquifactEscrowClient<'_>) {
    let sme = Address::generate(env);
    let id = env.register(LiquifactEscrow, ());
    let client = LiquifactEscrowClient::new(env, &id);
    client.init(&symbol_short!("INV001"), &sme, &AMOUNT, &800i64, &1000u64);
    (sme, id, client)
}

// ---------------------------------------------------------------------------
// record → get → clear happy path
// ---------------------------------------------------------------------------

#[test]
fn test_record_then_clear_removes_pledge() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    assert!(client.get_sme_collateral_commitment().is_some());

    client.clear_sme_collateral_commitment();
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ---------------------------------------------------------------------------
// Clear without prior record → NoCollateralToClear
// ---------------------------------------------------------------------------

#[test]
fn test_clear_without_record_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    let result = client.try_clear_sme_collateral_commitment();
    assert_eq!(result, Err(Ok(EscrowError::NoCollateralToClear)));
}

// ---------------------------------------------------------------------------
// Non-SME caller is rejected
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_clear_non_sme_caller_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let sme = Address::generate(&env);
    let id = env.register(LiquifactEscrow, ());
    let client = LiquifactEscrowClient::new(&env, &id);
    client.init(&symbol_short!("INV002"), &sme, &AMOUNT, &800i64, &1000u64);
    client.record_sme_collateral_commitment(&PLEDGE);

    // Provide empty auth set: require_auth on sme_address will panic.
    env.set_auths(&[]);
    client.clear_sme_collateral_commitment();
}

// ---------------------------------------------------------------------------
// CollateralClearedEvt payload (using to_xdr comparison)
// ---------------------------------------------------------------------------

#[test]
fn test_clear_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    // env.events().all() reflects the LAST call's events in the test env.
    client.clear_sme_collateral_commitment();

    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![CollateralClearedEvt {
            invoice_id: symbol_short!("INV001"),
            amount: PLEDGE,
        }
        .to_xdr(&env, &id)]
    );
}

// ---------------------------------------------------------------------------
// CollateralRecordedEvt payload
// ---------------------------------------------------------------------------

#[test]
fn test_record_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);

    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![CollateralRecordedEvt {
            invoice_id: symbol_short!("INV001"),
            amount: PLEDGE,
        }
        .to_xdr(&env, &id)]
    );
}

// ---------------------------------------------------------------------------
// Clear after settle (status=2) still works — metadata path is independent
// ---------------------------------------------------------------------------

#[test]
fn test_clear_after_settle_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    let investor = Address::generate(&env);
    client.fund(&investor, &AMOUNT);
    client.settle();

    client.clear_sme_collateral_commitment();
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ---------------------------------------------------------------------------
// Double clear → NoCollateralToClear on second attempt
// ---------------------------------------------------------------------------

#[test]
fn test_double_clear_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    client.clear_sme_collateral_commitment();

    let result = client.try_clear_sme_collateral_commitment();
    assert_eq!(result, Err(Ok(EscrowError::NoCollateralToClear)));
}

// ---------------------------------------------------------------------------
// get returns None before any record
// ---------------------------------------------------------------------------

#[test]
fn test_get_returns_none_before_record() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, _id, client) = setup(&env);
    assert!(client.get_sme_collateral_commitment().is_none());
}

// ---------------------------------------------------------------------------
// Overwrite: record twice, clear once → None; cleared amount is the last pledge
// ---------------------------------------------------------------------------

#[test]
fn test_overwrite_then_clear() {
    let env = Env::default();
    env.mock_all_auths();
    let (_sme, id, client) = setup(&env);

    client.record_sme_collateral_commitment(&PLEDGE);
    client.record_sme_collateral_commitment(&(PLEDGE * 2));

    let pledge = client.get_sme_collateral_commitment().unwrap();
    assert_eq!(pledge.amount, PLEDGE * 2);

    // The clear event carries the overwritten (latest) amount.
    client.clear_sme_collateral_commitment();

    // Check cleared event BEFORE the next client call resets the event snapshot.
    assert_eq!(
        env.events().all().filter_by_contract(&id),
        std::vec![CollateralClearedEvt {
            invoice_id: symbol_short!("INV001"),
            amount: PLEDGE * 2,
        }
        .to_xdr(&env, &id)]
    );
    assert!(client.get_sme_collateral_commitment().is_none());
}
