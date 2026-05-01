# Phase 3: Bank Accounts & Credit Cards - Pattern Map

**Mapped:** 2026-04-25 **Files analyzed:** 27 (extend) + 7 (new) = 34 **Analogs
found:** 32 / 34 (2 cousins for new artifacts)

This map gives the planner concrete code excerpts to copy, with file:line refs.
Every "extend" file already exists in-repo and has its own analog (itself).
Every "NEW" file references the closest cousin and explains how to mirror it.

Decisions in scope: D-01..D-19 plus assumptions A1 (TEXT-money) and A2 (reuse
`update_account` for balance updates) from RESEARCH.md.

---

## File Classification

### Domain group 1 — Rust core (extend)

| File                                               | Role            | Data Flow        | Closest Analog        | Match Quality |
| -------------------------------------------------- | --------------- | ---------------- | --------------------- | ------------- |
| `crates/core/src/accounts/accounts_model.rs`       | model           | request-response | itself (lines 22-123) | exact         |
| `crates/core/src/accounts/accounts_constants.rs`   | utility         | transform        | itself (lines 1-25)   | exact         |
| `crates/core/src/accounts/accounts_service.rs`     | service         | request-response | itself (lines 45-126) | exact         |
| `crates/core/src/accounts/accounts_traits.rs`      | trait/interface | n/a              | itself (lines 17-91)  | exact         |
| `crates/core/src/accounts/accounts_model_tests.rs` | test (unit)     | n/a              | itself (lines 1-96)   | exact         |

### Domain group 2 — Rust core (NEW)

| File                                                 | Role               | Data Flow        | Closest Analog                                                                                                                    | Match Quality |
| ---------------------------------------------------- | ------------------ | ---------------- | --------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `crates/core/src/accounts/accounts_service_tests.rs` | test (unit, async) | request-response | `crates/core/src/accounts/accounts_model_tests.rs` (test layout) + `crates/core/src/accounts/accounts_service.rs` (service shape) | role-match    |

### Domain group 3 — Storage-postgres (extend)

| File                                                 | Role                | Data Flow | Closest Analog                       | Match Quality |
| ---------------------------------------------------- | ------------------- | --------- | ------------------------------------ | ------------- |
| `crates/storage-postgres/src/accounts/model.rs`      | model               | CRUD      | itself (lines 12-123)                | exact         |
| `crates/storage-postgres/src/accounts/repository.rs` | repository          | CRUD      | itself (lines 27-137)                | exact         |
| `crates/storage-postgres/src/schema.rs`              | config (Diesel DSL) | n/a       | itself (lines 12-31, accounts table) | exact         |

### Domain group 4 — Storage-postgres (NEW)

| File                                                                                            | Role                    | Data Flow | Closest Analog                                                                                                                                                                                                      | Match Quality         |
| ----------------------------------------------------------------------------------------------- | ----------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql`   | migration               | DDL       | `crates/storage-postgres/migrations/20260422000000_auth_users/up.sql` (additive pattern) + `crates/storage-postgres/migrations/20260101000000_initial_schema/up.sql:32-49` (accounts table money-column convention) | role-match (additive) |
| `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql` | migration               | DDL       | `crates/storage-postgres/migrations/20260422000000_auth_users/down.sql`                                                                                                                                             | exact                 |
| `crates/storage-postgres/src/accounts/repository_tests.rs`                                      | test (integration, PG)  | CRUD      | none in `accounts/` — closest cousin is `crates/storage-postgres/src/fx/model.rs:46-111` for Decimal round-trip pattern                                                                                             | partial               |
| `crates/storage-postgres/src/accounts/migration_tests.rs`                                       | test (integration, DDL) | DDL       | none — uses Diesel's embedded migration runner; pattern derived from `crates/storage-postgres/src/db/mod.rs:57-77`                                                                                                  | partial               |

### Domain group 5 — Backend HTTP (Axum, extend)

| File                              | Role       | Data Flow        | Closest Analog       | Match Quality               |
| --------------------------------- | ---------- | ---------------- | -------------------- | --------------------------- |
| `apps/server/src/models.rs`       | DTO        | request-response | itself (lines 6-147) | exact                       |
| `apps/server/src/api/accounts.rs` | controller | request-response | itself (lines 22-74) | exact (no signature change) |

### Domain group 6 — Frontend constants & types (extend)

| File                                      | Role          | Data Flow        | Closest Analog       | Match Quality                                  |
| ----------------------------------------- | ------------- | ---------------- | -------------------- | ---------------------------------------------- |
| `apps/frontend/src/lib/constants.ts`      | utility       | transform        | itself (lines 44-72) | exact                                          |
| `apps/frontend/src/lib/types/account.ts`  | model (TS)    | n/a              | itself (lines 7-25)  | exact                                          |
| `apps/frontend/src/lib/schemas.ts`        | utility (zod) | transform        | itself (lines 78-96) | exact                                          |
| `apps/frontend/src/hooks/use-accounts.ts` | hook          | request-response | itself (lines 1-33)  | exact (no change needed; D-19 already correct) |

### Domain group 7 — Frontend adapters (verify-only or extend)

| File                                            | Role                 | Data Flow        | Closest Analog       | Match Quality                                           |
| ----------------------------------------------- | -------------------- | ---------------- | -------------------- | ------------------------------------------------------- |
| `apps/frontend/src/adapters/shared/accounts.ts` | adapter              | request-response | itself (lines 22-68) | exact (cascades from schema; only payload-shape change) |
| `apps/frontend/src/adapters/web/core.ts`        | config (command map) | request-response | itself (lines 38-42) | exact (no new commands per A2)                          |

### Domain group 8 — Frontend selectors / launchers (extend, exhaustive Record fix)

| File                                                       | Role      | Data Flow    | Closest Analog                                  | Match Quality            |
| ---------------------------------------------------------- | --------- | ------------ | ----------------------------------------------- | ------------------------ |
| `apps/frontend/src/components/account-selector.tsx`        | component | event-driven | itself (lines 26-31)                            | exact                    |
| `apps/frontend/src/components/account-selector-mobile.tsx` | component | event-driven | itself (lines 24-29, 110-123)                   | exact                    |
| `apps/frontend/src/components/app-launcher.tsx`            | component | event-driven | itself (lines 65-70) — **EXHAUSTIVE TS Record** | exact (compile-blocking) |

### Domain group 9 — Frontend pages (extend)

| File                                                                          | Role              | Data Flow        | Closest Analog                                  | Match Quality                                                         |
| ----------------------------------------------------------------------------- | ----------------- | ---------------- | ----------------------------------------------- | --------------------------------------------------------------------- |
| `apps/frontend/src/pages/dashboard/accounts-summary.tsx`                      | page (component)  | request-response | itself                                          | exact (verify only — group-by already routes through `account.group`) |
| `apps/frontend/src/pages/account/account-page.tsx`                            | page              | request-response | itself (lines 82-86) — **EXHAUSTIVE TS Record** | exact (compile-blocking + add CC sections)                            |
| `apps/frontend/src/pages/settings/accounts/accounts-page.tsx`                 | page (HOST)       | request-response | itself (lines 24-296)                           | exact (extend with group-by + CC chips per UI-SPEC §1)                |
| `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx` | component (modal) | request-response | itself (lines 13-37)                            | exact (extend hard-coded type cast at line 20)                        |
| `apps/frontend/src/pages/settings/accounts/components/account-form.tsx`       | component (form)  | request-response | itself (lines 47-51, 184-204)                   | exact (extend type list + add dynamic CC fields)                      |
| `apps/frontend/src/pages/settings/accounts/components/account-item.tsx`       | component         | request-response | itself (lines 10-26, 127-138)                   | exact (extend icon map + CC chip)                                     |

### Domain group 10 — Frontend tests (NEW)

| File                                                               | Role             | Data Flow        | Closest Analog                                                                                                | Match Quality |
| ------------------------------------------------------------------ | ---------------- | ---------------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `apps/frontend/src/lib/constants.test.ts`                          | test (unit)      | n/a              | none in `lib/` — pattern from any vitest spec; planner can generate from RESEARCH.md §Validation Architecture | partial       |
| `apps/frontend/src/lib/schemas.test.ts`                            | test (unit)      | n/a              | none — same situation                                                                                         | partial       |
| `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` | test (component) | request-response | none — closest cousin is `accounts-summary.test.tsx` (per RESEARCH.md mention)                                | partial       |

### Domain group 11 — E2E (NEW)

| File                      | Role                  | Data Flow        | Closest Analog                                                                                                                             | Match Quality |
| ------------------------- | --------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ------------- |
| `e2e/11-accounts.spec.ts` | test (E2E Playwright) | request-response | `e2e/05-form-validation.spec.ts` (lean structure, login + form flows) and `e2e/01-happy-path.spec.ts` (data tables + multi-account create) | role-match    |

---

## Pattern Assignments

> Convention: every excerpt below is verbatim from the cited file:line range.
> The planner instructs the executor to mirror the structure exactly,
> substituting only the new field names (NewAccount/Account/AccountUpdate gain
> 11 fields; AccountDB and DTOs gain the same 11; schema.rs gains 11 column
> declarations).

### `crates/core/src/accounts/accounts_model.rs` (model, request-response)

**Analog:** itself.

**Imports + struct shape pattern** (lines 1-47):

```rust
//! Account domain models.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{errors::ValidationError, Error, Result};

