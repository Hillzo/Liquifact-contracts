//! LiquiFact Escrow Contract
//!
//! Holds investor funds for an invoice until settlement.
//! - SME receives stablecoin when funding target is met
//! - Investors receive principal + yield when buyer pays at maturity

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, Symbol,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    Escrow,
    SmeCollateralPledge,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowError {
    NotInitialized = 1,
    NotOpen = 2,
    NotFunded = 3,
    /// Raised by clear_sme_collateral_commitment when no pledge is recorded.
    NoCollateralToClear = 4,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceEscrow {
    /// Unique invoice identifier (e.g. INV-1023)
    pub invoice_id: Symbol,
    /// SME wallet that receives liquidity
    pub sme_address: Address,
    /// Total amount in smallest unit (e.g. stroops for XLM)
    pub amount: i128,
    /// Funding target must be met to release to SME
    pub funding_target: i128,
    /// Total funded so far by investors
    pub funded_amount: i128,
    /// Yield basis points (e.g. 800 = 8%)
    pub yield_bps: i64,
    /// Maturity timestamp (ledger time)
    pub maturity: u64,
    /// Escrow status: 0 = open, 1 = funded, 2 = settled
    pub status: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralPledge {
    pub invoice_id: Symbol,
    pub amount: i128,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Emitted by record_sme_collateral_commitment.
#[contractevent(topics = ["collateral_recorded"])]
pub struct CollateralRecordedEvt {
    #[topic]
    pub invoice_id: Symbol,
    pub amount: i128,
}

/// Emitted by clear_sme_collateral_commitment when a pledge is retired.
///
/// `amount` carries the value from the removed pledge record.
#[contractevent(topics = ["collateral_cleared"])]
pub struct CollateralClearedEvt {
    #[topic]
    pub invoice_id: Symbol,
    /// The amount that was recorded in the retired pledge.
    pub amount: i128,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_escrow(env: &Env) -> Result<InvoiceEscrow, EscrowError> {
    env.storage()
        .instance()
        .get(&DataKey::Escrow)
        .ok_or(EscrowError::NotInitialized)
}

/// Load escrow and require the caller to be the SME address.
fn load_escrow_require_sme(env: &Env) -> Result<InvoiceEscrow, EscrowError> {
    let escrow = load_escrow(env)?;
    escrow.sme_address.require_auth();
    Ok(escrow)
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct LiquifactEscrow;

#[contractimpl]
impl LiquifactEscrow {
    /// Initialize a new invoice escrow.
    pub fn init(
        env: Env,
        invoice_id: Symbol,
        sme_address: Address,
        amount: i128,
        yield_bps: i64,
        maturity: u64,
    ) -> InvoiceEscrow {
        let escrow = InvoiceEscrow {
            invoice_id: invoice_id.clone(),
            sme_address: sme_address.clone(),
            amount,
            funding_target: amount,
            funded_amount: 0,
            yield_bps,
            maturity,
            status: 0,
        };
        env.storage().instance().set(&DataKey::Escrow, &escrow);
        escrow
    }

    /// Get current escrow state.
    pub fn get_escrow(env: Env) -> InvoiceEscrow {
        env.storage()
            .instance()
            .get(&DataKey::Escrow)
            .unwrap_or_else(|| panic!("Escrow not initialized"))
    }

    /// Record investor funding. In production, this would be called with token transfer.
    pub fn fund(env: Env, _investor: Address, amount: i128) -> InvoiceEscrow {
        let mut escrow = Self::get_escrow(env.clone());
        assert!(escrow.status == 0, "Escrow not open for funding");
        escrow.funded_amount += amount;
        if escrow.funded_amount >= escrow.funding_target {
            escrow.status = 1;
        }
        env.storage().instance().set(&DataKey::Escrow, &escrow);
        escrow
    }

    /// Mark escrow as settled (buyer paid). Releases principal + yield to investors.
    pub fn settle(env: Env) -> InvoiceEscrow {
        let mut escrow = Self::get_escrow(env.clone());
        assert!(
            escrow.status == 1,
            "Escrow must be funded before settlement"
        );
        escrow.status = 2;
        env.storage().instance().set(&DataKey::Escrow, &escrow);
        escrow
    }

    // -----------------------------------------------------------------------
    // Collateral metadata (no token movement)
    // -----------------------------------------------------------------------

    /// Record an off-chain collateral pledge for this invoice.
    ///
    /// Metadata-only: no tokens are moved or reserved. Requires SME auth.
    /// Overwrites any previously recorded pledge.
    /// Emits [`CollateralRecordedEvt`].
    pub fn record_sme_collateral_commitment(env: Env, amount: i128) -> Result<(), EscrowError> {
        let escrow = load_escrow_require_sme(&env)?;
        let pledge = CollateralPledge {
            invoice_id: escrow.invoice_id.clone(),
            amount,
        };
        env.storage()
            .instance()
            .set(&DataKey::SmeCollateralPledge, &pledge);
        CollateralRecordedEvt {
            invoice_id: escrow.invoice_id,
            amount,
        }
        .publish(&env);
        Ok(())
    }

    /// Return the current collateral pledge, if any.
    pub fn get_sme_collateral_commitment(env: Env) -> Option<CollateralPledge> {
        env.storage().instance().get(&DataKey::SmeCollateralPledge)
    }

    /// Retire a previously recorded collateral pledge.
    ///
    /// Metadata-only: no tokens are moved. Requires SME auth.
    ///
    /// Guard ordering (ADR-002):
    /// 1. Read-only existence check — returns [`EscrowError::NoCollateralToClear`] if absent.
    /// 2. `require_auth` on the SME address.
    /// 3. Remove storage entry and emit [`CollateralClearedEvt`].
    pub fn clear_sme_collateral_commitment(env: Env) -> Result<(), EscrowError> {
        // 1. Read-only existence check (no auth yet).
        let pledge: CollateralPledge = env
            .storage()
            .instance()
            .get(&DataKey::SmeCollateralPledge)
            .ok_or(EscrowError::NoCollateralToClear)?;

        // 2. Load escrow and require SME auth.
        let escrow = load_escrow_require_sme(&env)?;

        // 3. Remove entry and emit retirement event.
        env.storage()
            .instance()
            .remove(&DataKey::SmeCollateralPledge);
        CollateralClearedEvt {
            invoice_id: escrow.invoice_id,
            amount: pledge.amount,
        }
        .publish(&env);
        Ok(())
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod tests;
