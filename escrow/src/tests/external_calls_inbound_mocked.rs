//! Tests for inbound funding-token transfer balance-delta wrapper with mocked tokens.
//! Mirrors outbound tests but targets `transfer_funding_token_inbound_with_balance_checks`.

use super::super::external_calls::transfer_funding_token_inbound_with_balance_checks;
use super::*;
use soroban_sdk::{contract, contractimpl, token::TokenInterface, Address, Env, MuxedAddress};

// ---------------------------------------------------------------------------
// Mock token implementations (reused from outbound tests).
// ---------------------------------------------------------------------------

#[contract]
pub struct FeeOnTransferToken;

#[contractimpl]
impl TokenInterface for FeeOnTransferToken {
    fn balance(env: Env, id: Address) -> i128 {
        env.storage().persistent().get(&id).unwrap_or(0)
    }

    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();
        let fee = amount / 100; // 1% fee
        let credited = amount - fee;
        let to_addr = to.address();
        let from_bal = Self::balance(env.clone(), from.clone());
        env.storage().persistent().set(&from, &(from_bal - amount)); // full debit
        let to_bal = Self::balance(env.clone(), to_addr.clone());
        env.storage().persistent().set(&to_addr, &(to_bal + credited)); // under-credit
    }

    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 { 0 }
    fn approve(_env: Env, _from: Address, _spender: Address, _amount: i128, _exp: u32) {}
    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) { unimplemented!() }
    fn burn(_env: Env, _from: Address, _amount: i128) { unimplemented!() }
    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) { unimplemented!() }
    fn decimals(_env: Env) -> u32 { 7 }
    fn name(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "FeeToken") }
    fn symbol(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "FEE") }
}

/// Mint tokens directly into the fee token's storage (bypasses transfer).
fn mint_fee_token(env: &Env, contract_id: &Address, to: &Address, amount: i128) {
    env.as_contract(contract_id, || {
        let current: i128 = env.storage().persistent().get(to).unwrap_or(0);
        env.storage().persistent().set(to, &(current + amount));
    });
}

#[contract]
pub struct RebasingToken;

#[contractimpl]
impl TokenInterface for RebasingToken {
    fn balance(env: Env, id: Address) -> i128 { env.storage().persistent().get(&id).unwrap_or(0) }
    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();
        let to_addr = to.address();
        // Standard transfer
        let from_bal = Self::balance(env.clone(), from.clone());
        let to_bal = Self::balance(env.clone(), to_addr.clone());
        env.storage().persistent().set(&from, &(from_bal - amount));
        env.storage().persistent().set(&to_addr, &(to_bal + amount));
        // Rebase effect: mint 10% extra to sender after transfer
        let malicious_mint = amount / 10;
        env.storage().persistent().set(&from, &(from_bal - amount + malicious_mint));
    }
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 { 0 }
    fn approve(_env: Env, _from: Address, _spender: Address, _amount: i128, _exp: u32) {}
    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) { unimplemented!() }
    fn burn(_env: Env, _from: Address, _amount: i128) { unimplemented!() }
    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) { unimplemented!() }
    fn decimals(_env: Env) -> u32 { 7 }
    fn name(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "RebaseToken") }
    fn symbol(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "REBASE") }
}

/// Mint tokens directly into the rebasing token's storage.
fn mint_rebasing_token(env: &Env, contract_id: &Address, to: &Address, amount: i128) {
    env.as_contract(contract_id, || {
        let current: i128 = env.storage().persistent().get(to).unwrap_or(0);
        env.storage().persistent().set(to, &(current + amount));
    });
}

#[contract]
pub struct HookStealingToken;

#[contractimpl]
impl TokenInterface for HookStealingToken {
    fn balance(env: Env, id: Address) -> i128 { env.storage().persistent().get(&id).unwrap_or(0) }
    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();
        let to_addr = to.address();
        // Standard transfer
        let from_bal = Self::balance(env.clone(), from.clone());
        let to_bal = Self::balance(env.clone(), to_addr.clone());
        env.storage().persistent().set(&from, &(from_bal - amount));
        env.storage().persistent().set(&to_addr, &(to_bal + amount));
        // Hook: burn 10% of recipient's balance after transfer
        let burn_amount = amount / 10;
        let new_to_bal = to_bal + amount - burn_amount;
        env.storage().persistent().set(&to_addr, &new_to_bal);
    }
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 { 0 }
    fn approve(_env: Env, _from: Address, _spender: Address, _amount: i128, _exp: u32) {}
    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) { unimplemented!() }
    fn burn(_env: Env, _from: Address, _amount: i128) { unimplemented!() }
    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) { unimplemented!() }
    fn decimals(_env: Env) -> u32 { 7 }
    fn name(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "HookToken") }
    fn symbol(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "HOOK") }
}

/// Mint tokens directly into the hook token's storage.
fn mint_hook_token(env: &Env, contract_id: &Address, to: &Address, amount: i128) {
    env.as_contract(contract_id, || {
        let current: i128 = env.storage().persistent().get(to).unwrap_or(0);
        env.storage().persistent().set(to, &(current + amount));
    });
}