/// Tracking mode for an account - determines how holdings are tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingMode {
    Transactions,
    Holdings,
    #[default]
    NotSet,
}

/// Domain model representing an account in the system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub is_archived: bool,
    pub tracking_mode: TrackingMode,
}
```

Add new fields after `tracking_mode`. Per assumption A1 (TEXT money) all
Decimal-typed fields are `Option<rust_decimal::Decimal>` at the domain level and
serialize through TEXT in storage:

```rust
// NEW fields to append (D-06, D-11, D-12, D-18) — keep types consistent with
// existing core/Cargo.toml dep `rust_decimal = { workspace = true }`
pub institution: Option<String>,
pub opening_balance: Option<rust_decimal::Decimal>,
pub current_balance: Option<rust_decimal::Decimal>,
pub balance_updated_at: Option<NaiveDateTime>,
pub credit_limit: Option<rust_decimal::Decimal>,
pub statement_cycle_day: Option<i16>,
pub statement_balance: Option<rust_decimal::Decimal>,
pub minimum_payment: Option<rust_decimal::Decimal>,
pub statement_due_date: Option<chrono::NaiveDate>,
pub reward_points_balance: Option<i32>,
pub cashback_balance: Option<rust_decimal::Decimal>,
```

**Existing validate() pattern to extend** (lines 72-87):

```rust
impl NewAccount {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account name cannot be empty".to_string(),
            )));
        }
        if self.currency.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Currency cannot be empty".to_string(),
            )));
        }
        Ok(())
    }
}
```

Extend exact pattern: add CC-gated rules from RESEARCH.md §Validation Layer
(account_types::CREDIT_CARD branch). Use `Decimal::ZERO` (already in
`rust_decimal` prelude in `fx/model.rs:60`).

---

### `crates/core/src/accounts/accounts_constants.rs` (utility, transform)

**Analog:** itself.

**Existing constants + helper** (lines 1-25):

```rust
/// Default account type for new accounts
pub const DEFAULT_ACCOUNT_TYPE: &str = "SECURITIES";

/// Account type constants
pub mod account_types {
    pub const SECURITIES: &str = "SECURITIES";
    pub const CASH: &str = "CASH";
    pub const CRYPTOCURRENCY: &str = "CRYPTOCURRENCY";
}

pub fn default_group_for_account_type(account_type: &str) -> &'static str {
    match account_type {
        account_types::SECURITIES => "Investments",
        account_types::CASH => "Cash",
        account_types::CRYPTOCURRENCY => "Crypto",
        _ => "Investments",
    }
}
```

Extend `account_types` mod with 4 new `pub const` entries (CHECKING, SAVINGS,
CREDIT_CARD, LOAN). Extend the match in `default_group_for_account_type` per
D-16. Append the new `AccountKind` enum + `account_kind` helper from RESEARCH.md
§Derived Helpers — `account_kind()`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind { Asset, Liability, Investment }

pub fn account_kind(account_type: &str) -> AccountKind {
    match account_type {
        account_types::CHECKING | account_types::SAVINGS | account_types::CASH => AccountKind::Asset,
        account_types::CREDIT_CARD | account_types::LOAN => AccountKind::Liability,
        account_types::SECURITIES | account_types::CRYPTOCURRENCY => AccountKind::Investment,
        _ => AccountKind::Asset,
    }
}
```

`mod.rs:9` already does `pub use accounts_constants::*;`, so the new
`AccountKind` and `account_kind` are auto-exported.

---

### `crates/core/src/accounts/accounts_service.rs` (service, request-response)

**Analog:** itself.

**Imports pattern** (lines 1-12):

```rust
use log::{debug, info, warn};
use std::sync::{Arc, RwLock};

use super::accounts_model::{Account, AccountUpdate, NewAccount};
use super::accounts_traits::{AccountRepositoryTrait, AccountServiceTrait};
use crate::assets::AssetRepositoryTrait;
use crate::errors::Result;
use crate::events::{CurrencyChange, DomainEvent, DomainEventSink};
use crate::fx::FxServiceTrait;
use crate::quotes::sync_state::SyncStateStore;
```

**Update method shape (read existing → mutate → write)** (lines 80-126):

```rust
async fn update_account(&self, account_update: AccountUpdate) -> Result<Account> {
    let account_id = account_update.id.as_ref().ok_or_else(|| {
        crate::Error::Validation(crate::errors::ValidationError::InvalidInput(
            "Account ID is required".to_string(),
        ))
    })?;
    let existing = self.repository.get_by_id(account_id).await?;

    let result = self.repository.update(account_update).await?;
    // ... event emission ...
    Ok(result)
}
```

**Phase 3 D-12 extension (balance auto-bump per A2):** insert before
`self.repository.update(...)`:

```rust
// D-12: auto-stamp balance_updated_at when current_balance changes.
let mut account_update = account_update;
let new_bal = account_update.current_balance.clone();
if new_bal.is_some() && new_bal != existing.current_balance.clone() {
    account_update.balance_updated_at =
        Some(chrono::Utc::now().naive_utc());
}
```

(Field types for `current_balance` and `balance_updated_at` mirror domain model
additions.)

---

### `crates/core/src/accounts/accounts_traits.rs` (trait/interface, n/a)

**Analog:** itself. **No signature change required** (RESEARCH.md confirmed).

Existing trait shape to leave untouched (lines 17-46):

```rust
#[async_trait]
pub trait AccountRepositoryTrait: Send + Sync {
    async fn create(&self, new_account: NewAccount) -> Result<Account>;
    async fn update(&self, account_update: AccountUpdate) -> Result<Account>;
    async fn delete(&self, account_id: &str) -> Result<usize>;
    async fn get_by_id(&self, account_id: &str) -> Result<Account>;
    async fn list(
        &self,
        is_active_filter: Option<bool>,
        is_archived_filter: Option<bool>,
        account_ids: Option<&[String]>,
    ) -> Result<Vec<Account>>;
}
```

The new fields ride through `NewAccount` / `AccountUpdate` / `Account` as struct
data — the trait surface is invariant.

---

### `crates/core/src/accounts/accounts_model_tests.rs` (test, n/a)

**Analog:** itself.

**Helper-and-test pattern to extend** (lines 76-95):

```rust
fn create_test_account(tracking_mode: TrackingMode) -> Account {
    Account {
        id: "test-account-id".to_string(),
        name: "Test Account".to_string(),
        account_type: "SECURITIES".to_string(),
        group: None,
        currency: "USD".to_string(),
        is_default: false,
        is_active: true,
        created_at: NaiveDateTime::default(),
        updated_at: NaiveDateTime::default(),
        platform_id: None,
        account_number: None,
        meta: None,
        provider: None,
        provider_account_id: None,
        is_archived: false,
        tracking_mode,
    }
}
```

**Required updates:**

1. Add the 11 new fields to this struct literal (else all dependent tests stop
   compiling — `Account: Default` exists at line 22 of `accounts_model.rs`, so
   easier path: replace literal with
   `Account { name: ..., account_type: ..., ..Default::default() }`).
2. Add Wave-0 tests per RESEARCH.md §Wave 0 Gaps:
   - `test_new_account_validate_bank`
   - `test_new_account_validate_credit_card`
   - `test_new_account_validate_credit_card_rejects_invalid`
   - `test_new_account_validate_non_cc_rejects_cc_fields`
   - `test_account_kind` (per AccountKind matrix in RESEARCH.md §Derived
     Helpers)
   - `test_default_group_for_new_types` (D-16 mapping)

---

### `crates/core/src/accounts/accounts_service_tests.rs` (test, NEW — async)

**Analog:** `accounts_model_tests.rs` (test layout) + `accounts_service.rs`
(constructor signature). No existing service-level test file in the accounts
module.

**Cousin pattern from `accounts_service.rs:24-43`:**

