use super::*;
use crate::{AdminProposedEvent, EscrowCloseSnapshot, FundingTargetUpdated};
use soroban_sdk::Event;

// Admin/governance operations: target changes, maturity changes, admin handover,
// legal hold, migration guards, and collateral metadata.

#[test]
fn test_update_maturity_success() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV006b"),
        &sme,
        &1_000i128,
        &500i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    let updated = client.update_maturity(&2000u64);
    assert_eq!(updated.maturity, 2000u64);
    assert_eq!(updated.status, 0);
}

#[test]
#[should_panic]
fn test_update_maturity_wrong_state() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let investor = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV007"),
        &sme,
        &1_000i128,
        &500i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &1_000i128);
    client.update_maturity(&2000u64);
}

#[test]
#[should_panic]
fn test_update_maturity_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV009"),
        &sme,
        &1_000i128,
        &500i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    env.mock_auths(&[]);
    client.update_maturity(&2000u64);
}

#[test]
fn test_propose_admin_sets_pending_without_changing_admin() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let new_admin = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "T001"),
        &sme,
        &TARGET,
        &800i64,
        &1000u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    let pending = client.propose_admin(&new_admin);
    assert_eq!(pending, new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin));
    assert_eq!(client.get_escrow().admin, admin);
}

#[test]
fn test_accept_admin_promotes_pending_and_clears_pending() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let new_admin = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "TACPT1"),
        &sme,
        &TARGET,
        &800i64,
        &1000u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    client.propose_admin(&new_admin);
    let updated = client.accept_admin();
    assert_eq!(updated.admin, new_admin);
    assert_eq!(client.get_escrow().admin, new_admin);
    assert_eq!(client.get_pending_admin(), None);
}

#[test]
#[allow(deprecated)]
fn test_transfer_admin_deprecated_shim_only_proposes() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let new_admin = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "TSHIM1"),
        &sme,
        &TARGET,
        &800i64,
        &1000u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let unchanged = client.transfer_admin(&new_admin);
    assert_eq!(unchanged.admin, admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin));
}

#[test]
#[should_panic]
fn test_transfer_admin_same_address_panics() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "T002"),
        &sme,
        &TARGET,
        &800i64,
        &1000u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.propose_admin(&admin);
}

#[test]
#[should_panic]
fn test_transfer_admin_uninitialized_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = deploy(&env);
    let new_admin = Address::generate(&env);
    client.propose_admin(&new_admin);
}

#[test]
#[should_panic(expected = "No pending admin")]
fn test_accept_admin_without_pending_panics() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    client.accept_admin();
}

#[test]
#[should_panic]
fn test_accept_admin_requires_pending_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    let new_admin = Address::generate(&env);
    default_init(&client, &env, &admin, &sme);
    client.propose_admin(&new_admin);
    env.mock_auths(&[]);
    client.accept_admin();
}

#[test]
fn test_propose_admin_overwrites_prior_pending() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    default_init(&client, &env, &admin, &sme);

    client.propose_admin(&first);
    client.propose_admin(&second);

    assert_eq!(client.get_pending_admin(), Some(second.clone()));
    let updated = client.accept_admin();
    assert_eq!(updated.admin, second);
}

#[test]
fn test_propose_admin_emits_event() {
    use soroban_sdk::testutils::Events as _;

    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let contract_id = client.address.clone();
    let new_admin = Address::generate(&env);
    default_init(&client, &env, &admin, &sme);

    client.propose_admin(&new_admin);

    assert_eq!(
        env.events().all().events().last().unwrap().clone(),
        AdminProposedEvent {
            name: symbol_short!("adm_prop"),
            invoice_id: client.get_escrow().invoice_id,
            current_admin: admin,
            pending_admin: new_admin,
        }
        .to_xdr(&env, &contract_id)
    );
}

#[test]
#[should_panic]
fn test_migrate_at_current_version_panics() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    client.migrate(&SCHEMA_VERSION);
}

#[test]
#[should_panic]
fn test_migrate_wrong_from_version_panics() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    client.migrate(&99u32);
}

#[test]
#[should_panic]
fn test_migrate_no_path_branch() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = deploy_with_id(&env);
    // Simulate an older version 4 already in storage.
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Version, &4u32);
    });
    // migrate(4) should hit the "No migration path" branch.
    client.migrate(&4u32);
}

