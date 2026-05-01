# Phase 3: Bank Accounts & Credit Cards - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution
> agents. Decisions are captured in CONTEXT.md — this log preserves the
> alternatives considered.

**Date:** 2026-04-25 **Phase:** 03-bank-accounts-credit-cards **Areas
discussed:** Account type model, Credit-card field storage, Balance before Phase
4, Unified list UX + institution

---

## Account type model

### Q1: Which new AccountType values should Phase 3 add?

| Option                                       | Description                             | Selected |
| -------------------------------------------- | --------------------------------------- | -------- |
| CHECKING, SAVINGS, CREDIT_CARD (recommended) | Three leaf values; 1:1 with ACCT-01/02. |          |
| BANK + subtype                               | Single BANK type with subtype column.   |          |
| CHECKING, SAVINGS, CREDIT_CARD, LOAN         | Adds LOAN proactively. Scope expansion. | ✓        |

**User's choice:** CHECKING, SAVINGS, CREDIT_CARD, LOAN **Notes:** LOAN was
added beyond ACCT-01..07 scope. Clarified in Q2.

### Q2: What do you want from LOAN in Phase 3?

| Option                       | Description                                              | Selected |
| ---------------------------- | -------------------------------------------------------- | -------- |
| Enum slot only (recommended) | Basic CRUD, liability via helper, no principal/interest. | ✓        |
| Full loan tracking           | Principal, interest, amortization. Large scope.          |          |
| Drop LOAN                    | Defer entirely.                                          |          |

**User's choice:** Enum slot only **Notes:** Confirms LOAN is minimal; rich loan
tracking deferred to a future phase.

### Q3: How should asset-vs-liability semantics be expressed?

| Option                               | Description                                | Selected  |
| ------------------------------------ | ------------------------------------------ | --------- | ------------------------------ | --- |
| Derived helper in core (recommended) | `account_kind(AccountType) -> Asset        | Liability | Investment`. No schema change. | ✓   |
| Explicit column on accounts          | `is_liability` or `account_family` column. |           |
| Use existing `group` field           | Rely on user convention.                   |           |

**User's choice:** Derived helper in core

### Q4: Default tracking_mode for new bank or credit card account?

| Option                     | Description                                    | Selected |
| -------------------------- | ---------------------------------------------- | -------- |
| TRANSACTIONS (recommended) | Matches long-term model; unused until Phase 4. | ✓        |
| HOLDINGS                   | Treat like existing CASH accounts.             |          |
| NOT_SET until user picks   | Force user choice.                             |          |

**User's choice:** TRANSACTIONS

### Q5: What happens to existing CASH accounts?

| Option                    | Description                        | Selected |
| ------------------------- | ---------------------------------- | -------- |
| Leave as-is (recommended) | CHECKING and SAVINGS are additive. | ✓        |
| Migrate CASH -> CHECKING  | Auto-upgrade. Ambiguous.           |          |
| Deprecate CASH            | Remove entirely. Risky.            |          |

**User's choice:** Leave as-is

---

## Credit-card field storage

### Q1: Where should credit-card fields live in the schema?

| Option                                     | Description                           | Selected |
| ------------------------------------------ | ------------------------------------- | -------- |
| Nullable columns on accounts (recommended) | Diesel-typed columns, simple queries. | ✓        |
| Side table credit_card_details             | 1:1 JOIN for every CC query.          |          |
| meta JSONB on accounts                     | No schema change, no type safety.     |          |

**User's choice:** Nullable columns on accounts

### Q2: How are statement-balance / min-payment / due-date stored?

| Option                                           | Description                                            | Selected |
| ------------------------------------------------ | ------------------------------------------------------ | -------- |
| Current snapshot on account (recommended for v1) | Latest values on account row. User overwrites monthly. | ✓        |
| Separate statements table with history           | Per-cycle rows. Larger scope.                          |          |
| Defer entirely                                   | Skip statement tracking. Violates ACCT-06.             |          |

**User's choice:** Current snapshot on account

### Q3: How is credit utilization % computed?

| Option                           | Description                     | Selected |
| -------------------------------- | ------------------------------- | -------- |
| Derived on the fly (recommended) | `balance / credit_limit * 100`. | ✓        |
| Stored column updated on write   | Denormalized, can drift.        |          |

**User's choice:** Derived on the fly

### Q4: Reward tracking granularity for CREDIT_CARD in Phase 3?

| Option                                   | Description                                           | Selected |
| ---------------------------------------- | ----------------------------------------------------- | -------- |
| Single balance field (recommended)       | `reward_points_balance` + `cashback_balance`, manual. | ✓        |
| Points + cashback + category multipliers | Rules engine. Out of scope.                           |          |
| Drop rewards from Phase 3                | Defer ACCT-07.                                        |          |

**User's choice:** Single balance field

---

## Balance before Phase 4