```rust
impl AccountService {
    pub fn new(
        repository: Arc<dyn AccountRepositoryTrait>,
        fx_service: Arc<dyn FxServiceTrait>,
        base_currency: Arc<RwLock<String>>,
        event_sink: Arc<dyn DomainEventSink>,
        asset_repository: Arc<dyn AssetRepositoryTrait>,
        sync_state_store: Arc<dyn SyncStateStore>,
    ) -> Self { Self { repository, fx_service, base_currency, event_sink, asset_repository, sync_state_store } }
}
```

**Pattern to follow:**

1. Use `#[tokio::test]` (RESEARCH.md §Validation Architecture confirms harness).
2. Build mock implementations of every `AccountService::new` dependency (look in
   `crates/core/src/` for existing mocks — if absent, hand-roll simple in-memory
   mocks per trait surface).
3. Coverage focus per VALIDATION.md Wave 0:
   - `test_update_bumps_balance_timestamp` — verify `balance_updated_at`
     auto-bumps when `current_balance` changes.
   - `test_update_no_bump_when_balance_unchanged` — verify timestamp is
     untouched when balance is the same.
4. Mirror module pattern from `accounts_model_tests.rs:1-9`:
   ```rust
   #[cfg(test)]
   mod tests {
       use crate::accounts::{...};
       // tests
   }
   ```

---

### `crates/storage-postgres/src/accounts/model.rs` (model, CRUD)

**Analog:** itself.

**`AccountDB` Diesel struct pattern** (lines 8-32):

```rust
#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountDB {
    #[diesel(column_name = id)]
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    #[diesel(skip_insertion)]
    pub created_at: NaiveDateTime,
    #[diesel(skip_insertion)]
    pub updated_at: NaiveDateTime,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub is_archived: bool,
    pub tracking_mode: String,
}
```

Append new DB-typed fields. Per assumption A1 (TEXT-serialized Decimals), Diesel
field types are:

```rust
pub institution: Option<String>,
pub opening_balance: Option<String>,        // TEXT-serialized Decimal
pub current_balance: Option<String>,        // TEXT-serialized Decimal
pub balance_updated_at: Option<NaiveDateTime>,
pub credit_limit: Option<String>,
pub statement_cycle_day: Option<i16>,       // SMALLINT
pub statement_balance: Option<String>,
pub minimum_payment: Option<String>,
pub statement_due_date: Option<chrono::NaiveDate>,
pub reward_points_balance: Option<i32>,     // INTEGER
pub cashback_balance: Option<String>,
```

**`From<AccountDB> for Account` pattern** (lines 34-60):

```rust
impl From<AccountDB> for Account {
    fn from(db: AccountDB) -> Self {
        let tracking_mode = match db.tracking_mode.as_str() {
            "TRANSACTIONS" => TrackingMode::Transactions,
            "HOLDINGS" => TrackingMode::Holdings,
            _ => TrackingMode::NotSet,
        };
        Self {
            id: db.id,
            name: db.name,
            // ...
            tracking_mode,
        }
    }
}
```

For each new TEXT-decimal field copy the **canonical Decimal deserialization
pattern from `crates/storage-postgres/src/fx/model.rs:55-71`:**

```rust
use rust_decimal::Decimal;
use std::str::FromStr;

open: db
    .open
    .as_deref()
    .and_then(|s| Decimal::from_str(s).ok())
    .unwrap_or(Decimal::ZERO),
close: Decimal::from_str(&db.close).unwrap_or(Decimal::ZERO),
```

For our optional fields (none mandatory, all nullable), use:

```rust
opening_balance: db
    .opening_balance
    .as_deref()
    .and_then(|s| Decimal::from_str(s).ok()),
```

(returns `Option<Decimal>` — empty/parse-fail collapses to `None`, which is
correct for "user hasn't supplied" semantics).

**`From<NewAccount> for AccountDB` pattern** (lines 62-90):

```rust
impl From<NewAccount> for AccountDB {
    fn from(domain: NewAccount) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let tracking_mode = match domain.tracking_mode {
            TrackingMode::Transactions => "TRANSACTIONS",
            TrackingMode::Holdings => "HOLDINGS",
            TrackingMode::NotSet => "NOT_SET",
        }
        .to_string();
        Self {
            id: domain.id.unwrap_or_default(),
            name: domain.name,
            // ...
            tracking_mode,
        }
    }
}
```

For each Decimal: serialize via `Decimal::to_string` — see canonical pattern at
`fx/model.rs:99-103`:

```rust
open: Some(quote.open.to_string()),
close: quote.close.to_string(),
```

For optional fields:

```rust
opening_balance: domain.opening_balance.as_ref().map(|d| d.to_string()),
```

**`From<AccountUpdate> for AccountDB` pattern** (lines 92-122): Mirror the
existing impl. Per landmine 5 (RESEARCH.md), planner must decide whether new
fields are "preserved on update" (read from DB if absent in payload, like
currency at `repository.rs:61`) or "always overwritten." **Recommendation per
UI-SPEC §5 + Discussion:** new CC fields are "always-overwritable" — they flow
straight through from the update DTO. Do NOT add new preserve-from-existing
branches in `repository.rs:61-75`.

---

### `crates/storage-postgres/src/accounts/repository.rs` (repository, CRUD)

**Analog:** itself. **No method-signature change required.**

**`create` shape to leave untouched** (lines 27-43):

```rust
async fn create(&self, new_account: NewAccount) -> Result<Account> {
    new_account.validate()?;
    let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
    let mut account_db: AccountDB = new_account.into();
    account_db.id = Uuid::now_v7().to_string();
    diesel::insert_into(accounts::table)
        .values(&account_db)
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;
    Ok(account_db.into())
}
```