#[contract]
pub struct LyingToken;

#[contractimpl]
impl TokenInterface for LyingToken {
    fn balance(env: Env, id: Address) -> i128 { env.storage().persistent().get(&id).unwrap_or(0) }
    fn transfer(_env: Env, from: Address, _to: MuxedAddress, _amount: i128) {
        from.require_auth();
        // No state change – token lies about transfer.
    }
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 { 0 }
    fn approve(_env: Env, _from: Address, _spender: Address, _amount: i128, _exp: u32) {}
    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) { unimplemented!() }
    fn burn(_env: Env, _from: Address, _amount: i128) { unimplemented!() }
    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) { unimplemented!() }
    fn decimals(_env: Env) -> u32 { 7 }
    fn name(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "LyingToken") }
    fn symbol(env: Env) -> soroban_sdk::String { soroban_sdk::String::from_str(&env, "LYE") }
}

/// Mint tokens directly into the lying token's storage.
fn mint_lying_token(env: &Env, contract_id: &Address, to: &Address, amount: i128) {
    env.as_contract(contract_id, || {
        let current: i128 = env.storage().persistent().get(to).unwrap_or(0);
        env.storage().persistent().set(to, &(current + amount));
    });
}

// ---------------------------------------------------------------------------
// Tests for inbound wrapper.
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_inbound_fee_on_transfer_token_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let fee_token_id = env.register(FeeOnTransferToken, ());
    let investor = Address::generate(&env);
    let escrow = deploy_id(&env);
    mint_fee_token(&env, &fee_token_id, &investor, 1000i128);
    // Recipient (escrow) receives less than amount -> panic
    transfer_funding_token_inbound_with_balance_checks(&env, &fee_token_id, &investor, &escrow, 1000i128);
}

#[test]
#[should_panic]
fn test_inbound_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let token = install_stellar_asset_token(&env);
    let investor = deploy_id(&env);
    let escrow = Address::generate(&env);
    transfer_funding_token_inbound_with_balance_checks(&env, &token.id, &investor, &escrow, 0);
}

#[test]
#[should_panic]
fn test_inbound_negative_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let token = install_stellar_asset_token(&env);
    let investor = deploy_id(&env);
    let escrow = Address::generate(&env);
    transfer_funding_token_inbound_with_balance_checks(&env, &token.id, &investor, &escrow, -1i128);
}

#[test]
#[should_panic]
fn test_inbound_insufficient_balance_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let token = install_stellar_asset_token(&env);
    let investor = deploy_id(&env);
    let escrow = Address::generate(&env);
    // Investor has no tokens
    transfer_funding_token_inbound_with_balance_checks(&env, &token.id, &investor, &escrow, 1i128);
}

#[test]
#[should_panic]
fn test_inbound_lying_token_no_change_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let lying_token_id = env.register(LyingToken, ());
    let investor = Address::generate(&env);
    let escrow = deploy_id(&env);
    mint_lying_token(&env, &lying_token_id, &investor, 1000i128);
    // No balance change -> SenderBalanceDeltaMismatch
    transfer_funding_token_inbound_with_balance_checks(&env, &lying_token_id, &investor, &escrow, 1000i128);
}

#[test]
#[should_panic]
fn test_inbound_recipient_decreases_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let hook_token_id = env.register(HookStealingToken, ());
    let investor = Address::generate(&env);
    let escrow = deploy_id(&env);
    mint_hook_token(&env, &hook_token_id, &investor, 1000i128);
    // Hook reduces escrow balance after transfer
    transfer_funding_token_inbound_with_balance_checks(&env, &hook_token_id, &investor, &escrow, 1000i128);
}

#[test]
#[should_panic]
fn test_inbound_sender_increases_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let rebase_token_id = env.register(RebasingToken, ());
    let investor = Address::generate(&env);
    let escrow = deploy_id(&env);
    mint_rebasing_token(&env, &rebase_token_id, &investor, 1000i128);
    // Sender ends with extra tokens -> SenderBalanceUnderflow
    transfer_funding_token_inbound_with_balance_checks(&env, &rebase_token_id, &investor, &escrow, 1000i128);
}

#[test]
fn test_inbound_happy_path() {
    let env = Env::default();
    env.mock_all_auths();
    let token = install_stellar_asset_token(&env);
    let investor = deploy_id(&env);
    let escrow = Address::generate(&env);
    let amount = 1000i128;
    token.stellar.mint(&investor, &amount);
    let investor_before = token.token.balance(&investor);
    let escrow_before = token.token.balance(&escrow);
    transfer_funding_token_inbound_with_balance_checks(&env, &token.id, &investor, &escrow, amount);
    let investor_after = token.token.balance(&investor);
    let escrow_after = token.token.balance(&escrow);
    assert_eq!(investor_before - investor_after, amount);
    assert_eq!(escrow_after - escrow_before, amount);
}