#[test]
#[should_panic]
fn test_migrate_from_zero_uninitialized_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let client = deploy(&env);
    // Uninitialized storage returns version 0; migrate(0) hits the no-path branch.
    client.migrate(&0u32);
}

#[test]
fn test_read_model_summary_includes_optional_admin_fields() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let funding_token = Address::generate(&env);
    let treasury = Address::generate(&env);

    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "TSUM01"),
        &sme,
        &TARGET,
        &800i64,
        &1000u64,
        &funding_token,
        &None,
        &treasury,
        &None,
        &Some(100i128),
        &Some(7u32),
        &Some(10_000i128),
        &None,
    );

    let summary = client.get_escrow_summary();

    assert_eq!(summary.escrow, client.get_escrow());
    assert_eq!(summary.legal_hold, client.get_legal_hold());
    assert_eq!(summary.funding_close_snapshot, EscrowCloseSnapshot::None);
    assert_eq!(summary.unique_funder_count, 0);
    assert!(!summary.is_allowlist_active);
    assert_eq!(summary.schema_version, client.get_version());
    assert_eq!(client.get_max_per_investor_cap(), Some(10_000i128));
}

#[test]
fn test_record_collateral_stored_and_does_not_block_settle() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let investor = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "COL001"),
        &sme,
        &TARGET,
        &800i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    let c = client.record_sme_collateral_commitment(&symbol_short!("USDC"), &5000i128);
    assert_eq!(c.amount, 5000i128);
    assert_eq!(c.asset, symbol_short!("USDC"));
    assert_eq!(client.get_sme_collateral_commitment(), Some(c));

    client.fund(&investor, &TARGET);
    let settled = client.settle();
    assert_eq!(settled.status, 2);
}

#[test]
#[should_panic]
fn test_collateral_zero_panics() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "COL002"),
        &sme,
        &TARGET,
        &800i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.record_sme_collateral_commitment(&symbol_short!("XLM"), &0i128);
}

#[test]
#[should_panic]
fn test_collateral_requires_sme_auth() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "COL003"),
        &sme,
        &TARGET,
        &800i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    env.mock_auths(&[]);
    client.record_sme_collateral_commitment(&symbol_short!("XLM"), &100i128);
}

#[test]
fn test_legal_hold_blocks_settle_withdraw_claim_and_fund() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let investor = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "LH001"),
        &sme,
        &TARGET,
        &800i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &TARGET);
    client.set_legal_hold(&true);
    assert!(client.get_legal_hold());

    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.settle();
    }))
    .is_err());

    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.withdraw();
    }))
    .is_err());

    client.clear_legal_hold();
    assert!(!client.get_legal_hold());
    let settled = client.settle();
    assert_eq!(settled.status, 2);

    client.set_legal_hold(&true);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.claim_investor_payout(&investor);
    }))
    .is_err());

    client.clear_legal_hold();
    client.claim_investor_payout(&investor);
    assert!(client.is_investor_claimed(&investor));
}

#[test]
#[should_panic]
fn test_legal_hold_blocks_new_funds_when_open() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    let investor = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "LH002"),
        &sme,
        &TARGET,
        &800i64,
        &0u64,
        &Address::generate(&env),
        &None,
        &Address::generate(&env),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.set_legal_hold(&true);
    client.fund(&investor, &1i128);
}

/// Soroban instance storage returns `None` for a key that has never been written.
/// `legal_hold_active` maps that `None` to `false` via `unwrap_or(false)`, so a
/// fresh deploy must read `false` without any explicit `set_legal_hold` call.
#[test]
fn test_get_legal_hold_defaults_false_on_fresh_deploy() {
    let env = Env::default();
    // No init, no set_legal_hold – DataKey::LegalHold is absent from storage.
    let client = deploy(&env);
    assert!(!client.get_legal_hold());
}

#[test]
fn test_update_funding_target_by_admin_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV001"),
        &sme,
        &5_000i128,
        &800i64,
        &3000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let updated = client.update_funding_target(&10_000i128);
    assert_eq!(updated.funding_target, 10_000i128);
    assert_eq!(updated.status, 0);
}

#[test]
#[should_panic]
fn test_update_funding_target_by_non_admin_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV001"),
        &sme,
        &5_000i128,
        &800i64,
        &3000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    env.mock_auths(&[]);
    client.update_funding_target(&10_000i128);
}

