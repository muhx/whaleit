---
phase: 03-bank-accounts-credit-cards
review_depth: standard
status: issues_found
files_reviewed: 41
critical_count: 0
high_count: 3
medium_count: 7
low_count: 8
created: 2026-04-25
---

# Phase 3 Code Review

## Summary

Phase 3 lands the bank-accounts and credit-card domain extension across the full
stack: PG migration with 11 nullable columns (NUMERIC(20,8) for money), a clean
Rust core extension (4 new AccountType leaves + AccountKind helper), full
DTO/service/repo/zod/component layers, and an end-to-end Playwright spec. The
data model, validation contract, and decimal handling are coherent and faithful
to the locked decisions (D-06, D-08, D-10, D-11, D-12, D-13, D-19). Test
coverage is broad for happy paths.

The implementation is generally clean, but two **High** issues will bite users
in normal flows: (1) `AccountEditModal` does not propagate the new Phase 3
fields into the form's `defaultValues`, so editing any existing bank/CC/loan
account will fail validation (the modal-driven form requires `openingBalance`,
but the modal does not seed it from the account); and (2)
`AccountUpdate::validate()` permits a user to switch a CREDIT_CARD account to
CHECKING/SAVINGS/LOAN while leaving the CC columns intact in the DB, because the
update validator never enforces "CC fields must be cleared when type leaves
CREDIT_CARD" and Diesel's `AsChangeset` treats `None` Option fields as "skip
column" — so the user has no in-band way to NULL them. A third High is that the
HTTP `AccountUpdate` DTO accepts a client-supplied `balanceUpdatedAt` that the
service only overrides when `current_balance` changes, breaking the D-12
invariant ("server is the source of truth for when the balance was last
touched").

The remaining findings are Medium/Low: a few exhaustiveness gaps in legacy
selector code that hard-codes per-type icons, non-semantic colors on the
liability "Available" chip, missing index on `balance_updated_at`, missing
`treat_none_as_null` clarity, and minor test/code-quality nits. No Critical
issues — no SQL injection, no XSS, no data-corruption-on-create, and the
auto-stamp logic itself is correct.

## Findings

### High

#### H-01 — `AccountEditModal` drops every new Phase 3 field on edit, breaking validation

- **File:**
  `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx:14-29`
- **Severity:** High
- **Category:** Bug
- **Description:** The modal's `defaultValues` only seeds `id`, `name`,
  `currentBalance`, `accountType`, `group`, `currency`, `isDefault`, `isActive`,
  `isArchived`, `trackingMode`, `meta`. It does NOT seed the Phase 3 fields:
  `institution`, `openingBalance`, `creditLimit`, `statementCycleDay`,
  `statementBalance`, `minimumPayment`, `statementDueDate`,
  `rewardPointsBalance`, `cashbackBalance`. Consequence: when a user opens the
  edit dialog on an existing CHECKING/SAVINGS/LOAN account, the form initializes
  with `openingBalance: undefined`, the form's `superRefine` then refuses submit
  ("Opening balance is required for this account type."), and the user must
  re-enter all CC/bank fields manually on every edit. CC accounts are even more
  broken — opening a Credit Card edit dialog will refuse to submit until the
  user re-enters `creditLimit` and `statementCycleDay`. This blocks the basic
  edit flow declared by ACCT-04.
- **Suggested fix:** Forward all nine Phase 3 fields into `defaultValues` so the
  form mirrors the current DB state:
  ```ts
  const defaultValues = {
    id: account?.id ?? undefined,
    name: account?.name ?? "",
    currentBalance: account?.currentBalance,
    accountType: (account?.accountType ?? "SECURITIES") as AccountType,
    group: account?.group ?? undefined,
    currency: account?.currency ?? settings?.baseCurrency ?? "USD",
    isDefault: account?.isDefault ?? false,
    isActive: account?.id ? account?.isActive : true,
    isArchived: account?.isArchived ?? false,
    trackingMode: account?.trackingMode,
    meta: account?.meta,
    // Phase 3 (D-06, D-11, D-18):
    institution: account?.institution,
    openingBalance: account?.openingBalance,
    creditLimit: account?.creditLimit,
    statementCycleDay: account?.statementCycleDay,
    statementBalance: account?.statementBalance,
    minimumPayment: account?.minimumPayment,
    statementDueDate: account?.statementDueDate,
    rewardPointsBalance: account?.rewardPointsBalance,
    cashbackBalance: account?.cashbackBalance,
  };
  ```
  Add an `accounts-page.test.tsx` case that opens the edit dialog on a CC
  account and asserts `creditLimit` is pre-filled.

#### H-02 — `AccountUpdate::validate()` does not clear CC fields on type change; Diesel `None`-skip leaves stale CC data

- **File:** `crates/core/src/accounts/accounts_model.rs:236-269` (validator) +
  `crates/storage-postgres/src/accounts/repository.rs:45-84` (update path)
- **Severity:** High
- **Category:** Bug (D-06 invariant violation)
- **Description:** `NewAccount::validate()` enforces the D-06 rule "CC fields
  are only valid for CREDIT_CARD accounts" on **create** but
  `AccountUpdate::validate()` only checks the same rule when the update payload
  **carries** CC fields (`is_some()`). Combined with Diesel's default
  `AsChangeset` semantics for `Option<T>` (None means "skip column, do not
  write"), a user who changes an existing CREDIT_CARD account's `account_type`
  to CHECKING and submits the form WITHOUT the CC fields passes validation — but
  the DB row keeps the old `credit_limit`, `statement_cycle_day`,
  `statement_balance`, `minimum_payment`, `statement_due_date`,
  `reward_points_balance`, `cashback_balance` values intact. The resulting state
  is a CHECKING row with non-NULL CC columns, violating the D-06 invariant the
  migration relies on. Downstream consumers reading the row will see stale CC
  data on a non-CC account.

  A second consequence: there is no in-band way for a CC user to clear a
  statement balance (set it back to NULL) — `None` in the update means "skip",
  and there's no `Some(None)` sentinel because the field is `Option<Decimal>`
  not `Option<Option<Decimal>>`.

- **Suggested fix:** Two-part:
  1. In `AccountUpdate::validate()`, when `account_type != CREDIT_CARD`, reject
     the update unless the caller explicitly provides each CC field as `None`
     AND the repository writes those NULLs. Practically: extend
     `From<AccountUpdate> for AccountDB` so that when the new `account_type` is
     non-CC, all CC columns are explicitly set to `None` AND the repo's update
     path uses `treat_none_as_null` for those columns (or issues a follow-up
     `UPDATE accounts SET credit_limit = NULL, ... WHERE id = ?` when the type
     transitions out of CREDIT_CARD).
  2. Document `treat_none_as_null` posture for the new fields in
     `accounts/model.rs`. Today it's implicit — Diesel default. Without a
     comment, the next maintainer cannot reason about whether `update_account`
     with `current_balance: None` clears the column or leaves it. (Today: leaves
     it. Same for `cashback_balance`.)
  3. Add a Rust integration test `test_update_clears_cc_fields_on_type_change`
     that creates a CC, updates `account_type` to CHECKING, then asserts all 7
     CC columns are NULL. Expected to fail with the current implementation.

#### H-03 — Server DTO accepts client-controlled `balanceUpdatedAt`; auto-stamp doesn't always overwrite

- **File:** `apps/server/src/models.rs:189` +
  `crates/core/src/accounts/accounts_service.rs:80-98`
- **Severity:** High
- **Category:** Bug (D-12 invariant violation)
- **Description:** The HTTP `AccountUpdate` DTO exposes
  `balance_updated_at: Option<NaiveDateTime>` as a client-writable field (line
  189 in `apps/server/src/models.rs`). The service-layer auto-stamp
  (`accounts_service.rs:93-97`) only **conditionally** overrides it:
  ```rust
  if account_update.current_balance.is_some()
      && account_update.current_balance != existing.current_balance
  {
      account_update.balance_updated_at = Some(chrono::Utc::now().naive_utc());
  }
  ```
  So if a client sends `balanceUpdatedAt: <some past date>` along with
  `currentBalance: undefined` (or unchanged), the service does NOT overwrite —
  it passes the client value straight through to the repository. This breaks the
  D-12 contract that the server is the source of truth for "when was the balance
  last touched." A malicious or buggy client could backdate or future-date the
  field to manipulate downstream consumers (e.g., the "Last updated: X days ago"
  copy on the account detail page; future audit/sort logic).
- **Suggested fix:** Drop `balance_updated_at` from the inbound `AccountUpdate`
  DTO entirely (it's an output-only field). Either:
  - Remove `pub balance_updated_at: Option<NaiveDateTime>,` from
    `apps/server/src/models.rs:189` and the equivalent in `NewAccount` (line 110
    — accepted on create but should be server-set), OR
  - In the `From<AccountUpdate> for core_accounts::AccountUpdate` impl, always
    set `balance_updated_at: None` to discard the client value. Then the
    service-layer stamp is the sole writer.

  Add a service-level test asserting that
  `update_account(AccountUpdate { balance_updated_at: Some(<past>), current_balance: None, .. })`
  does NOT propagate the client's timestamp.

### Medium

#### M-01 — Missing index on `balance_updated_at`; future "stale balances" UI will scan

- **File:**
  `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql:8-21`
- **Severity:** Medium
- **Category:** Performance / Maintainability
- **Description:** The Phase 4 reconciliation story (D-14) and any future
  "balances last updated more than N days ago" UI will sort/filter on
  `balance_updated_at`. The migration adds the column but no index. With a
  small-to-medium account count this is fine; with many accounts (per user) this
  becomes a sequential scan. Performance issues are out of v1 scope per
  `<review_focus>`, but flagging because the migration is the cheapest place to
  fix.
- **Suggested fix:** Add to up.sql:
  ```sql
  CREATE INDEX IF NOT EXISTS idx_accounts_balance_updated_at
      ON accounts (balance_updated_at) WHERE balance_updated_at IS NOT NULL;
  ```
  And to down.sql:
  ```sql
  DROP INDEX IF EXISTS idx_accounts_balance_updated_at;
  ```

#### M-02 — `down.sql` doesn't restore CHECK constraints if columns are re-added later

- **File:**
  `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql`
- **Severity:** Medium
- **Category:** Maintainability
- **Description:** `down.sql` correctly drops all 11 columns. However, the CHECK
  constraints on `statement_cycle_day` (1..31) and `reward_points_balance`
  (>= 0) are dropped along with their columns (CASCADE on column drop). If a
  future migration re-adds these columns without re-adding the CHECK clauses,
  validation falls solely to the Rust service layer. This is acceptable per the
  locked decision ("validation at service layer") but worth noting that the
  CHECK is defense-in-depth and shouldn't be removed silently.
- **Suggested fix:** Add a migration_tests.rs assertion that, after up, the
  constraints exist:
  ```rust
  // SELECT pg_catalog.pg_get_constraintdef(c.oid)
  //   FROM pg_constraint c WHERE conrelid = 'accounts'::regclass
  //   AND contype = 'c'
  ```
  Verify that statement_cycle_day's CHECK is present. Catches future drift.

#### M-03 — `From<AccountUpdate> for AccountDB` writes empty currency to a non-nullable column

- **File:** `crates/storage-postgres/src/accounts/model.rs:142-146`
- **Severity:** Medium
- **Category:** Bug (latent, masked by repository workaround)
- **Description:** The `From<AccountUpdate> for AccountDB` impl sets
  `currency: String::new()` (empty string) at line 142. The `accounts.currency`
  column is `NOT NULL`. The only reason this doesn't break is that
  `repository.rs:61` overwrites `account_db.currency = existing.currency;`
  before the UPDATE. This is fragile: a future refactor that drops the overwrite
  (or a new `update` caller that bypasses the repo's read-modify pattern) will
  write empty strings to a NOT NULL column. The same risk applies to
  `created_at: NaiveDateTime::default()` (line 145) which is also overwritten by
  line 62.
- **Suggested fix:** Make these fields explicitly never-overwrite via Diesel
  `AsChangeset` `#[diesel(skip_update)]` (or split the changeset struct) so that
  the `From` impl's placeholder values are never written to the DB even if the
  repo logic changes. Alternatively, add a clear
  `// SAFETY: overwritten by repository::update before UPDATE` doc comment on
  lines 142 and 145.

#### M-04 — `account_kind()` falls back to `Asset` for unknown types — silent forward-compat trap

- **File:** `crates/core/src/accounts/accounts_constants.rs:50-59`
- **Severity:** Medium
- **Category:** Bug (forward-compat)
- **Description:** When the broker sync stores legacy types like "RRSP", "TFSA",
  "401K", "MARGIN" (per `crates/connect/src/broker/models.rs:680-704`),
  `account_kind()` matches none of them and falls through to
  `_ => AccountKind::Asset`. Net-worth consumers using this helper will treat
  retirement accounts as Assets (not Investment), which is mostly correct but
  could inflate "Asset" totals or mis-categorize displays. The TS mirror at
  `apps/frontend/src/lib/constants.ts:113-117` uses an exhaustive
  `_exhaustive: never` check, but only over the closed AccountType union —
  broker-injected legacy types arrive as plain strings and bypass it.

  This is documented as "conservative default for forward-compat" in the
  comment, so it's intentional, but combine with M-05 below: many consumers
  default to `Icons.CreditCard` or `Icons.Wallet` — also silent fallthroughs.
  The accumulation of silent fallthroughs makes bugs hard to surface.

- **Suggested fix:** Either log a warning on unknown types in `account_kind`
  (debug-level, so it's not noisy in prod) or make the function return
  `Option<AccountKind>` and force callers to opt into the default. Lower
  priority since the broker-account currency-of-origin issue exists today; this
  just doesn't make it worse.

#### M-05 — `AccountItem` "Available" chip uses `bg-success/10 text-success` for a liability

- **File:**
  `apps/frontend/src/pages/settings/accounts/components/account-item.tsx:153`
- **Severity:** Medium
- **Category:** Style / UX semantics
- **Description:** The "Available credit" chip on a CREDIT_CARD row renders with
  `bg-success/10 text-success border-border` (green). This conflicts with the
  design semantics: green typically signals "good / asset positive." Available
  credit on a CC is a liability-side metric — high available = low debt, but the
  value itself isn't an asset. UI-SPEC §1
  - §6 specify color tiers (success / warning / destructive) keyed on
    utilization percent, not a flat green chip. The current chip ignores
    utilization tier entirely. Compare to the CC detail page
    (account-page.tsx:693-701) which DOES tier the Progress color via
    `utilizationTier(pct)`. Inconsistency between row chip and detail card.
- **Suggested fix:** Reuse `utilizationTier(pct)` to color the row chip:
  ```tsx
  const pct = utilizationPercent(account.creditLimit, account.currentBalance);
  const tier = utilizationTier(pct);
  const chipColor =
    tier === "destructive"
      ? "bg-destructive/10 text-destructive"
      : tier === "warning"
        ? "bg-warning/10 text-warning"
        : "bg-success/10 text-success";
  ```
  Or, if the chip is meant to be neutral, use `bg-muted text-foreground`.

#### M-06 — `is_archived: domain.is_archived.unwrap_or(false)` silently flips on missing input

- **File:** `crates/storage-postgres/src/accounts/model.rs:152`
- **Severity:** Medium
- **Category:** Bug
- **Description:** In `From<AccountUpdate> for AccountDB`, line 152 reads
  `is_archived: domain.is_archived.unwrap_or(false)`. The repository's update
  path (repository.rs:70-72) compensates by overriding with the existing value
  when `is_archived_provided` is false. But the model `From` impl alone reads as
  "default false" — fragile against the same refactor risk as M-03. If a future
  caller bypasses the repo's read-modify pattern and uses the `AccountDB`
  directly, this silently un-archives an archived account.
- **Suggested fix:** Same pattern as M-03 — either skip_update via Diesel
  attribute, or add an explicit `// SAFETY` doc comment, or make the field
  `Option<bool>` in the DB struct so `None` round-trips. The service-level guard
  exists; it should be belt-and-suspenders.

#### M-07 — UpdateBalanceModal "balance unchanged" guard uses `===` on `Number.parseFloat` results

- **File:**
  `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx:35`
- **Severity:** Medium
- **Category:** Bug (precision)
- **Description:** `unchanged = newBalance === currentBalanceNum`. Since both
  come from `Number.parseFloat(account.currentBalance)` (a string of arbitrary
  precision per NUMERIC(20,8)), this comparison is in `f64`. A balance like
  "100.00000001" round-trips to `100.00000001` in JS via `parseFloat` (fine for
  8 decimal places, JS double has 15-16 digits of mantissa), so this is unlikely
  to misfire in practice. But the comparison crosses the precision boundary
  unnecessarily — the modal could compare strings instead, since the data flows
  in/out as strings:
  ```ts
  const unchanged =
    newBalance === undefined || String(newBalance) === account.currentBalance;
  ```
  This guards correctness if the backend ever sends "1000.00" and the user types
  "1000" (currently considered unchanged → button disabled correctly).
  Lower-likelihood bug; flagging because money handling consistency is a Phase 3
  focus area per `<review_focus>`.
- **Suggested fix:** Compare as strings, or coerce both to a Decimal-like
  representation. The minimal fix is documented in the snippet above.

### Low

#### L-01 — `apps/server/src/models.rs:124-126` defaults `tracking_mode` to "NOT_SET" string but core enum default is also `NotSet` — duplication

- **File:** `apps/server/src/models.rs:124-126`
- **Severity:** Low
- **Category:** Maintainability
- **Description:** The DTO defines `default_tracking_mode() -> "NOT_SET"`
  manually, then `parse_tracking_mode("NOT_SET")` maps it back to
  `TrackingMode::NotSet`. This is consistent with the rest of the codebase
  (string-on-wire, enum-in-rust) but the constant "NOT_SET" appears in three
  places now: `accounts_model.rs` `serde` rename, `accounts/model.rs` `From`
  impl (line 91), and here. Drift risk.
- **Suggested fix:** Export a `pub const NOT_SET_STR: &str = "NOT_SET";` from
  `core::accounts` and reuse. Or, simpler, expose a public
  `TrackingMode::as_str()` method.

#### L-02 — `account-form.tsx:79` `needsSetup` logic flips the `trackingMode` default to undefined; React-Hook-Form treats undefined and "" identically

- **File:**
  `apps/frontend/src/pages/settings/accounts/components/account-form.tsx:79-92`
- **Severity:** Low
- **Category:** Maintainability
- **Description:** The submit button is disabled when
  `needsSetup && !currentTrackingMode`. The `RadioGroup` value is a controlled
  `field.value` from RHF. The combination works, but the logic — "if NOT_SET or
  undefined, force user to pick"— is duplicated with the "Tracking Mode" alert.
  A future refactor that deletes the warning alert without removing the disable
  guard will surface as a silently-clickable submit. Low priority.
- **Suggested fix:** Extract `needsSetup && !currentTrackingMode` into a named
  const `mustChooseTrackingMode` and use it in both the alert predicate and the
  submit `disabled` predicate.

#### L-03 — Account form's `requiresInstitution = requiresOpeningBalance` aliasing reads as a logic bug

- **File:**
  `apps/frontend/src/pages/settings/accounts/components/account-form.tsx:97-102`
- **Severity:** Low
- **Category:** Maintainability
- **Description:** The variables are set equal:
  `const requiresOpeningBalance = requiresInstitution;`. The intent — "same set
  per D-11" — is correct (institution is required iff openingBalance is
  required), but the alias is fragile: if a future revision narrows institution
  scope but not openingBalance, the dev will have to spot the alias.
- **Suggested fix:** Inline the same condition twice, or factor into a helper:
  ```ts
  const isBankCcOrLoan = (t?: string) =>
    t === "CHECKING" || t === "SAVINGS" || t === "CREDIT_CARD" || t === "LOAN";
  const requiresInstitution = isBankCcOrLoan(selectedType);
  const requiresOpeningBalance = isBankCcOrLoan(selectedType);
  ```

#### L-04 — `accountTypeIcons` per-file duplication across selectors and AccountItem

- **File:** `apps/frontend/src/components/account-selector.tsx:26-35`,
  `apps/frontend/src/components/account-selector-mobile.tsx:24-33`,
  `apps/frontend/src/components/app-launcher.tsx:65-74`,
  `apps/frontend/src/pages/account/account-page.tsx:89-97`,
  `apps/frontend/src/pages/settings/accounts/components/account-item.tsx:11-47`
- **Severity:** Low
- **Category:** Maintainability
- **Description:** Five files now define an `accountTypeIcons` map with
  identical CHECKING/SAVINGS/CREDIT_CARD/LOAN entries. When a sixth AccountType
  is added (e.g., "INVESTMENT_TAXABLE"), the dev must remember all five
  locations. This is pre-existing for SECURITIES / CASH / CRYPTOCURRENCY but
  Phase 3 doubled the surface.
- **Suggested fix:** Move to a shared module
  `apps/frontend/src/lib/account-icons.ts`:
  ```ts
  export const ACCOUNT_TYPE_ICONS: Record<AccountType, Icon> = {
    SECURITIES: Icons.Briefcase,
    CASH: Icons.DollarSign,
    CRYPTOCURRENCY: Icons.Bitcoin,
    CHECKING: Icons.Wallet,
    SAVINGS: Icons.Coins,
    CREDIT_CARD: Icons.CreditCard,
    LOAN: Icons.Building,
  };
  export function iconForAccountType(t: AccountType): Icon {
    return ACCOUNT_TYPE_ICONS[t] ?? Icons.Wallet;
  }
  ```
  CLAUDE.md "Surgical Changes" guideline applies — fix this only if another
  consumer is added later, not as drive-by cleanup.

#### L-05 — `account-page.tsx:781` "pts" suffix is hard-coded English

- **File:** `apps/frontend/src/pages/account/account-page.tsx:781`
- **Severity:** Low
- **Category:** Maintainability (i18n)
- **Description:** Reward points balance is rendered as
  `${account.rewardPointsBalance.toLocaleString()} pts`. The "pts" suffix is
  hard-coded. The codebase doesn't have i18n today (every string is English), so
  this is consistent with surrounding code, but worth noting if i18n is in the
  roadmap.
- **Suggested fix:** None for v1. Flag for future i18n pass.

#### L-06 — `e2e/11-accounts.spec.ts:218` uses `page.waitForTimeout(500)` after archive

- **File:** `e2e/11-accounts.spec.ts:218`
- **Severity:** Low
- **Category:** Maintainability (test reliability)
- **Description:** A hard 500ms sleep is placed after the archive confirm click
  "to let the optimistic update settle." This is a known flake vector — slower
  CI machines may need >500ms. Better to wait for a positive signal (e.g., the
  row's `Archived` badge to appear in another part of the page or the dialog to
  close).
- **Suggested fix:** Replace with an assertion-based wait. The test already
  follows up with
  `expect(page.getByRole("link", { name: checkingName })).toHaveCount(0)` —
  that's the actual signal. Drop the timeout and let the assertion's default
  Playwright retry handle it. (Or, if the assertion races, wait for the alert
  dialog to unmount:
  `await expect(archiveDialog).not.toBeVisible({ timeout: 5_000 })` — already
  done at line 215, so the timeout is redundant.)

#### L-07 — `account-form.tsx:255-260` re-parses `field.value` to number on every render

- **File:**
  `apps/frontend/src/pages/settings/accounts/components/account-form.tsx:255-260`
  and identical patterns at 284, 333, 411
- **Severity:** Low
- **Category:** Maintainability (precision)
- **Description:** `MoneyInput` value is computed via
  `field.value ? Number.parseFloat(field.value) : undefined`. Per the CLAUDE.md
  money-handling guidance and Phase 3's NUMERIC(20,8) decision (D-10),
  Decimal-as-string is the wire format precisely to avoid f64 drift. Routing
  through `parseFloat` truncates to f64 precision (15-16 digits), then
  `String(v)` on the way back may produce a different representation (e.g.,
  "1000.00" → 1000 → "1000"). For balances inside the f64 safe range (most
  users), this is fine. For very-precise values (e.g., crypto-equivalent CC
  balances stored to 8 decimal places), the form normalizes the display.
  Documented in CONTEXT.md D-10, so this is by design — but the code repeats the
  pattern four times without a helper.
- **Suggested fix:** Extract a small util:
  ```ts
  const decimalStringToNumber = (v: string | undefined): number | undefined =>
    v ? Number.parseFloat(v) : undefined;
  const numberToDecimalString = (
    v: number | null | undefined,
  ): string | undefined =>
    v !== null && v !== undefined ? String(v) : undefined;
  ```
  and apply across the four MoneyInput usages.

#### L-08 — `holdings-mobile-filter-sheet.tsx:153-161` constructs a synthetic Account without `currentBalance` rename

- **File:**
  `apps/frontend/src/pages/holdings/components/holdings-mobile-filter-sheet.tsx:151-161`
- **Severity:** Low
- **Category:** Maintainability
- **Description:** The synthetic "All Portfolio" pseudo-account is constructed
  as a literal — same pattern as `holdings-page.tsx:57-67` and
  `portfolio-insights.tsx:37-47`. None of these set the new Phase 3 fields
  (institution, openingBalance, etc.) which is fine because they are optional,
  but the lack of a single helper means a future field addition (e.g., the
  `tracking_mode` enum gaining a fourth variant) has three sites to update.
  `lib/constants.ts:18-32` already exports a
  `createPortfolioAccount(baseCurrency)` helper but these three files don't use
  it (likely pre-existing).
- **Suggested fix:** Out of Phase 3 scope. Flag for cleanup. The Phase 3 changes
  did not introduce these duplicates; they only brushed past them.

## Files Reviewed

- `apps/frontend/src/components/account-selector-mobile.tsx`
- `apps/frontend/src/components/account-selector.tsx`
- `apps/frontend/src/components/app-launcher.tsx`
- `apps/frontend/src/lib/constants.test.ts`
- `apps/frontend/src/lib/constants.ts`
- `apps/frontend/src/lib/schemas.test.ts`
- `apps/frontend/src/lib/schemas.ts`
- `apps/frontend/src/lib/types/account.ts`
- `apps/frontend/src/pages/account/account-page.tsx`
- `apps/frontend/src/pages/activity/components/activity-data-grid/activity-utils.test.ts`
- `apps/frontend/src/pages/dashboard/accounts-summary.test.tsx`
- `apps/frontend/src/pages/holdings/components/holdings-mobile-filter-sheet.tsx`
- `apps/frontend/src/pages/holdings/holdings-page.tsx`
- `apps/frontend/src/pages/insights/portfolio-insights.tsx`
- `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`
- `apps/frontend/src/pages/settings/accounts/accounts-page.tsx`
- `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx`
- `apps/frontend/src/pages/settings/accounts/components/account-item.tsx`
- `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx`
- `apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts`
- `apps/frontend/src/pages/settings/accounts/credit-helpers.ts`
- `apps/server/src/models.rs`
- `crates/connect/src/broker/service.rs`
- `crates/core/src/accounts/accounts_constants.rs`
- `crates/core/src/accounts/accounts_model.rs`
- `crates/core/src/accounts/accounts_model_tests.rs`
- `crates/core/src/accounts/accounts_service.rs`
- `crates/core/src/accounts/accounts_service_tests.rs`
- `crates/core/src/accounts/mod.rs`
- `crates/core/src/activities/activities_service_tests.rs`
- `crates/core/src/portfolio/net_worth/net_worth_service_tests.rs`
- `crates/core/src/portfolio/snapshot/snapshot_service.rs`
- `crates/core/src/portfolio/snapshot/snapshot_service_tests.rs`
- `crates/storage-postgres/Cargo.toml`
- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql`
- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql`
- `crates/storage-postgres/src/accounts/migration_tests.rs`
- `crates/storage-postgres/src/accounts/mod.rs`
- `crates/storage-postgres/src/accounts/model.rs`
- `crates/storage-postgres/src/accounts/repository_tests.rs`
- `crates/storage-postgres/src/db/mod.rs`
- `crates/storage-postgres/src/schema.rs`
- `e2e/11-accounts.spec.ts`
- `packages/addon-sdk/src/data-types.ts`

---

_Reviewed: 2026-04-25T00:00:00Z_ _Reviewer: Claude (gsd-code-reviewer)_ _Depth:
standard_
