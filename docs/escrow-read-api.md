# Escrow Read API

Complete catalog of all public read-only views on `LiquifactEscrow`. All functions are pure reads:
no state mutation, no authorization required unless specified otherwise.

**Integrator note:** Return types, defaults, and absent-key behavior documented for each view match
the on-chain implementation exactly. Off-chain tooling should use these views rather than
re-implementing storage reads to guarantee identical semantics.

---

## Index

**Core Escrow State:**
- [get_escrow](#get_escrow--invoiceescrow)
- [get_version](#get_version--u32)
- [get_escrow_summary](#get_escrow_summary--escrowsummary)

**Immutable Bindings:**
- [get_funding_token](#get_funding_token--address)
- [get_treasury](#get_treasury--address)
- [get_registry_ref](#get_registry_ref--optionaddress)

**Admin & Governance:**
- [get_pending_admin](#get_pending_admin--optionaddress)
- [get_legal_hold](#get_legal_hold--bool)
- [get_legal_hold_clear_delay](#get_legal_hold_clear_delay--u64)
- [get_legal_hold_clearable_at](#get_legal_hold_clearable_at--optionu64)

**Funding Constraints:**
- [get_funding_deadline](#get_funding_deadline--optionu64)
- [is_funding_expired](#is_funding_expired--bool)
- [get_min_contribution_floor](#get_min_contribution_floor--i128)
- [get_max_unique_investors_cap](#get_max_unique_investors_cap--optionu32)
- [get_max_per_investor_cap](#get_max_per_investor_cap--optioni128)

**Maturity & Settlement:**
- [has_maturity_lock](#has_maturity_lock--bool)
- [get_funding_close_snapshot](#get_funding_close_snapshot--optionfundingclosesnapshot)

**Per-Investor State:**
- [get_contribution](#get_contributioninvestor-address--i128)
- [get_unique_funder_count](#get_unique_funder_count--u32)
- [get_investor_yield_bps](#get_investor_yield_bpsinvestor-address--i64)
- [get_investor_claim_not_before](#get_investor_claim_not_beforeinvestor-address--u64)
- [is_investor_claimed](#is_investor_claimedinvestor-address--bool)
- [is_investor_refunded](#is_investor_refundedinvestor-address--bool)
- [compute_investor_payout](#compute_investor_payoutinvestor-address--i128)

**Attestations:**
- [get_primary_attestation_hash](#get_primary_attestation_hash--optionbytesn32)
- [get_attestation_append_log](#get_attestation_append_log--vecbytesn32)
- [is_attestation_revoked](#is_attestation_revokedindex-u32--bool)

**Collateral Metadata:**
- [get_sme_collateral_commitment](#get_sme_collateral_commitment--optionsmecollateralcommitment)

**Allowlist:**
- [is_allowlist_active](#is_allowlist_active--bool)
- [is_investor_allowlisted](#is_investor_allowlistedinvestor-address--bool)

**Distributed Principal:**
- [get_distributed_principal](#get_distributed_principal--i128)

---

## Core Escrow State

### `get_escrow() → InvoiceEscrow`

**Storage key:** `DataKey::Escrow`  
**Signature:** `pub fn get_escrow(env: Env) -> InvoiceEscrow`

Returns the full escrow snapshot containing all core state fields.

**Requires initialization:** Yes — emits [`EscrowError::EscrowNotInitialized`] (code 20) if called before `init`.

**Return value:**
- `InvoiceEscrow` struct with fields: `invoice_id`, `admin`, `sme_address`, `amount`, `funding_target`, `funded_amount`, `yield_bps`, `maturity`, `status`.

---

### `get_version() → u32`

**Storage key:** `DataKey::Version`  
**Signature:** `pub fn get_version(env: Env) -> u32`

Returns the stored schema version written by `init` (see `SCHEMA_VERSION`).

**Requires initialization:** No  
**Default when absent:** `0`

**Return value:**
- `u32` schema version (current production: `6`).
- Returns `0` if called before `init`.

---

### `get_escrow_summary() → EscrowSummary`

**Signature:** `pub fn get_escrow_summary(env: Env) -> EscrowSummary`

Bundles multiple read-only values in a single host invocation, optimizing read latency and gas efficiency for off-chain indexers and frontend rendering.

**Requires initialization:** Yes — panics via `get_escrow` if escrow is not initialized.

**Return value:** `EscrowSummary` struct containing:
- `escrow: InvoiceEscrow` — Full escrow snapshot.
- `has_maturity_lock: bool` — True when `escrow.maturity > 0`.
- `legal_hold: bool` — True if compliance hold is active.
- `funding_close_snapshot: EscrowCloseSnapshot` — Custom option-like enum (`None` or `Some(FundingCloseSnapshot)`).
- `unique_funder_count: u32` — Distinct address count.
- `is_allowlist_active: bool` — Allowlist gate status.
- `schema_version: u32` — Contract schema version.
- `sme_collateral_commitment: CollateralCommitmentSnapshot` — Custom option-like enum (`None` or `Some(SmeCollateralCommitment)`).
- `has_primary_attestation: bool` — Primary attestation binding status.
- `attestation_log_length: u32` — Number of append-log entries.

---

## Immutable Bindings

### `get_funding_token() → Address`

**Storage key:** `DataKey::FundingToken`  
**Signature:** `pub fn get_funding_token(env: Env) -> Address`

Returns the SEP-41 token contract address bound to this escrow instance at `init`.

**Immutable:** Set once at `init`; cannot change after deploy.  
**Requires initialization:** Yes — emits [`EscrowError::FundingTokenNotSet`] (code 21) if called before `init`.

**Return value:**
- `Address` of the funding token contract.
- This is the only token that `sweep_terminal_dust` may transfer to the treasury.

---

### `get_treasury() → Address`

**Storage key:** `DataKey::Treasury`  
**Signature:** `pub fn get_treasury(env: Env) -> Address`

Returns the protocol treasury address that receives terminal dust sweeps.

**Immutable:** Set once at `init`; cannot change after deploy.  
**Requires initialization:** Yes — emits [`EscrowError::TreasuryNotSet`] (code 22) if called before `init`.

**Return value:**
- `Address` of the treasury.
- The treasury must authorize `sweep_terminal_dust`; the admin cannot sweep unless it is also the treasury.

---

### `get_registry_ref() → Option<Address>`

**Storage key:** `DataKey::RegistryRef`  
**Signature:** `pub fn get_registry_ref(env: Env) -> Option<Address>`

Returns the optional registry contract address supplied at `init`, or `None` when absent.

**Immutable:** Set once at `init`; cannot change after deploy.  
**Requires initialization:** No  
**Default when absent:** `None`

**Non-authority model:**
- `RegistryRef` is a **read-only discoverability hint** for off-chain indexers only.
- No on-chain logic in this contract reads or calls this address.
- Its presence **does not** prove registry membership; call the registry contract directly to verify.
- The key is omitted from instance storage entirely when `registry = None` at `init`.

**Return value:**
- `Some(Address)` when a registry was configured.
- `None` otherwise.

---

## Admin & Governance

### `get_pending_admin() → Option<Address>`

**Storage key:** `DataKey::PendingAdmin`  
**Signature:** `pub fn get_pending_admin(env: Env) -> Option<Address>`

Returns the proposed successor admin waiting for `accept_admin`, or `None` when no handover is in progress.

**Requires initialization:** No  
**Default when absent:** `None`

**Return value:**
- `Some(Address)` when a handover is pending.
- `None` when no `propose_admin` has been issued, or after a successful `accept_admin`.

---

### `get_legal_hold() → bool`

**Storage key:** `DataKey::LegalHold`  
**Signature:** `pub fn get_legal_hold(env: Env) -> bool`

Returns `true` when a compliance hold is active; blocks `settle`, `withdraw`, `claim_investor_payout`, `fund`, and `sweep_terminal_dust`.

**Requires initialization:** No  
**Default when absent:** `false`

---

### `get_legal_hold_clear_delay() → u64`

**Storage key:** `DataKey::LegalHoldClearDelay`  
**Signature:** `pub fn get_legal_hold_clear_delay(env: Env) -> u64`

Returns the configured minimum delay (in seconds) between `request_clear_legal_hold` and `set_legal_hold(false)`.

**Requires initialization:** No  
**Default when absent:** `0` (no delay enforced; hold can be cleared immediately)

---

### `get_legal_hold_clearable_at() → Option<u64>`

**Storage key:** `DataKey::LegalHoldClearableAt`  
**Signature:** `pub fn get_legal_hold_clearable_at(env: Env) -> Option<u64>`

Returns the earliest ledger timestamp at which a pending legal-hold clear may be applied, or `None` when no clear request has been recorded.

**Requires initialization:** No  
**Default when absent:** `None`

**Return value:**
- `Some(timestamp)` after `request_clear_legal_hold` is called.
- `None` when no request is pending (or after a successful clear removes the key).

---

## Funding Constraints

### `get_funding_deadline() → Option<u64>`

**Storage key:** `DataKey::FundingDeadline`  
**Signature:** `pub fn get_funding_deadline(env: Env) -> Option<u64>`

Returns the optional funding deadline (ledger timestamp). After this timestamp passes, `fund` calls are rejected.

**Requires initialization:** No  
**Default when absent:** `None` (no deadline — funding is open indefinitely)

**Return value:**
- `Some(timestamp)` when configured at `init`.
- `None` when no deadline was set.

---

### `is_funding_expired() → bool`

**Signature:** `pub fn is_funding_expired(env: Env) -> bool`

Returns `true` when a funding deadline is set **and** `Env::ledger().timestamp() > deadline`.

**Requires initialization:** No  
**Default when absent:** `false` (no deadline set → never expired)

**Logic:**
```
if FundingDeadline exists:
    return ledger.timestamp() > deadline
else:
    return false
```

---

### `get_min_contribution_floor() → i128`

**Storage key:** `DataKey::MinContributionFloor`  
**Signature:** `pub fn get_min_contribution_floor(env: Env) -> i128`

Returns the minimum per-call funding amount in token base units. Applies to every `fund` / `fund_with_commitment` call.

**Requires initialization:** No (but written as `0` at `init`)  
**Default when absent:** `0` (no extra floor beyond "amount must be positive")

**Notes:**
- The floor applies to **each individual deposit**, not to cumulative principal.
- Written as `0` even when unconfigured at `init`, so reads always succeed post-init.

---

### `get_max_unique_investors_cap() → Option<u32>`

**Storage key:** `DataKey::MaxUniqueInvestorsCap`  
**Signature:** `pub fn get_max_unique_investors_cap(env: Env) -> Option<u32>`

Returns the optional cap on distinct investor addresses. Reflects the current stored cap, including any reduction via `lower_max_unique_investors`.

**Requires initialization:** No  
**Default when absent:** `None` (unlimited investors)

**Return value:**
- `Some(u32)` when configured.
- `None` when no cap was set at `init`.

---

### `get_max_per_investor_cap() → Option<i128>`

**Storage key:** `DataKey::MaxPerInvestorCap`  
**Signature:** `pub fn get_max_per_investor_cap(env: Env) -> Option<i128>`

Returns the optional immutable cap on cumulative principal for a single investor address.

**Requires initialization:** No  
**Default when absent:** `None` (unlimited per-investor)

**Return value:**
- `Some(i128)` when configured at `init`.
- `None` when unconfigured.

---

## Maturity & Settlement

### `has_maturity_lock() → bool`

**Derived from:** `DataKey::Escrow.maturity`  
**Signature:** `pub fn has_maturity_lock(env: Env) -> bool`

Returns `true` when `InvoiceEscrow::maturity > 0` and `settle()` is gated by ledger time.

**Requires initialization:** Yes — calls `get_escrow` internally.

**Logic:**
```
return get_escrow().maturity > 0
```

**Return value:**
- `true` — settlement requires `Env::ledger().timestamp() >= maturity`.
- `false` — `maturity == 0`; no time lock, funded escrow can settle immediately.

---

### `get_funding_close_snapshot() → Option<FundingCloseSnapshot>`

**Storage key:** `DataKey::FundingCloseSnapshot`  
**Signature:** `pub fn get_funding_close_snapshot(env: Env) -> Option<FundingCloseSnapshot>`

Returns the pro-rata denominator snapshot captured exactly once when the escrow first transitioned from open (0) to funded (1).

**Requires initialization:** No  
**Default when absent:** `None` (escrow has not yet reached funded status)

**Immutable once written:** the snapshot is never updated after the status-0-to-1 transition.

**Return value:**
- `None` until the escrow reaches `status == 1`.
- `Some(FundingCloseSnapshot)` with fields:
  - `total_principal: i128` — `funded_amount` at close (includes over-funding past target).
  - `funding_target: i128` — Snapshot of target at close time.
  - `closed_at_ledger_timestamp: u64` — Ledger timestamp of the funding transition.
  - `closed_at_ledger_sequence: u32` — Ledger sequence at transition.

Historical alias of [`get_effective_yield_bps`](#get_effective_yield_bpsinvestor-address--i64) —
same return value, documented around the per-investor storage slot.

---

## `get_effective_yield_bps(investor: Address) → i64`

**Storage key:** `DataKey::InvestorEffectiveYield(investor)`, falling back to `DataKey::Escrow.yield_bps`

Returns the **resolved effective yield (bps)** the investor would receive at settlement — exactly the
rate `compute_investor_payout` applies when computing the coupon. The resolution is identical to the
payout math:

```text
effective_yield_bps = InvestorEffectiveYield(investor)   // tier locked at first deposit
                      .unwrap_or(escrow.yield_bps)        // else the escrow base yield
```

| Investor state | Returns |
| --- | --- |
| Tiered (funded via `fund_with_commitment`) | the tier `yield_bps` selected at first deposit |
| Base-only / non-tiered | the escrow base `yield_bps` |
| Unknown (never funded) | the escrow base `yield_bps` |

### Stored vs resolved

`DataKey::InvestorEffectiveYield` is the **stored** per-investor slot: present only after a tiered
first deposit, absent otherwise. This view returns the **resolved** value — the stored slot when
present, otherwise the base-yield fallback — so integrators read the same number the payout math uses
without re-implementing the `unwrap_or` fallback themselves.

`get_investor_yield_bps` returns the same value; prefer `get_effective_yield_bps` when the intent is
"the rate `compute_investor_payout` will actually apply."

---

## Per-Investor State

### `get_contribution(investor: Address) → i128`

**Storage key:** `DataKey::InvestorContribution(investor)` (persistent)  
**Signature:** `pub fn get_contribution(env: Env, investor: Address) -> i128`

Returns the cumulative principal contributed by `investor` in token base units.

**Requires initialization:** No  
**Default when absent:** `0` (never contributed)  
**Storage type:** Persistent (independent TTL per address; see ADR-007)

---

### `get_unique_funder_count() → u32`

**Storage key:** `DataKey::UniqueFunderCount`  
**Signature:** `pub fn get_unique_funder_count(env: Env) -> u32`

Returns the count of distinct investor addresses with non-zero contributions. Initialized to `0` at `init`.

**Requires initialization:** No (but written as `0` at `init`)  
**Default when absent:** `0`

**Notes:** counts distinct chain accounts, not real-world persons (Sybil resistance is not a goal of this counter).

---

### `get_investor_yield_bps(investor: Address) → i64`

**Storage key:** `DataKey::InvestorEffectiveYield(investor)` (persistent)  
**Signature:** `pub fn get_investor_yield_bps(env: Env, investor: Address) -> i64`

Returns the effective annualized yield in basis points locked in at the investor's first deposit.

**Requires initialization:** Yes — reads `get_escrow()` for the base yield fallback.  
**Default when absent:** falls back to `InvoiceEscrow::yield_bps` (base yield for legacy / simple `fund` positions)  
**Storage type:** Persistent

**Return value:**
- Investor's tier-selected `yield_bps` when set via `fund_with_commitment`.
- Base `InvoiceEscrow::yield_bps` for simple `fund` deposits or pre-v2 positions.

---

### `get_investor_claim_not_before(investor: Address) → u64`

**Storage key:** `DataKey::InvestorClaimNotBefore(investor)` (persistent)  
**Signature:** `pub fn get_investor_claim_not_before(env: Env, investor: Address) -> u64`

Returns the earliest ledger timestamp at which `claim_investor_payout` may succeed for this investor.

**Requires initialization:** No  
**Default when absent:** `0` (no extra gate beyond settled status)  
**Storage type:** Persistent

**Return value:**
- `0` for simple `fund` deposits or when `committed_lock_secs == 0`.
- `now + committed_lock_secs` at deposit time for tiered commitments.

---

### `is_investor_claimed(investor: Address) → bool`

**Storage key:** `DataKey::InvestorClaimed(investor)` (persistent)  
**Signature:** `pub fn is_investor_claimed(env: Env, investor: Address) -> bool`

Returns `true` when the investor has exercised `claim_investor_payout` after settlement.

**Requires initialization:** No  
**Default when absent:** `false`  
**Storage type:** Persistent

**Notes:** written once and never unset. A second `claim_investor_payout` call is a no-op (idempotent) rather than an error.

---

### `is_investor_refunded(investor: Address) → bool`

**Storage key:** `DataKey::InvestorRefunded(investor)`  
**Signature:** `pub fn is_investor_refunded(env: Env, investor: Address) -> bool`

Returns `true` when an investor's principal has been returned via `refund` in a cancelled (status 4) escrow.

**Requires initialization:** No  
**Default when absent:** `false`

**Notes:** written once; prevents double-refund. After `refund` succeeds, `get_contribution` for the same address returns `0`.

---

### `compute_investor_payout(investor: Address) → i128`

**Signature:** `pub fn compute_investor_payout(env: Env, investor: Address) -> i128`

- `None` — Escrow is not yet funded; no close snapshot exists.
- `Some(FundingCloseSnapshot)` — The pro-rata denominator snapshot captured when the escrow first transitioned to **funded**.

---

## `get_investors(start: u32, limit: u32) → Vec<Address>`

**Storage key:** `DataKey::InvestorIndex`

Returns a paginated list of investor addresses who have contributed to the escrow.

- **Pure Read** — no authorization required.
- **Pagination** — uses `start` (0-based) and `limit` to support paging.
- **Bounded limit** — the `limit` parameter is capped internally (at 50) to prevent CPU/memory resource exhaustion.
- **Legacy Compatibility (ADR-007)** — returns an empty vector for legacy contracts deployed before the introduction of the investor index, ensuring backward compatibility.