The validate-before-insert pattern (line 30) is the single place CC-field
validation runs (per RESEARCH.md §Validation Layer "Called by
`repository.rs:30`").

**`update` preserve-from-existing pattern** (lines 45-84):

```rust
let existing = accounts::table
    .select(AccountDB::as_select())
    .find(&account_db.id)
    .first::<AccountDB>(&mut conn)
    .await
    .map_err(StoragePgError::from)?;

account_db.currency = existing.currency;       // immutable after create
account_db.created_at = existing.created_at;
account_db.updated_at = chrono::Utc::now().naive_utc();
account_db.provider_account_id = existing.provider_account_id;
account_db.platform_id = existing.platform_id;
account_db.provider = existing.provider;
account_db.account_number = existing.account_number;
account_db.meta = existing.meta;

if !is_archived_provided { account_db.is_archived = existing.is_archived; }
if !tracking_mode_provided { account_db.tracking_mode = existing.tracking_mode; }
```

Per planner decision above (always-overwritable for new CC fields), do NOT add
new preserve branches here. The existing `From<AccountUpdate> for AccountDB`
already maps them through.

---

### `crates/storage-postgres/src/schema.rs` (config, n/a)

**Analog:** itself.

**Existing accounts DSL** (lines 12-31):

```rust
diesel::table! {
    accounts (id) {
        id -> Text,
        name -> Text,
        account_type -> Text,
        group -> Nullable<Text>,
        currency -> Text,
        is_default -> Bool,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        platform_id -> Nullable<Text>,
        account_number -> Nullable<Text>,
        meta -> Nullable<Text>,
        provider -> Nullable<Text>,
        provider_account_id -> Nullable<Text>,
        is_archived -> Bool,
        tracking_mode -> Text,
    }
}
```

Append (in same block, after `tracking_mode`):

```rust
        institution -> Nullable<Text>,
        opening_balance -> Nullable<Text>,
        current_balance -> Nullable<Text>,
        balance_updated_at -> Nullable<Timestamp>,
        credit_limit -> Nullable<Text>,
        statement_cycle_day -> Nullable<SmallInt>,
        statement_balance -> Nullable<Text>,
        minimum_payment -> Nullable<Text>,
        statement_due_date -> Nullable<Date>,
        reward_points_balance -> Nullable<Integer>,
        cashback_balance -> Nullable<Text>,
```

`SmallInt` and `Date` types are already used elsewhere in this file
(`daily_account_valuation.valuation_date -> Date` at line 209). No new
Diesel-type imports required.

Per RESEARCH.md landmine 4: `schema.rs` is hand-synchronized — do not skip this
step. Compile will fail if missing.

---

### Migration: `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql` (NEW)

**Analog:**
`crates/storage-postgres/migrations/20260422000000_auth_users/up.sql` (additive
new-table pattern; we adapt to ALTER TABLE for additive columns) +
`migrations/20260101000000_initial_schema/up.sql:32-49` (the exact accounts
table definition + money-column convention).

**Money-column convention (from initial schema header, lines 1-8):**

```sql
-- Initial PostgreSQL schema for WhaleIt
-- Consolidated migration creating all tables.
-- Mirrors SQLite schema with native PostgreSQL types:
--   - IDs: TEXT (UUID v7 stored as string for core compatibility)
--   - Booleans: BOOLEAN (native PG)
--   - Timestamps: TIMESTAMP (without timezone, maps to NaiveDateTime)
--   - Decimals: TEXT (serialized Rust Decimal)
--   - JSON: TEXT (serialized serde_json::Value)
```

**Existing accounts table (from initial schema, lines 32-49):**

```sql
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    "group" TEXT,
    currency TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    platform_id TEXT REFERENCES platforms(id),
    account_number TEXT,
    meta TEXT,
    provider TEXT,
    provider_account_id TEXT,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    tracking_mode TEXT NOT NULL DEFAULT 'NOT_SET'
);
```

**Existing additive-table example (auth_users up.sql in full):**

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    -- ...
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
-- ...
CREATE INDEX idx_users_email ON users(email);
```

**Phase 3 migration to write — apply the established conventions:**

```sql
-- Phase 3: Bank accounts, credit cards, and balance fields.
-- All money columns: TEXT (serialized rust_decimal::Decimal) — mirrors
-- the convention in 20260101000000_initial_schema/up.sql.

ALTER TABLE accounts
    ADD COLUMN institution TEXT,
    ADD COLUMN opening_balance TEXT,
    ADD COLUMN current_balance TEXT,
    ADD COLUMN balance_updated_at TIMESTAMP,
    ADD COLUMN credit_limit TEXT,
    ADD COLUMN statement_cycle_day SMALLINT
        CHECK (statement_cycle_day IS NULL OR (statement_cycle_day BETWEEN 1 AND 31)),
    ADD COLUMN statement_balance TEXT,
    ADD COLUMN minimum_payment TEXT,
    ADD COLUMN statement_due_date DATE,
    ADD COLUMN reward_points_balance INTEGER
        CHECK (reward_points_balance IS NULL OR reward_points_balance >= 0),
    ADD COLUMN cashback_balance TEXT;
```

Notes for the planner:

- Per RESEARCH.md §CHECK Constraints, the initial schema uses zero CHECK
  constraints on accounts. Phase 3 introduces TWO minimal CHECKs (cycle_day
  range, points >= 0) since these are integer-typed and DB-side enforcement is
  cheap. Other range checks (e.g. `credit_limit > 0`) live at the service layer
  because money is TEXT.
- No `account_type` whitelist CHECK is added — RESEARCH.md notes this is a
  "nice-to-have" only.
- Naming follows the timestamp+sequential convention
  (`YYYYMMDDHHMMSS_descriptive_name/`); 2026-04-25 + `000000` matches the
  existing `20260101000000_*` and `20260422000000_*` directories.
- `embed_migrations!()` at `crates/storage-postgres/src/db/mod.rs:15`
  auto-picks-up the new directory at server start.

### Migration: `.../20260425000000_accounts_extend_types_and_balances/down.sql` (NEW)

**Analog:** `migrations/20260422000000_auth_users/down.sql`:

```sql
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS verification_tokens;
DROP TABLE IF EXISTS users;
```

**Phase 3 down.sql:**

```sql
ALTER TABLE accounts
    DROP COLUMN IF EXISTS institution,
    DROP COLUMN IF EXISTS opening_balance,
    DROP COLUMN IF EXISTS current_balance,
    DROP COLUMN IF EXISTS balance_updated_at,
    DROP COLUMN IF EXISTS credit_limit,
    DROP COLUMN IF EXISTS statement_cycle_day,
    DROP COLUMN IF EXISTS statement_balance,
    DROP COLUMN IF EXISTS minimum_payment,
    DROP COLUMN IF EXISTS statement_due_date,
    DROP COLUMN IF EXISTS reward_points_balance,
    DROP COLUMN IF EXISTS cashback_balance;
```

The CHECK constraints DROP automatically with the columns (PG semantics).

---

### `crates/storage-postgres/src/accounts/repository_tests.rs` (test, NEW — integration)

**Analog:** No existing test under `crates/storage-postgres/src/accounts/`.
Closest cousins:

1. `crates/storage-postgres/src/fx/model.rs:46-111` — Decimal round-trip pattern
   (use this for assertion shape).
2. `crates/storage-postgres/src/db/mod.rs:57-77` — `run_migrations` runner (use
   this to set up test DB).
3. `crates/storage-postgres/src/accounts/repository.rs:21-25` — repository
   constructor (`PgAccountRepository::new(pool)`).

**Pattern outline for the planner:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_pool, run_migrations};

    async fn setup() -> Arc<PgAccountRepository> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL set");
        run_migrations(&url).await.unwrap();
        let pool = Arc::new(init_pool(&url).await.unwrap());
        Arc::new(PgAccountRepository::new(pool))
    }

    #[tokio::test]
    async fn test_create_credit_card_round_trip() {
        let repo = setup().await;
        let new = NewAccount { /* CHECKING/SAVINGS/CREDIT_CARD/LOAN with full new fields */ };
        let created = repo.create(new).await.unwrap();
        assert_eq!(created.account_type, "CREDIT_CARD");
        assert_eq!(created.credit_limit, Some(Decimal::new(1000000, 2)));
        // ...
    }

    #[tokio::test]
    async fn test_update_preserves_unrelated_fields() { /* ... */ }
}
```

VALIDATION.md Wave-0 acceptance: tests must pass
`cargo test -p whaleit-storage-postgres accounts::repository_tests` with
`DATABASE_URL` set.

Use the exact Decimal-from/to-string conversions from the production `From`
impls (mirror `fx/model.rs:55-71`).

### `crates/storage-postgres/src/accounts/migration_tests.rs` (test, NEW)

**Analog:** none. Pattern: invoke `run_migrations` against a fresh DB,
introspect `accounts` schema via `information_schema`. Lighter alternative: load
`AccountDB::as_select()` against an empty table and assert it compiles + queries
correctly (Diesel compile-check is the strongest schema integrity test).

**Recommendation:** keep this file minimal. One test is enough:

```rust
#[tokio::test]
async fn test_migration_runs_clean() {
    let url = std::env::var("DATABASE_URL").unwrap();
    crate::db::run_migrations(&url).await.unwrap();
    // SELECT * across new columns to prove they exist
}
```

---

### `apps/server/src/models.rs` (DTO, request-response)

**Analog:** itself.

**DTO + From<core::Account> pattern** (lines 6-54):

```rust
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    // ... existing fields
}

impl From<core_accounts::Account> for Account {
    fn from(a: core_accounts::Account) -> Self {
        let tracking_mode = match a.tracking_mode {
            core_accounts::TrackingMode::Transactions => "TRANSACTIONS",
            // ...
        }.to_string();
        Self { id: a.id, name: a.name, /* ... */ tracking_mode, /* ... */ }
    }
}
```

Add the 11 new fields. For Decimal-typed domain fields, the DTO can carry either
`Option<rust_decimal::Decimal>` (utoipa supports Decimal via the `decimal`
feature) **or** stringify them. Pattern check: `core::Account` will hold
`Option<Decimal>`; the simplest faithful DTO is also `Option<Decimal>` —
utoipa's `ToSchema` derive accepts it. If utoipa balks, fall back to
`Option<String>` and serialize Decimal in the From impl.

**`NewAccount` DTO + From-into-core pattern** (lines 56-109):

```rust
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    // ...
}

impl From<NewAccount> for core_accounts::NewAccount {
    fn from(a: NewAccount) -> Self { /* field-by-field copy */ }
}
```

Mirror exactly: 11 new fields on the DTO, 11 new field copies in the impl. Same
for `AccountUpdate` at lines 111-147.

---

### `apps/server/src/api/accounts.rs` (controller, request-response)

**Analog:** itself. **No signature change required** — DTOs flow through.

Existing handler shape (lines 38-46):

```rust
async fn create_account(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewAccount>,
) -> ApiResult<Json<Account>> {
    let core_new = payload.into();
    let created = state.account_service.create_account(core_new).await?;
    Ok(Json(Account::from(created)))
}
```

Per A2 (no separate `update_account_balance` endpoint), no router/handler
extension. Verify only — DTO changes carry the new fields automatically.

---

### `apps/frontend/src/lib/constants.ts` (utility, transform)

**Analog:** itself.

**Existing pattern (lines 44-72):**

```typescript
export const AccountType = {
  SECURITIES: "SECURITIES",
  CASH: "CASH",
  CRYPTOCURRENCY: "CRYPTOCURRENCY",
} as const;

export type AccountType = (typeof AccountType)[keyof typeof AccountType];

export const accountTypeSchema = z.enum([
  AccountType.SECURITIES,
  AccountType.CASH,
  AccountType.CRYPTOCURRENCY,
]);

export function defaultGroupForAccountType(accountType: AccountType): string {
  switch (accountType) {
    case AccountType.SECURITIES:
      return "Investments";
    case AccountType.CASH:
      return "Cash";
    case AccountType.CRYPTOCURRENCY:
      return "Crypto";
    default:
      return "Investments";
  }
}
```

Extend `AccountType` const with 4 new entries; extend `accountTypeSchema` zod
enum; extend the `switch` in `defaultGroupForAccountType` per D-16.

Append `AccountKind` + `accountKind()` helper from RESEARCH.md §Derived Helpers
— TypeScript:

```typescript
export const AccountKind = {
  ASSET: "ASSET",
  LIABILITY: "LIABILITY",
  INVESTMENT: "INVESTMENT",
} as const;
export type AccountKind = (typeof AccountKind)[keyof typeof AccountKind];

export function accountKind(accountType: AccountType): AccountKind {
  switch (accountType) {
    case AccountType.CHECKING:
    case AccountType.SAVINGS:
    case AccountType.CASH:
      return AccountKind.ASSET;
    case AccountType.CREDIT_CARD:
    case AccountType.LOAN:
      return AccountKind.LIABILITY;
    case AccountType.SECURITIES:
    case AccountType.CRYPTOCURRENCY:
      return AccountKind.INVESTMENT;
    default: {
      const _exhaustive: never = accountType;
      return AccountKind.ASSET;
    }
  }
}
```

The `never` exhaustiveness check in `default` is the key safety pattern — adding
any future `AccountType` variant will fail compilation here.

---

### `apps/frontend/src/lib/types/account.ts` (model, n/a)

**Analog:** itself.

**Existing TS interface** (lines 7-25):

```typescript
export interface Account {
  id: string;
  name: string;
  accountType: AccountType;
  group?: string;
  balance: number; // ← LEGACY FIELD — RESEARCH.md landmine 7
  currency: string;
  isDefault: boolean;
  isActive: boolean;
  isArchived: boolean;
  trackingMode: TrackingMode;
  createdAt: Date;
  updatedAt: Date;
  platformId?: string;
  accountNumber?: string;
  meta?: string;
  provider?: string;
  providerAccountId?: string;
}
```

Add 11 fields (camelCase per existing convention). Per RESEARCH.md landmine 7,
leave the legacy `balance: number` field intact — both fields coexist:

```typescript
// NEW — mirrors Rust core
institution?: string;
openingBalance?: string;        // Decimal-as-string (matches DTO when string)
currentBalance?: string;
balanceUpdatedAt?: Date;
creditLimit?: string;
statementCycleDay?: number;
statementBalance?: string;
minimumPayment?: string;
statementDueDate?: string;      // ISO date
rewardPointsBalance?: number;
cashbackBalance?: string;
```

(If the planner picks `Decimal`-as-`number` on the wire, swap the typing to
`number`. Stringly typed is the conservative match for TEXT-storage.)

---

### `apps/frontend/src/lib/schemas.ts` (utility, transform)

**Analog:** itself.

**Existing zod schema (lines 76-96):**

```typescript
export const trackingModeSchema = z.enum([
  "TRANSACTIONS",
  "HOLDINGS",
  "NOT_SET",
]);

export const newAccountSchema = z.object({
  id: z.string().uuid().optional(),
  name: z.string().min(2).max(50),
  group: z.string().optional(),
  isDefault: z.boolean().optional(),
  isActive: z.boolean().optional(),
  isArchived: z.boolean().optional().default(false),
  accountType: accountTypeSchema,
  currency: z.string({ required_error: "Please select a currency." }),
  trackingMode: trackingModeSchema.optional().default("NOT_SET"),
  meta: z.string().nullable().optional(),
});
```

Extend with the 11 new fields. Use `.superRefine()` to mirror the Rust
service-layer validation (CC-required-when-CREDIT_CARD, null-when-not-CC):

```typescript
export const newAccountSchema = z
  .object({
    // ... existing
    institution: z.string().optional(),
    openingBalance: z.string().optional(),
    currentBalance: z.string().optional(),
    // ... 8 more
  })
  .superRefine((data, ctx) => {
    const isCC = data.accountType === "CREDIT_CARD";
    if (!isCC) {
      // CC-fields-must-be-null
      if (data.creditLimit)
        ctx.addIssue({
          code: "custom",
          path: ["creditLimit"],
          message: "Credit card fields are only valid for CREDIT_CARD accounts",
        });
      // ... other CC fields
    } else {
      // CREDIT_CARD-fields-required
      if (!data.creditLimit)
        ctx.addIssue({
          code: "custom",
          path: ["creditLimit"],
          message: "Credit limit must be greater than 0",
        });
      // ... statementCycleDay 1..31
    }
  });
```

---

### `apps/frontend/src/hooks/use-accounts.ts` (hook, request-response)

**Analog:** itself. **No code change required.**

Existing default already satisfies D-19 (lines 7-8):

```typescript
export function useAccounts(options?: { filterActive?: boolean; includeArchived?: boolean }) {
  const { filterActive = true, includeArchived = false } = options ?? {};
```

Per RESEARCH.md §Archive Filter Audit, every existing call site is correct. The
new `/settings/accounts` page (D-15 amendment) MUST pass
`includeArchived: showArchivedToggle` so the toggle reveals archived rows on
demand. The settings page already passes `includeArchived: true` and applies a
local filter (`accounts-page.tsx:25`); plug the new toggle into the local filter
so the default state hides archived rows from the new unified list.

---

### `apps/frontend/src/adapters/shared/accounts.ts` (adapter, request-response)

**Analog:** itself. **No code change required for the adapter** (cascades from
`newAccountSchema`).

Existing pattern (lines 22-58):

```typescript
export const getAccounts = async (
  includeArchived?: boolean,
): Promise<Account[]> => {
  try {
    const accounts = await invoke<SerializedAccount[]>("get_accounts", {
      includeArchived: includeArchived ?? false,
    });
    return accounts.map(normalizeAccountDates);
  } catch (error) {
    /* ... */
  }
};

export const createAccount = async (account: NewAccount): Promise<Account> => {
  try {
    const created = await invoke<SerializedAccount>("create_account", {
      account,
    });
    return normalizeAccountDates(created);
  } catch (error) {
    /* ... */
  }
};

export const updateAccount = async (account: NewAccount): Promise<Account> => {
  try {
    const payload = isDesktop
      ? (() => {
          const { currency: _, ...rest } = account;
          return rest;
        })()
      : account;
    const updated = await invoke<SerializedAccount>("update_account", {
      accountUpdate: payload,
    });
    return normalizeAccountDates(updated);
  } catch (error) {
    /* ... */
  }
};
```

`NewAccount` is `z.infer<typeof newAccountSchema>` — schema extension auto-
extends this. **However:** `normalizeAccountDates` only normalizes
`createdAt`/`updatedAt`. Per landmine 5 / new field `balanceUpdatedAt` (which is
`Date`), extend `normalizeAccountDates` to also coerce `balanceUpdatedAt` to a
`Date` instance.

---

### `apps/frontend/src/adapters/web/core.ts` (config, request-response)

**Analog:** itself. **No COMMANDS change required (per A2)**. Existing account
commands at lines 38-42:

```typescript
get_accounts: { method: "GET", path: "/accounts" },
create_account: { method: "POST", path: "/accounts" },
update_account: { method: "PUT", path: "/accounts" },
delete_account: { method: "DELETE", path: "/accounts" },
```

Balance updates use the existing `update_account` mutation per A2.

---

### `apps/frontend/src/components/account-selector.tsx` (component, event-driven)

**Analog:** itself.

**Icon map pattern (lines 26-31):**

```typescript
const accountTypeIcons: Record<string, Icon> = {
  SECURITIES: Icons.Briefcase,
  CASH: Icons.DollarSign,
  CRYPTOCURRENCY: Icons.Bitcoin,
  [PORTFOLIO_ACCOUNT_ID]: Icons.Wallet,
};
```

Note: this is `Record<string, Icon>` — **loose**, so unknown types fall back at
runtime. Extend with the 4 new entries per UI-SPEC §Component Inventory:

```typescript
CHECKING: Icons.Wallet,
SAVINGS: Icons.Coins,
CREDIT_CARD: Icons.CreditCard,
LOAN: Icons.Building2,  // Building2 — fallback for the Landmark icon UI-SPEC mentions
```

`useAccounts` invocation (lines 127-130) already passes
`includeArchived: false`. **Verify only** — D-19 satisfied.

---

### `apps/frontend/src/components/account-selector-mobile.tsx` (component, event-driven)

**Analog:** itself.

**Icon map (lines 24-29) — same as desktop selector**: extend with 4 new
entries.

**Label switch (lines 110-123):**

```typescript
const getAccountTypeLabel = (type: string): string => {
  switch (type) {
    case PORTFOLIO_ACCOUNT_ID:
      return "Portfolio";
    case "SECURITIES":
      return "Securities Accounts";
    case "CASH":
      return "Cash Accounts";
    case "CRYPTOCURRENCY":
      return "Cryptocurrency Accounts";
    default:
      return "Other Accounts";
  }
};
```

Extend with 4 new cases (labels per UI-SPEC §Copywriting Contract):

```typescript
case "CHECKING": return "Checking Accounts";
case "SAVINGS": return "Savings Accounts";
case "CREDIT_CARD": return "Credit Cards";
case "LOAN": return "Loans";
```

---

### `apps/frontend/src/components/app-launcher.tsx` (component, event-driven) — **EXHAUSTIVE Record**

**Analog:** itself.

**Compile-blocking pattern (lines 65-70):**

```typescript
const accountTypeIcons: Record<
  AccountType | typeof PORTFOLIO_ACCOUNT_ID,
  Icon
> = {
  [AccountType.SECURITIES]: Icons.Briefcase,
  [AccountType.CASH]: Icons.DollarSign,
  [AccountType.CRYPTOCURRENCY]: Icons.Bitcoin,
  [PORTFOLIO_ACCOUNT_ID]: Icons.Wallet,
};
```

`Record<AccountType, Icon>` is exhaustive — adding 4 new `AccountType` variants
WILL break TS compile. Add 4 new entries:

```typescript
[AccountType.CHECKING]: Icons.Wallet,
[AccountType.SAVINGS]: Icons.Coins,
[AccountType.CREDIT_CARD]: Icons.CreditCard,
[AccountType.LOAN]: Icons.Building2,
```

(Same icon mapping as account-selector.tsx — keep consistent.)

---

### `apps/frontend/src/pages/account/account-page.tsx` (page) — **EXHAUSTIVE Record + CC sections**

**Analog:** itself.

**Compile-blocking pattern (lines 82-86):**

```typescript
const accountTypeIcons: Record<AccountType, Icon> = {
  SECURITIES: Icons.Briefcase,
  CASH: Icons.DollarSign,
  CRYPTOCURRENCY: Icons.Bitcoin,
};
```

Same fix as above: add 4 entries. Plus add CC-specific sections per UI-SPEC §3
(Credit overview / Statement snapshot / Rewards). Render-gate on
`accountKind(account.accountType) === "LIABILITY"` (or strict
`account.accountType === AccountType.CREDIT_CARD` for CC-specific UI).

---

### `apps/frontend/src/pages/dashboard/accounts-summary.tsx` (page)

**Analog:** itself.

**Group-by-account.group pattern (lines 311-361):**

```typescript
const combinedAccountViews = useMemo((): AccountSummaryDisplayData[] => {
  return accounts.map(
    (acc, i): AccountSummaryDisplayData => ({
      accountName: acc.name,
      /* ... */
      accountType: acc.accountType, // pass-through, no switch
      accountGroup: acc.group ?? null,
      isGroup: false,
    }),
  );
}, [accounts, latestValuations, performanceQueries, settings?.baseCurrency]);
```

`accountType` is data, never matched. **No code change required** — RESEARCH.md
confirms safe. New types flow through. Reuse the `AccountSummaryComponent`
(lines 56-257) verbatim from the new `/settings/accounts` host extension.

---

### `apps/frontend/src/pages/settings/accounts/accounts-page.tsx` (page, HOST)

**Analog:** itself. This is the unified-list **host route** per D-15 amendment.

**Existing structure to extend (lines 24-296):**

Filter state pattern (lines 38-41):

```typescript
const [visibleModal, setVisibleModal] = useState(false);
const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
const [searchQuery, setSearchQuery] = useState("");
const [filter, setFilter] = useState<FilterType>("all");
```

Group/section pattern (lines 107-119):

```typescript
const { activeAccounts, inactiveAccounts } = useMemo(() => {
  const active = filteredAccounts.filter((a) => a.isActive && !a.isArchived);
  const inactive = filteredAccounts
    .filter((a) => !a.isActive || a.isArchived)
    .sort(/* ... */);
  return { activeAccounts: active, inactiveAccounts: inactive };
}, [filteredAccounts]);
```

Modal trigger pattern (lines 290-294):

```tsx
<AccountEditModal
  account={selectedAccount || undefined}
  open={visibleModal}
  onClose={() => setVisibleModal(false)}
/>
```

**Phase 3 additions (per UI-SPEC §1):**

1. Add a "group-by" axis using
   `account.group ?? defaultGroupForAccountType(account.accountType)` so
   accounts cluster into Banking / Credit Cards / Loans / Investments / Cash /
   Crypto. Reuse the grouping reduce shape from `accounts-summary.tsx:311-361`.
2. Render CC rows with the "Available credit" chip
   (`creditLimit - currentBalance`) — see `account-item.tsx` extension below for
   the inline chip JSX. Use `Icons.CreditCard` to mark the row.
3. Render group totals in base currency (FX conversion: see
   `accounts-summary.tsx:337-339`).
4. The "Show archived" switch (UI-SPEC §1 filter bar) toggles between
   `includeArchived: false` (default) and `true`. Per D-19, default off. Replace
   the existing `filter === "archived"` toggle group with a `Switch` component
   as UI-SPEC requires, OR add the Switch alongside the existing ToggleGroup
   (planner's call). Either way, the filter state stays local.

---

### `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx` (component)

**Analog:** itself.

**Hard-coded type cast at line 20** — must extend:

```typescript
accountType: (account?.accountType ?? "SECURITIES") as "SECURITIES" | "CASH" | "CRYPTOCURRENCY",
```

Replace with the proper `AccountType` import from `@/lib/constants`:

```typescript
import { AccountType, type AccountType as AccountTypeT } from "@/lib/constants";
// ...
accountType: (account?.accountType ?? AccountType.SECURITIES) as AccountTypeT,
```

`AccountForm` will already accept the broader union via the schema cascade.

Default values shape to extend with new fields (lines 16-28):

```typescript
const defaultValues = {
  id: account?.id ?? undefined,
  name: account?.name ?? "",
  balance: account?.balance ?? 0,                    // legacy
  accountType: ...,
  group: account?.group ?? undefined,
  currency: account?.currency ?? settings?.baseCurrency ?? "USD",
  isDefault: account?.isDefault ?? false,
  isActive: account?.id ? account?.isActive : true,
  isArchived: account?.isArchived ?? false,
  trackingMode: account?.trackingMode,
  meta: account?.meta,
};
```

Extend with the 11 new field defaults:

```typescript
institution: account?.institution ?? "",
openingBalance: account?.openingBalance ?? "",
currentBalance: account?.currentBalance ?? "",
balanceUpdatedAt: account?.balanceUpdatedAt,
creditLimit: account?.creditLimit ?? "",
statementCycleDay: account?.statementCycleDay,
statementBalance: account?.statementBalance ?? "",
minimumPayment: account?.minimumPayment ?? "",
statementDueDate: account?.statementDueDate,
rewardPointsBalance: account?.rewardPointsBalance,
cashbackBalance: account?.cashbackBalance ?? "",
```

---

### `apps/frontend/src/pages/settings/accounts/components/account-form.tsx` (component, dynamic per type)

**Analog:** itself.

**Account type list (lines 47-51) — extend:**

```typescript
const accountTypes: ResponsiveSelectOption[] = [
  { label: "Securities", value: "SECURITIES" },
  { label: "Cash", value: "CASH" },
  { label: "Crypto", value: "CRYPTOCURRENCY" },
];
```

Extend with 4 new options. Per UI-SPEC §2 the new-account flow uses a
`ToggleGroup` (icon + label cards) rather than `ResponsiveSelect`. The existing
form uses `ResponsiveSelect`; rather than rewrite, the planner keeps
`ResponsiveSelect` and lays the dynamic CC-fields section underneath it (see
UI-SPEC §2 form sections 4 & 5).

**Form-field pattern to mirror for new CC fields (lines 184-204):**

```tsx
<FormField
  control={form.control}
  name="accountType"
  render={({ field }) => (
    <FormItem className="flex flex-col">
      <FormLabel>Account Type</FormLabel>
      <FormControl>
        <ResponsiveSelect
          value={field.value}
          onValueChange={field.onChange}
          options={accountTypes}
          placeholder="Select an account type"
          /* ... */
        />
      </FormControl>
      <FormMessage />
    </FormItem>
  )}
/>
```

Conditional CC-fields section pattern:
`form.watch("accountType") === "CREDIT_CARD"` gates rendering. Use `MoneyInput`
for limit/balance/payment/cashback, `Select` for `statementCycleDay` (1..31),
`DatePickerInput` for `statementDueDate`. All primitives are already in
`@whaleit/ui` per RESEARCH.md.

**Submit-payload pattern (lines 87-103) carries through unchanged** —
schema-cascade applies.

---

### `apps/frontend/src/pages/settings/accounts/components/account-item.tsx` (component)

**Analog:** itself.

**Icon map (lines 10-26) — extend:**

```typescript
const accountTypeConfig: Record<
  AccountType,
  { icon: Icon; bgClass: string; iconClass: string }
> = {
  SECURITIES: {
    icon: Icons.Briefcase,
    bgClass: "bg-blue-500/10",
    iconClass: "text-blue-500",
  },
  CASH: {
    icon: Icons.DollarSign,
    bgClass: "bg-green-500/10",
    iconClass: "text-green-500",
  },
  CRYPTOCURRENCY: {
    icon: Icons.Bitcoin,
    bgClass: "bg-orange-500/10",
    iconClass: "text-orange-500",
  },
};
```

This is `Record<AccountType, ...>` — exhaustive; **add 4 entries**:

```typescript
CHECKING: { icon: Icons.Wallet, bgClass: "bg-sky-500/10", iconClass: "text-sky-500" },
SAVINGS: { icon: Icons.Coins, bgClass: "bg-emerald-500/10", iconClass: "text-emerald-500" },
CREDIT_CARD: { icon: Icons.CreditCard, bgClass: "bg-fuchsia-500/10", iconClass: "text-fuchsia-500" },
LOAN: { icon: Icons.Building2, bgClass: "bg-amber-500/10", iconClass: "text-amber-500" },
```

(Color tokens MUST stay within UI-SPEC palette — the planner may need to swap to
semantic tokens. The above is a starting point matching existing `bg-*-500/10`
conventions.)

**Inline chip pattern for archived (lines 127-138):**

```tsx
{
  account.isArchived && (
    <span className="inline-flex items-center gap-1 rounded-md border border-red-200/40 bg-red-100/30 px-2 py-1 text-xs font-medium text-red-600 dark:border-red-500/20 dark:bg-red-500/10 dark:text-red-400">
      <Icons.FileArchive className="h-3 w-3" />
      Archived
    </span>
  );
}
```

Extend with an "Available credit" chip for CC rows (UI-SPEC §1) — mirror the
same shape but with `Icons.CreditCard` and the computed
`creditLimit - currentBalance` value:

```tsx
{
  account.accountType === "CREDIT_CARD" && account.creditLimit && (
    <span className="inline-flex items-center gap-1 rounded-md border border-emerald-200/40 bg-emerald-100/30 px-2 py-1 text-xs font-medium text-emerald-600">
      <Icons.CreditCard className="h-3 w-3" />
      Available {formatBaseCurrency(creditLimit - currentBalance)}
    </span>
  );
}
```

(Use UI-SPEC color discipline — `bg-success/10 text-success` per the Color
section, not `emerald-*`. Above is structural illustration.)

---

### `apps/frontend/src/lib/constants.test.ts` (test, NEW)

**Analog:** none in `lib/`. Pattern from any vitest-style spec; planner can
hand-roll. Coverage focus per VALIDATION.md Wave 0:

- `accountKind(t)` returns expected `AccountKind` for each `AccountType`
- `defaultGroupForAccountType(t)` returns expected group per D-16

```typescript
import { describe, it, expect } from "vitest";
import {
  AccountType,
  AccountKind,
  accountKind,
  defaultGroupForAccountType,
} from "./constants";

describe("accountKind", () => {
  it("classifies CHECKING, SAVINGS, CASH as ASSET", () => {
    /* ... */
  });
  it("classifies CREDIT_CARD, LOAN as LIABILITY", () => {
    /* ... */
  });
  it("classifies SECURITIES, CRYPTOCURRENCY as INVESTMENT", () => {
    /* ... */
  });
});

describe("defaultGroupForAccountType", () => {
  it("returns 'Banking' for CHECKING and SAVINGS", () => {
    /* ... */
  });
  // etc.
});
```

### `apps/frontend/src/lib/schemas.test.ts` (test, NEW)

**Analog:** none. Coverage focus:

- `newAccountSchema.safeParse(...)` accepts a valid CC payload
- Rejects CC payload missing `creditLimit`
- Rejects CHECKING payload that includes `creditLimit`
- Rejects CC payload with `statementCycleDay = 32`

### `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` (test, NEW)

**Analog:** RESEARCH.md mentions extending `accounts-summary.test.tsx`. Use
React Testing Library (already in stack per RESEARCH.md). Coverage focus:

- Renders rows for all account types
- Hides archived rows by default; reveals when "Show archived" Switch is on
- CC rows display "Available credit" chip
- Click "+ New" opens the AccountEditModal

### `e2e/11-accounts.spec.ts` (test, NEW)

**Analog:** `e2e/05-form-validation.spec.ts` (lean structure with login +
form-flows) + `e2e/01-happy-path.spec.ts` (multi-step account-create flow).

**Lean structure pattern from `e2e/05-form-validation.spec.ts:1-23`:**

```typescript
import { expect, Page, test } from "@playwright/test";
import { BASE_URL, loginIfNeeded } from "./helpers";

test.describe.configure({ mode: "serial" });

test.describe("Bank Accounts & Credit Cards", () => {
  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test("1. Setup: login", async () => {
    test.setTimeout(180000);
    await loginIfNeeded(page);
    await page.goto(`${BASE_URL}/dashboard`, { waitUntil: "domcontentloaded" });
  });

  test("2. Create CHECKING account", async () => {
    /* ... */
  });
  test("3. Create CREDIT_CARD with required CC fields", async () => {
    /* ... */
  });
  test("4. Update credit-card balance via 'Update balance' modal", async () => {
    /* ... */
  });
  test("5. Archive account, verify hidden from selectors and unified list", async () => {
    /* ... */
  });
  test("6. 'Show archived' toggle reveals archived row", async () => {
    /* ... */
  });
});
```

**Existing helper to reuse (`e2e/helpers.ts:112-119`):**

```typescript
export async function createAccount(
  page: Page,
  name: string,
  currency: string,
  trackingMode: "Transactions" | "Holdings" = "Transactions",
) {
  await page.goto(`${BASE_URL}/settings/accounts`, {
    waitUntil: "domcontentloaded",
  });
  await expect(
    page.getByRole("heading", { name: "Accounts", exact: true }),
  ).toBeVisible();
  // ...
}
```

The new spec MUST navigate to `/settings/accounts` (not `/accounts`) per D-15
amendment. Reuse `loginIfNeeded`. Reuse `createAccount` helper for CHECKING; the
CC flow needs new helper steps for the dynamic CC fields.

---

## Shared Patterns

### Decimal serialization (TEXT ↔ rust_decimal::Decimal)

**Source:** `crates/storage-postgres/src/fx/model.rs:46-111` (canonical).

**Apply to:** every new TEXT-typed money column in `accounts/model.rs` From
impls.

```rust
// Read (DB string → Decimal)
opening_balance: db
    .opening_balance
    .as_deref()
    .and_then(|s| Decimal::from_str(s).ok()),

// Write (Decimal → DB string)
opening_balance: domain.opening_balance.as_ref().map(|d| d.to_string()),
```

Both `rust_decimal` and `rust_decimal_macros` are already declared in
`crates/core/Cargo.toml:22-23`; `rust_decimal` is in
`crates/storage-postgres/Cargo.toml:27`. No new dependency needed.

### Validation at service-layer (CC-gated rules)

**Source:** `crates/core/src/accounts/accounts_model.rs:72-87`
(`NewAccount::validate`).

**Apply to:** `accounts_model.rs` `NewAccount::validate` and
`AccountUpdate::validate`. The full extension shape is RESEARCH.md §Validation
Layer Phase 3 Extension (lines 627-696). The pattern uses
`Error::Validation(ValidationError::InvalidInput(...))` returns and is called
from `repository.rs:30` and `repository.rs:46`.

### Embedded migration runner

**Source:** `crates/storage-postgres/src/db/mod.rs:57-77` (calls
`MigrationHarness.run_pending_migrations(MIGRATIONS)`).

**Apply to:** Phase 3 migration runs automatically on `apps/server` startup — no
separate command. After authoring `up.sql`, regenerate `schema.rs` (RESEARCH.md
landmine 4: hand-synchronized).

### `serde(rename_all = "camelCase")` boundary

**Source:** every domain DTO and API DTO already uses
`#[serde(rename_all = "camelCase")]` (e.g. `apps/server/src/models.rs:7,57,112`,
`crates/core/src/accounts/accounts_model.rs:23,51,91`).

**Apply to:** new fields use `snake_case` in Rust (`opening_balance`), camelCase
on the wire (`openingBalance`), camelCase in TS — matches the existing
convention without manual `#[serde(rename = ...)]`.

### Exhaustive `Record<AccountType, T>` (TS compile gate)

**Source:** `apps/frontend/src/components/app-launcher.tsx:65-70` and
`apps/frontend/src/pages/account/account-page.tsx:82-86`.

**Apply to:** these two files (compile-blocking) plus `account-item.tsx:10-26`
(also typed `Record<AccountType, ...>` — same fix). The loose
`Record<string, Icon>` in `account-selector.tsx` and
`account-selector-mobile.tsx` is NOT compile-blocking but must still be extended
to keep runtime icons correct.

### Archive-default pattern

**Source:** `apps/frontend/src/hooks/use-accounts.ts:7-8`.

**Apply to:** the new "Show archived" toggle on `/settings/accounts` HOST.
Default state is `false`; when toggled, pass through to
`useAccounts({ includeArchived: showArchived })` (or, since the settings page
already loads all accounts and filters locally, plug into the local filter as
described in D-19 / RESEARCH.md §Archive Filter Audit).

### `accounts-summary.tsx` row shape

**Source:** `apps/frontend/src/pages/dashboard/accounts-summary.tsx:56-257`
(`AccountSummaryComponent`).

**Apply to:** the unified-list rows on the new `/settings/accounts` host.
UI-SPEC §1 specifies "reuse `accounts-summary.tsx` row component shape exactly."
Extract and reuse, or copy the JSX into a new shared component.

### Grouped section + count badge

**Source:**
`apps/frontend/src/pages/settings/accounts/accounts-page.tsx:252-282`.

**Apply to:** the new group-by-account.group rendering on the host page. Reuse
the `<h3 className="text-muted-foreground text-sm font-medium">` +
`<span className="bg-success/20 text-success rounded-full ...">` count chip
pattern verbatim.

---

## No Analog Found

| File                                                               | Role                 | Data Flow        | Reason                                                                                                                   | Planner Direction                                                                                                                                                                     |
| ------------------------------------------------------------------ | -------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/core/src/accounts/accounts_service_tests.rs`               | service unit (async) | request-response | No existing test in `accounts/` for service-layer; closest is `accounts_model_tests.rs` (sync, no mocks)                 | Use `accounts_model_tests.rs:1-9` test-module shape; mock `AccountRepositoryTrait` and the other 5 dependencies of `AccountService::new` (constructor at `accounts_service.rs:24-43`) |
| `crates/storage-postgres/src/accounts/repository_tests.rs`         | integration (PG)     | CRUD             | No `accounts/` integration tests exist; closest is `fx/model.rs` Decimal-roundtrip pattern                               | Use `db/mod.rs:57-77` `run_migrations` to set up test DB; assert via `PgAccountRepository::new(pool)` per `repository.rs:21-25`                                                       |
| `crates/storage-postgres/src/accounts/migration_tests.rs`          | migration smoke test | DDL              | No DDL test analog                                                                                                       | One `#[tokio::test]` running `run_migrations` against fresh DB and a `SELECT` over new columns                                                                                        |
| `apps/frontend/src/lib/constants.test.ts`                          | TS unit (vitest)     | n/a              | No file-level analog in `lib/`                                                                                           | Hand-roll vitest spec; pattern from RESEARCH.md §Validation Architecture                                                                                                              |
| `apps/frontend/src/lib/schemas.test.ts`                            | TS unit (vitest)     | n/a              | No analog                                                                                                                | Same                                                                                                                                                                                  |
| `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` | TS component (RTL)   | request-response | RESEARCH.md mentions `accounts-summary.test.tsx` exists — extend the same RTL pattern; cousin not in this directory tree | Use `accounts-summary.test.tsx` (per RESEARCH.md §Phase Requirements → Test Map) as the shape reference                                                                               |
| `e2e/11-accounts.spec.ts`                                          | E2E                  | request-response | None for accounts specifically; closest is `05-form-validation.spec.ts` for lean structure                               | Use the structure shown above; reuse helpers from `e2e/helpers.ts`                                                                                                                    |

---

## Metadata

**Analog search scope:**

- `crates/core/src/accounts/`
- `crates/core/src/portfolio/net_worth/` (categorize_by_account_type landmine)
- `crates/storage-postgres/src/accounts/`
- `crates/storage-postgres/src/fx/` (Decimal-serialization analog)
- `crates/storage-postgres/migrations/` (both existing migration directories)
- `crates/storage-postgres/src/db/mod.rs` (migration runner)
- `apps/server/src/api/accounts.rs`, `apps/server/src/models.rs`
- `apps/frontend/src/lib/constants.ts`,
  `apps/frontend/src/lib/types/account.ts`, `apps/frontend/src/lib/schemas.ts`
- `apps/frontend/src/hooks/use-accounts.ts`
- `apps/frontend/src/adapters/shared/accounts.ts`,
  `apps/frontend/src/adapters/web/core.ts`
- `apps/frontend/src/components/account-selector.tsx`,
  `account-selector-mobile.tsx`, `app-launcher.tsx`
- `apps/frontend/src/pages/dashboard/accounts-summary.tsx`
- `apps/frontend/src/pages/account/account-page.tsx`
- `apps/frontend/src/pages/settings/accounts/accounts-page.tsx` and
  `components/{account-edit-modal,account-form,account-item}.tsx`
- `e2e/01-happy-path.spec.ts`, `e2e/05-form-validation.spec.ts`,
  `e2e/helpers.ts`

**Files scanned:** 31

**Pattern extraction date:** 2026-04-25

---

## PATTERN MAPPING COMPLETE

**Phase:** 3 - Bank Accounts & Credit Cards **Files classified:** 34 (27
extend + 7 new) **Analogs found:** 32 / 34

### Coverage

- Files with exact analog: 25
- Files with role-match / cousin analog: 7
- Files with no analog (new ground; cousins documented): 2 (frontend test files
  with no in-tree vitest cousin)

### Key Patterns Identified

- **Money columns are TEXT-serialized `rust_decimal::Decimal`** (per A1 +
  `fx/model.rs:46-111` canonical pattern) — Phase 3's 7 money columns follow
  this exactly; no NUMERIC dialect introduced.
- **Migration directories use timestamp-prefixed naming**
  (`YYYYMMDDHHMMSS_descriptive_name/{up,down}.sql`) per the two existing
  migrations; Phase 3 ALTER TABLE pattern is additive (mirrors the auth_users
  additive structure but on an existing table).
- **Trait surface is invariant.** `AccountRepositoryTrait` /
  `AccountServiceTrait` signatures unchanged — new fields ride through
  `NewAccount` / `AccountUpdate` / `Account` as struct data. Updates flow
  through the existing `update_account` mutation per A2 (no new endpoint).
- **Validation lives in `NewAccount::validate()`** (called once at
  `repository.rs:30` and `repository.rs:46`); CC-gated rules extend the same
  function. Frontend mirrors via `newAccountSchema.superRefine`.
- **Two TS files have exhaustive `Record<AccountType, Icon>`**
  (`app-launcher.tsx:65`, `account-page.tsx:82`) plus `account-item.tsx:10` —
  adding 4 new variants WILL break compile until all three are extended. Loose
  `Record<string, Icon>` selectors won't break TS but must be extended to keep
  runtime icons correct.
- **`/settings/accounts` page is the host** (D-15 amendment); the existing page
  already manages search + filter + group + modal — Phase 3 extends in place, no
  new route.
- **Migrations run embedded** (`db/mod.rs:15` `embed_migrations!()`) at server
  boot — but `schema.rs` is hand-synchronized (RESEARCH.md landmine 4); planner
  must include the regen step explicitly.
- **`accounts-summary.tsx` is account-type-data-only** (no switch on type), so
  it works with new types without code change. Confirmed safe.

### File Created

`/Users/muhamad.rohman/Workspace/github.com/muhx/whaleit/.planning/phases/03-bank-accounts-credit-cards/03-PATTERNS.md`

### Ready for Planning

Pattern mapping complete. Planner can now reference analog patterns in PLAN.md
files with concrete file:line excerpts for every domain group.