#[test]
#[should_panic]
fn test_update_funding_target_fails_when_funded() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV001"),
        &sme,
        &5_000i128,
        &800i64,
        &3000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128);
    client.update_funding_target(&10_000i128);
}

#[test]
#[should_panic]
fn test_update_funding_target_below_funded_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV001"),
        &sme,
        &10_000i128,
        &800i64,
        &3000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &4_000i128);
    client.update_funding_target(&3_000i128);
}

#[test]
#[should_panic]
fn test_update_funding_target_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "INV001"),
        &sme,
        &5_000i128,
        &800i64,
        &3000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.update_funding_target(&0i128);
}

// --- FundingTargetUpdated event and rejection coverage ---

/// Verify that `update_funding_target` emits a `FundingTargetUpdated` event whose
/// topic is `symbol_short!("fund_tgt")` and whose data fields carry the correct
/// `invoice_id`, `old_target`, and `new_target` values.
#[test]
fn test_update_funding_target_event_fields() {
    use soroban_sdk::testutils::Events as _;

    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);
    let contract_id = client.address.clone();

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "EVT001"),
        &sme,
        &5_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    client.update_funding_target(&9_000i128);

    assert_eq!(
        env.events().all(),
        std::vec![FundingTargetUpdated {
            name: symbol_short!("fund_tgt"),
            invoice_id: client.get_escrow().invoice_id,
            old_target: 5_000i128,
            new_target: 9_000i128,
        }
        .to_xdr(&env, &contract_id)]
    );
}

/// `update_funding_target` must be rejected when the escrow is in the **settled**
/// state (status == 2); only the open state (0) is permitted.
#[test]
#[should_panic]
fn test_update_funding_target_fails_when_settled() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "SETL001"),
        &sme,
        &5_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128); // status → 1 (funded)
    client.settle(); // status → 2 (settled)
    client.update_funding_target(&6_000i128);
}

/// `update_funding_target` must be rejected when the escrow is in the **withdrawn**
/// state (status == 3); only the open state (0) is permitted.
#[test]
#[should_panic]
fn test_update_funding_target_fails_when_withdrawn() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "WD001"),
        &sme,
        &5_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128); // status → 1 (funded)
    client.withdraw(); // status → 3 (withdrawn)
    client.update_funding_target(&6_000i128);
}

/// Setting the new target exactly equal to `funded_amount` is the boundary case
/// that must succeed: the invariant is `new_target >= funded_amount`, so equality
/// is allowed.
#[test]
fn test_update_funding_target_equal_to_funded_amount_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "BOUND001"),
        &sme,
        &10_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &4_000i128); // funded_amount == 4_000, status still 0

    // new_target == funded_amount: boundary — must not panic.
    let updated = client.update_funding_target(&4_000i128);
    assert_eq!(updated.funding_target, 4_000i128);
    assert_eq!(updated.funded_amount, 4_000i128);
    assert_eq!(updated.status, 0);
}

/// Passing a negative value must panic with "Target must be strictly positive".
#[test]
#[should_panic]
fn test_update_funding_target_negative_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "NEG001"),
        &sme,
        &5_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.update_funding_target(&-1i128);
}
// --- update_maturity: open-only, ledger time semantics, MaturityUpdatedEvent ---

/// `update_maturity` must emit a `MaturityUpdatedEvent` with the correct
/// topic (`symbol_short!("maturity")`), `invoice_id`, `old_maturity`, and
/// `new_maturity` fields. Ledger timestamps are validator-observed integers;
/// the contract stores and compares them as raw `u64` seconds.
#[test]
fn test_update_maturity_event_fields() {
    use crate::MaturityUpdatedEvent;
    use soroban_sdk::testutils::Events as _;

    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);
    let contract_id = client.address.clone();

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT001"),
        &sme,
        &5_000i128,
        &800i64,
        &1000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    client.update_maturity(&2000u64);

    assert_eq!(
        env.events().all(),
        std::vec![MaturityUpdatedEvent {
            name: symbol_short!("maturity"),
            invoice_id: client.get_escrow().invoice_id,
            old_maturity: 1000u64,
            new_maturity: 2000u64,
        }
        .to_xdr(&env, &contract_id)]
    );
}

