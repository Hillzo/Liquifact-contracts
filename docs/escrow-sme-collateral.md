# Escrow — SME Collateral Commitment

## Overview

The LiquiFact escrow contract supports **metadata-only** collateral pledge recording.
No tokens are moved or reserved by these operations; they exist solely for indexers
and dashboards to surface off-chain pledge intent alongside an invoice's on-chain state.

---

## Entrypoints

### `record_sme_collateral_commitment(env, amount) -> Result<(), EscrowError>`

Records an off-chain collateral pledge against the escrow's invoice.

- **Auth**: SME address (`sme_address` from the escrow record).
- **Storage**: writes `DataKey::SmeCollateralPledge` (instance storage).
- **Event**: emits `CollateralRecordedEvt { invoice_id, amount }` under topic
  `(col_rec, sme_address)`.
- **Idempotency**: calling again overwrites the previous amount.
- **Token movement**: none.

### `get_sme_collateral_commitment(env) -> Option<CollateralPledge>`

Returns the current pledge record, or `None` if none has been recorded (or it was cleared).

- **Auth**: none required (read-only).

### `clear_sme_collateral_commitment(env) -> Result<(), EscrowError>`

Retires a previously recorded pledge, removing it from storage.

- **Auth**: SME address (`sme_address` from the escrow record).
- **Error**: `EscrowError::NoCollateralToClear` (code 4) if no pledge exists.
- **Storage**: removes `DataKey::SmeCollateralPledge`.
- **Event**: emits `CollateralClearedEvt { invoice_id, amount }` under topic
  `(col_clr, sme_address)`, where `amount` is the value from the removed record.
- **Token movement**: none.

---

## Guard ordering (ADR-002)

`clear_sme_collateral_commitment` applies guards in this order to keep auth
checks from masking informative errors:

1. **Read-only existence check** — return `NoCollateralToClear` immediately if
   `DataKey::SmeCollateralPledge` is absent (no auth consumed).
2. **`require_auth`** — assert the caller is the SME address.
3. **Mutation** — remove the storage entry and emit `CollateralClearedEvt`.

---

## Data types

```rust
pub struct CollateralPledge {
    pub invoice_id: Symbol,
    pub amount: i128,
}

pub struct CollateralRecordedEvt {
    pub invoice_id: Symbol,
    pub amount: i128,
}

pub struct CollateralClearedEvt {
    pub invoice_id: Symbol,
    pub amount: i128,   // carried from the pledge at the time of removal
}
```

---

## Error codes

| Code | Variant              | Trigger                                            |
|------|----------------------|----------------------------------------------------|
| 1    | `NotInitialized`     | Escrow not yet created via `init`                  |
| 2    | `NotOpen`            | Reserved for future status guards                  |
| 3    | `NotFunded`          | Reserved for future status guards                  |
| 4    | `NoCollateralToClear`| `clear_sme_collateral_commitment` with no pledge   |

---

## Security notes

- **Metadata-only**: neither `record_sme_collateral_commitment` nor
  `clear_sme_collateral_commitment` transfers or locks tokens.
- **SME-only writes**: all mutating operations require `sme_address.require_auth()`.
- **No status dependency**: collateral metadata can be cleared regardless of escrow
  status (open / funded / settled), allowing clean-up after settlement or cancellation.
- **No double-clear risk**: the existence check on entry ensures a second clear call
  returns `NoCollateralToClear` rather than silently succeeding.

---

## Example flow

```
SME calls record_sme_collateral_commitment(5_000_0000000)
  → DataKey::SmeCollateralPledge stored
  → CollateralRecordedEvt emitted

[invoice settled off-chain; pledge released]

SME calls clear_sme_collateral_commitment()
  → DataKey::SmeCollateralPledge removed
  → CollateralClearedEvt { invoice_id: "INV001", amount: 5_000_0000000 } emitted
```