### Q1: Where does the displayed balance come from for a bank or CC account in Phase 3?

| Option                                                         | Description                                                                              | Selected |
| -------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | -------- |
| Opening balance + manual 'update balance' action (recommended) | opening_balance stored at creation; manual update mutates current_balance until Phase 4. | ✓        |
| HOLDINGS tracking_mode until Phase 4                           | Reuse existing HOLDINGS path. Inverts Area 1 default.                                    |          |
| Defer balance display to Phase 4                               | Violates ACCT-03.                                                                        |          |

**User's choice:** Opening balance + manual update

### Q2: How is opening_balance stored on the account?

| Option                                         | Description                        | Selected |
| ---------------------------------------------- | ---------------------------------- | -------- |
| Dedicated opening_balance column (recommended) | Stored once at creation.           | ✓        |
| Synthetic 'opening' transaction in Phase 4     | Fails Phase 3 balance requirement. |          |
| Just a current_balance column user mutates     | No audit trail.                    |          |

**User's choice:** Dedicated opening_balance column

### Q3: How is the credit-card balance sign represented?

| Option                                                      | Description                       | Selected |
| ----------------------------------------------------------- | --------------------------------- | -------- |
| Positive value, CREDIT_CARD implies liability (recommended) | 500.00 for $500 owed.             | ✓        |
| Negative value for debt                                     | Breaks 'current balance' display. |          |

**User's choice:** Positive value with derived liability semantics

### Q4: When Phase 4 ships, migration story for Phase-3-era bank/CC accounts?

| Option                                                                       | Description                                                                                | Selected |
| ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ | -------- |
| Auto-generate 'Opening Balance' transaction + delta adjustment (recommended) | Phase 4 materializes opening_balance and any manual delta as transactions on first insert. | ✓        |
| Users reconcile manually in Phase 4                                          | More friction.                                                                             |          |
| Decide in Phase 4                                                            | Risk of being constrained by production data.                                              |          |

**User's choice:** Auto-generate Opening Balance transaction

---

## Unified list UX + institution

### Q1: How should the unified account list be structured?

| Option                                                  | Description                              | Selected |
| ------------------------------------------------------- | ---------------------------------------- | -------- |
| Extend existing accounts-summary grouping (recommended) | Pre-seed default groups per AccountType. | ✓        |
| Asset / Debt / Investment top-level sections            | Bigger UI change.                        |          |
| Full new 'Accounts' page with tabs                      | Largest redesign.                        |          |

**User's choice:** Extend existing grouping

### Q2: What balance info per row in the unified list?

| Option                                           | Description                                                                              | Selected |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------- | -------- |
| Current balance + account currency (recommended) | Name, institution, balance in account currency, FX base equiv, CC available-credit chip. | ✓        |
| Current balance only (base currency)             | Hide account currency.                                                                   |          |
| Rich row with CC metadata inline                 | Too dense.                                                                               |          |

**User's choice:** Current balance + account currency

### Q3: Where does 'institution' (bank/issuer name) live?

| Option                                            | Description                        | Selected |
| ------------------------------------------------- | ---------------------------------- | -------- |
| Free-text string column on accounts (recommended) | `institution VARCHAR` on accounts. | ✓        |
| Reuse platform_id (new platform rows)             | Overloads broker concept.          |          |
| New institutions table with lookup                | More work than Phase 3 needs.      |          |

**User's choice:** Free-text string column

### Q4: How does archiving surface in the UI?

| Option                                            | Description                                      | Selected |
| ------------------------------------------------- | ------------------------------------------------ | -------- |
| Hidden by default, toggle to reveal (recommended) | Excluded from selectors; "Show archived" toggle. | ✓        |
| Grey-ed in main list                              | Clutters list.                                   |          |
| Separate Archive page                             | Extra navigation.                                |          |

**User's choice:** Hidden by default, toggle to reveal

---

## Claude's Discretion

Deferred to downstream researcher / planner / executor:

- Precise Diesel column precision/scale and CHECK constraints.
- Service-layer validation rules for CC-only nullability and per-field ranges.
- UI copy, button labels, empty-state wording.
- Exact placement of the "Update balance" action, available-credit chip styling,
  and "Show archived" toggle.
- Whether new IPC commands (e.g., `update_account_balance`) are introduced or
  existing `update_account` is extended.
- Migration file numbering and schema.rs regeneration sequence (follow Phase 2
  pattern).

## Deferred Ideas

Ideas mentioned during discussion that are noted for future phases:

- Full loan tracking (principal, APR, amortization).
- Statement history table with per-cycle rows (Phase 7 foundation).
- Rewards rules engine (category multipliers, earning rules).
- Institutions lookup table / autocomplete.
- Dedicated `/accounts/archive` page.
- Balance reconciliation tooling (Phase 4+).
- Bank API sync (Plaid, etc.) — out of scope per PROJECT.md.