/// `update_maturity` must be rejected when the escrow is in the **funded**
/// state (status == 1); only Open (0) is permitted.
#[test]
#[should_panic]
fn test_update_maturity_fails_when_funded() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT002"),
        &sme,
        &5_000i128,
        &800i64,
        &1000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128); // status → 1 (funded)
    client.update_maturity(&2000u64);
}

/// `update_maturity` must be rejected when the escrow is **settled**
/// (status == 2); only Open (0) is permitted.
#[test]
#[should_panic]
fn test_update_maturity_fails_when_settled() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT003"),
        &sme,
        &5_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128); // status → 1
    client.settle(); // status → 2
    client.update_maturity(&2000u64);
}

/// `update_maturity` must be rejected when the escrow is **withdrawn**
/// (status == 3); only Open (0) is permitted.
#[test]
#[should_panic]
fn test_update_maturity_fails_when_withdrawn() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT004"),
        &sme,
        &5_000i128,
        &800i64,
        &0u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128); // status → 1
    client.withdraw(); // status → 3
    client.update_maturity(&2000u64);
}

/// Setting maturity to zero is valid — it means no maturity gate.
/// The contract must accept zero as new_maturity in Open state.
#[test]
fn test_update_maturity_to_zero_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT005"),
        &sme,
        &5_000i128,
        &800i64,
        &1000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    let updated = client.update_maturity(&0u64);
    assert_eq!(updated.maturity, 0u64);
    assert_eq!(updated.status, 0);
}

/// Ledger time semantics: `settle` uses `env.ledger().timestamp()`
/// (validator-observed seconds). Settle must pass exactly at maturity —
/// confirming the boundary is `now >= maturity` (inclusive).
#[test]
fn test_settle_passes_exactly_at_maturity_ledger_time() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT006"),
        &sme,
        &5_000i128,
        &800i64,
        &5000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128);

    // Advance ledger to exactly maturity — must succeed
    env.ledger().with_mut(|l| l.timestamp = 5000);
    let settled = client.settle();
    assert_eq!(settled.status, 2);
}

/// Ledger time semantics: settle must panic one second before maturity —
/// confirming the `>=` boundary strictly excludes values below maturity.
#[test]
#[should_panic]
fn test_settle_fails_one_second_before_maturity() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT007"),
        &sme,
        &5_000i128,
        &800i64,
        &5000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &5_000i128);

    // One second before maturity — must reject
    env.ledger().with_mut(|l| l.timestamp = 4999);
    client.settle();
}

/// A second `update_maturity` call in the same Open state must overwrite
/// the previous value correctly — storage is atomic per call.
#[test]
fn test_update_maturity_twice_overwrites() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);

    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "MAT008"),
        &sme,
        &5_000i128,
        &800i64,
        &1000u64,
        &token,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    client.update_maturity(&2000u64);
    let updated = client.update_maturity(&3000u64);
    assert_eq!(updated.maturity, 3000u64);
    assert_eq!(client.get_escrow().maturity, 3000u64);
}

// ── Authorization guard ordering audit (issue #265) ───────────────────────────
//
// Negative tests: each guarded entrypoint must trap when `require_auth` fails
// (Soroban host aborts the transaction). Canonical ordering is documented in
// `docs/escrow-security-checklist.md` §6 and ADR-002.

fn auth_audit_init_funded(
    env: &Env,
) -> (
    LiquifactEscrowClient<'_>,
    Address,
    Address,
    Address,
    Address,
) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let sme = Address::generate(env);
    let investor = Address::generate(env);
    let client = deploy(env);
    default_init(&client, env, &admin, &sme);
    client.fund(&investor, &TARGET);
    (client, admin, sme, investor, Address::generate(env))
}

#[test]
#[should_panic]
fn auth_audit_propose_admin_requires_current_admin() {
    let env = Env::default();
    let (client, _, _, _, _) = auth_audit_init_funded(&env);
    let new_admin = Address::generate(&env);
    env.mock_auths(&[]);
    client.propose_admin(&new_admin);
}

#[test]
#[should_panic]
fn auth_audit_accept_admin_requires_pending_admin() {
    let env = Env::default();
    let (client, _, _, _, pending_admin) = auth_audit_init_funded(&env);
    client.propose_admin(&pending_admin);
    env.mock_auths(&[]);
    client.accept_admin();
}

#[test]
#[should_panic]
fn auth_audit_fund_requires_investor() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    let investor = Address::generate(&env);
    env.mock_auths(&[]);
    client.fund(&investor, &TARGET);
}

#[test]
#[should_panic]
fn auth_audit_fund_with_commitment_requires_investor() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    let investor = Address::generate(&env);
    env.mock_auths(&[]);
    client.fund_with_commitment(&investor, &TARGET, &0u64);
}

#[test]
#[should_panic]
fn auth_audit_settle_requires_sme() {
    let env = Env::default();
    let (client, _, _, _, _) = auth_audit_init_funded(&env);
    env.mock_auths(&[]);
    client.settle();
}

#[test]
#[should_panic]
fn auth_audit_withdraw_requires_sme() {
    let env = Env::default();
    let (client, _, _, _, _) = auth_audit_init_funded(&env);
    env.mock_auths(&[]);
    client.withdraw();
}

#[test]
#[should_panic]
fn auth_audit_claim_investor_payout_requires_investor() {
    let env = Env::default();
    let (client, _, _, investor, _) = auth_audit_init_funded(&env);
    client.settle();
    env.mock_auths(&[]);
    client.claim_investor_payout(&investor);
}

#[test]
#[should_panic]
fn auth_audit_set_legal_hold_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    env.mock_auths(&[]);
    client.set_legal_hold(&true);
}

#[test]
#[should_panic]
fn auth_audit_bind_primary_attestation_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    env.mock_auths(&[]);
    client.bind_primary_attestation_hash(&soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));
}

#[test]
#[should_panic]
fn auth_audit_append_attestation_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    env.mock_auths(&[]);
    client.append_attestation_digest(&soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));
}

#[test]
#[should_panic]
fn auth_audit_set_allowlist_active_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);
    env.mock_auths(&[]);
    client.set_allowlist_active(&true);
}

#[test]
#[should_panic]
fn auth_audit_sweep_terminal_dust_requires_treasury() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let investor = Address::generate(&env);
    let token = install_stellar_asset_token(&env);
    let treasury = Address::generate(&env);
    let escrow_id = deploy_id(&env);
    let client = LiquifactEscrowClient::new(&env, &escrow_id);
    client.init(
        &admin,
        &soroban_sdk::String::from_str(&env, "AUTHSW"),
        &sme,
        &TARGET,
        &800i64,
        &0u64,
        &token.id,
        &None,
        &treasury,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.fund(&investor, &TARGET);
    client.settle();
    token.stellar.mint(&escrow_id, &100i128);
    env.mock_auths(&[]);
    client.sweep_terminal_dust(&100i128);
}

#[test]
fn test_upgrade_admin_success() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);

    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    client.upgrade(&new_wasm_hash);
}

#[test]
fn test_upgrade_state_survives() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);

    let escrow_before = client.get_escrow();

    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    client.upgrade(&new_wasm_hash);

    let escrow_after: InvoiceEscrow = env.as_contract(&client.address, || {
        env.storage().instance().get(&DataKey::Escrow).unwrap()
    });

    assert_eq!(escrow_after.admin, escrow_before.admin);
    assert_eq!(escrow_after.invoice_id, escrow_before.invoice_id);
    assert_eq!(escrow_after.sme_address, escrow_before.sme_address);
    assert_eq!(escrow_after.amount, escrow_before.amount);
    assert_eq!(escrow_after.funding_target, escrow_before.funding_target);
    assert_eq!(escrow_after.funded_amount, escrow_before.funded_amount);
    assert_eq!(escrow_after.yield_bps, escrow_before.yield_bps);
    assert_eq!(escrow_after.maturity, escrow_before.maturity);
    assert_eq!(escrow_after.status, escrow_before.status);
}

#[test]
fn test_upgrade_event_emitted() {
    use soroban_sdk::testutils::Events as _;
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);

    let escrow = client.get_escrow();

    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    client.upgrade(&new_wasm_hash);

    let event = env.events().all().events().last().unwrap().clone();
    assert_eq!(
        event,
        ContractUpgraded {
            name: symbol_short!("upgrade"),
            invoice_id: escrow.invoice_id,
            new_wasm_hash: new_wasm_hash.clone(),
        }
        .to_xdr(&env, &client.address)
    );
}

#[test]
fn test_upgrade_unauthorized_caller_rejected() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);

    let escrow_before = client.get_escrow();
    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    // Call upgrade as a non-admin by clearing mock auths
    env.mock_auths(&[]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.upgrade(&new_wasm_hash);
    }));
    assert!(result.is_err());

    // Restore mock auths to verify storage remains unchanged
    env.mock_all_auths();
    let escrow_after = client.get_escrow();
    assert_eq!(escrow_after.admin, escrow_before.admin);
    assert_eq!(escrow_after.invoice_id, escrow_before.invoice_id);
}

#[test]
fn test_wrong_mock_auth_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sme = Address::generate(&env);
    let client = deploy(&env);
    default_init(&client, &env, &admin, &sme);

    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    // Mock auth for a wrong random address
    let wrong_addr = Address::generate(&env);
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &wrong_addr,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: soroban_sdk::vec![&env, new_wasm_hash.clone().into()],
            sub_invokes: &[],
        },
    }]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.upgrade(&new_wasm_hash);
    }));
    assert!(result.is_err());
}

#[test]
fn test_upgrade_read_preserved_escrow() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);

    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    client.upgrade(&new_wasm_hash);

    let escrow = env.as_contract(&client.address, || LiquifactEscrow::get_escrow(env.clone()));

    assert_eq!(escrow.admin, admin);
}

#[test]
fn test_upgrade_does_not_increment_version() {
    let env = Env::default();
    let (client, admin, sme) = setup(&env);
    default_init(&client, &env, &admin, &sme);

    let version_before = env.as_contract(&client.address, || {
        LiquifactEscrow::get_version(env.clone())
    });
    assert_eq!(version_before, SCHEMA_VERSION);

    let dummy_wasm = soroban_sdk::Bytes::from_slice(&env, DUMMY_WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(dummy_wasm);

    client.upgrade(&new_wasm_hash);

    let version_after = env.as_contract(&client.address, || {
        LiquifactEscrow::get_version(env.clone())
    });
    assert_eq!(version_after, SCHEMA_VERSION);
}

const DUMMY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x04, 0x05, 0x01, 0x70, 0x01, 0x01, 0x01, 0x05,
    0x03, 0x01, 0x00, 0x10, 0x06, 0x21, 0x04, 0x7f, 0x01, 0x41, 0x80, 0x80, 0xc0, 0x00, 0x0b, 0x7f,
    0x00, 0x41, 0x80, 0x80, 0xc0, 0x00, 0x0b, 0x7f, 0x00, 0x41, 0x80, 0x80, 0xc0, 0x00, 0x0b, 0x7f,
    0x00, 0x41, 0x80, 0x80, 0xc0, 0x00, 0x0b, 0x07, 0x29, 0x04, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72,
    0x79, 0x02, 0x00, 0x01, 0x5f, 0x03, 0x01, 0x0a, 0x5f, 0x5f, 0x64, 0x61, 0x74, 0x61, 0x5f, 0x65,
    0x6e, 0x64, 0x03, 0x02, 0x0b, 0x5f, 0x5f, 0x68, 0x65, 0x61, 0x70, 0x5f, 0x62, 0x61, 0x73, 0x65,
    0x03, 0x03, 0x0b, 0x09, 0x01, 0x00, 0x41, 0x80, 0x80, 0xc0, 0x00, 0x0b, 0x00, 0x00, 0x1e, 0x11,
    0x63, 0x6f, 0x6e, 0x74, 0x72, 0x61, 0x63, 0x74, 0x65, 0x6e, 0x76, 0x6d, 0x65, 0x74, 0x61, 0x76,
    0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6f, 0x0e,
    0x63, 0x6f, 0x6e, 0x74, 0x72, 0x61, 0x63, 0x74, 0x6d, 0x65, 0x74, 0x61, 0x76, 0x30, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x72, 0x73, 0x76, 0x65, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x06, 0x31, 0x2e, 0x39, 0x34, 0x2e, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x08, 0x72, 0x73, 0x73, 0x64, 0x6b, 0x76, 0x65, 0x72, 0x00, 0x00, 0x00, 0x2f, 0x32, 0x35,
    0x2e, 0x32, 0x2e, 0x30, 0x23, 0x33, 0x65, 0x35, 0x32, 0x39, 0x61, 0x36, 0x38, 0x62, 0x34, 0x34,
    0x39, 0x63, 0x34, 0x64, 0x63, 0x33, 0x66, 0x33, 0x63, 0x32, 0x64, 0x35, 0x34, 0x33, 0x30, 0x34,
    0x61, 0x32, 0x33, 0x62, 0x61, 0x38, 0x35, 0x39, 0x37, 0x64, 0x31, 0x63, 0x66, 0x00, 0x00, 0x40,
    0x04, 0x6e, 0x61, 0x6d, 0x65, 0x00, 0x19, 0x18, 0x64, 0x75, 0x6d, 0x6d, 0x79, 0x5f, 0x77, 0x61,
    0x73, 0x6d, 0x5f, 0x63, 0x6f, 0x6e, 0x74, 0x72, 0x61, 0x63, 0x74, 0x2e, 0x77, 0x61, 0x73, 0x6d,
    0x07, 0x12, 0x01, 0x00, 0x0f, 0x5f, 0x5f, 0x73, 0x74, 0x61, 0x63, 0x6b, 0x5f, 0x70, 0x6f, 0x69,
    0x6e, 0x74, 0x65, 0x72, 0x09, 0x0a, 0x01, 0x00, 0x07, 0x2e, 0x72, 0x6f, 0x64, 0x61, 0x74, 0x61,
    0x00, 0x4d, 0x09, 0x70, 0x72, 0x6f, 0x64, 0x75, 0x63, 0x65, 0x72, 0x73, 0x02, 0x08, 0x6c, 0x61,
    0x6e, 0x67, 0x75, 0x61, 0x67, 0x65, 0x01, 0x04, 0x52, 0x75, 0x73, 0x74, 0x00, 0x0c, 0x70, 0x72,
    0x6f, 0x63, 0x65, 0x73, 0x73, 0x65, 0x64, 0x2d, 0x62, 0x79, 0x01, 0x05, 0x72, 0x75, 0x73, 0x74,
    0x63, 0x1d, 0x31, 0x2e, 0x39, 0x34, 0x2e, 0x31, 0x20, 0x28, 0x65, 0x34, 0x30, 0x38, 0x39, 0x34,
    0x37, 0x62, 0x66, 0x20, 0x32, 0x30, 0x32, 0x36, 0x2d, 0x30, 0x33, 0x2d, 0x32, 0x35, 0x29, 0x00,
    0x94, 0x01, 0x0f, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x5f, 0x66, 0x65, 0x61, 0x74, 0x75, 0x72,
    0x65, 0x73, 0x08, 0x2b, 0x0b, 0x62, 0x75, 0x6c, 0x6b, 0x2d, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79,
    0x2b, 0x0f, 0x62, 0x75, 0x6c, 0x6b, 0x2d, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x2d, 0x6f, 0x70,
    0x74, 0x2b, 0x16, 0x63, 0x61, 0x6c, 0x6c, 0x2d, 0x69, 0x6e, 0x64, 0x69, 0x72, 0x65, 0x63, 0x74,
    0x2d, 0x6f, 0x76, 0x65, 0x72, 0x6c, 0x6f, 0x6e, 0x67, 0x2b, 0x0a, 0x6d, 0x75, 0x6c, 0x74, 0x69,
    0x76, 0x61, 0x6c, 0x75, 0x65, 0x2b, 0x0f, 0x6d, 0x75, 0x74, 0x61, 0x62, 0x6c, 0x65, 0x2d, 0x67,
    0x6c, 0x6f, 0x62, 0x61, 0x6c, 0x73, 0x2b, 0x13, 0x6e, 0x6f, 0x6e, 0x74, 0x72, 0x61, 0x70, 0x70,
    0x69, 0x6e, 0x67, 0x2d, 0x66, 0x70, 0x74, 0x6f, 0x69, 0x6e, 0x74, 0x2b, 0x0f, 0x72, 0x65, 0x66,
    0x65, 0x72, 0x65, 0x6e, 0x63, 0x65, 0x2d, 0x74, 0x79, 0x70, 0x65, 0x73, 0x2b, 0x08, 0x73, 0x69,
    0x67, 0x6e, 0x2d, 0x65, 0x78, 0x74,
];
