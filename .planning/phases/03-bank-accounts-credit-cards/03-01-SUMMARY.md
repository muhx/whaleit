---
phase: 03-bank-accounts-credit-cards
plan: 01
subsystem: database
tags: [postgres, diesel, migration, schema, numeric, rust_decimal]

# Dependency graph
requires:
  - phase: 02-dual-database-engine
    provides:
      "PostgreSQL crate (storage-postgres), diesel-async pool,
      embed_migrations!() boot path, hand-synchronized schema.rs convention"
provides:
  - "11 new nullable columns on the accounts table for bank accounts, credit
    cards, and balance fields"
  - "rust_decimal::Decimal can round-trip through PG NUMERIC columns via Diesel"
  - "Migration 20260425000000_accounts_extend_types_and_balances ready for
    embed_migrations!() to apply on next server boot"
  - "Updated diesel DSL for the accounts table covering all 11 new columns"
affects:
  - 03-02-account-domain-model
  - 03-03-account-repository
  - 03-04-account-service
  - 03-05-account-commands
  - 03-06-account-ui
  - 03-07-account-edit-flow
  - 03-08-account-balance-update

# Tech tracking
tech-stack:
  added:
    - "rust_decimal feature: db-diesel2-postgres (ToSql/FromSql for Numeric)"
  patterns:
    - "Additive ALTER TABLE migrations (no destructive changes to existing
      columns)"
    - "Inline CHECK constraints attached to columns at creation"
    - "NUMERIC(20,8) for new money columns (D-10) — coexists with legacy
      TEXT-stored money columns from initial_schema"
    - "schema.rs hand-synchronization (no [print_schema] block in diesel.toml)"

key-files:
  created:
    - "crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql"
    - "crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql"
  modified:
    - "Cargo.toml (rust_decimal feature flag)"
    - "crates/storage-postgres/src/schema.rs (accounts block: +11 columns)"
    - ".planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md (Per-Task
      Verification Map row updated)"
    - ".gitignore (db/ -> /db/, recovery deviation)"
    - "crates/storage-postgres/src/db/mod.rs (restored from main repo, recovery
      deviation)"

key-decisions:
  - "D-10 enforced literally: NUMERIC(20,8) for the 6 new money columns
    (opening_balance, current_balance, credit_limit, statement_balance,
    minimum_payment, cashback_balance). Diverges from the existing TEXT-storage
    pattern for legacy money columns; legacy columns left untouched."
  - "CHECK constraints kept minimal per RESEARCH.md: only statement_cycle_day
    BETWEEN 1 AND 31 and reward_points_balance >= 0. Service layer owns
    money-range validation per T-3-03."
  - "schema.rs hand-edited (diesel.toml has no [print_schema] block); no new
    sql_types imports needed because Numeric/SmallInt/Date/Integer were already
    in use elsewhere in the file."
  - "Migration not applied to a live DB — DATABASE_URL was unset and diesel CLI
    was not available. embed_migrations!() will apply on next server boot per
    the plan's documented fallback. DSL is verified at compile time against the
    migration."

patterns-established:
  - "Phase 3 money column convention: new monetary fields use NUMERIC(20,8);
    legacy TEXT-stored money fields are not migrated — coexistence is
    intentional."
  - "Inline CHECK constraints: define constraint at column creation so it drops
    automatically when the column drops."

requirements-completed: [ACCT-01, ACCT-02, ACCT-05, ACCT-06, ACCT-07]

# Metrics
duration: 6min
completed: 2026-04-25
---

# Phase 3 Plan 01: Accounts schema extension Summary

**Additive PG migration adding 11 nullable account columns (institution + 6
NUMERIC(20,8) money fields + cycle_day SMALLINT + due_date DATE + reward_points
INTEGER + balance_updated_at TIMESTAMP), rust_decimal db-diesel2-postgres
feature flip, and hand-regenerated schema.rs DSL.**

## Performance

- **Duration:** ~6 min (348 s)
- **Started:** 2026-04-25T03:43:59Z
- **Completed:** 2026-04-25T03:49:47Z
- **Tasks:** 3 / 3
- **Files modified:** 5 (plus 2 from the recovery deviation)

## Accomplishments

- Cargo.toml `rust_decimal` workspace dep gains `db-diesel2-postgres` feature,
  wiring `ToSql`/`FromSql` for `diesel::sql_types::Numeric`.
  `cargo build -p whaleit-storage-postgres` finishes clean (313 crates
  compiled).
- New migration directory `20260425000000_accounts_extend_types_and_balances/`
  with both `up.sql` (11 ADD COLUMN clauses + 2 inline CHECK constraints) and
  `down.sql` (11 idempotent DROP COLUMN IF EXISTS clauses).
- `crates/storage-postgres/src/schema.rs` accounts block extended with 11 column
  declarations matching the migration types.
  `cargo check -p whaleit-storage-postgres` passes (0 errors).
- `03-VALIDATION.md` Per-Task Verification Map updated: migration row marked
  Plan `03-01`, Task `3`, Status `✅`.

## Task Commits

Each task was committed atomically (with `--no-verify` per parallel-executor
guidance):

1. **[Recovery deviation]** Restore db/mod.rs and narrow gitignore — `53ea8c9f`
   (fix)
2. **Task 1:** Enable rust_decimal db-diesel2-postgres feature — `6bb46230`
   (chore)
3. **Task 2:** Add migration up.sql + down.sql for 11 new columns — `7430ba63`
   (feat)
4. **Task 3:** Regenerate schema.rs accounts block + VALIDATION map — `a9f4639f`
   (feat)

(SUMMARY commit will follow.)

## Files Created/Modified

- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql`
  — 11 ADD COLUMN clauses with NUMERIC(20,8) for the 6 money columns,
  SMALLINT/INTEGER/DATE/TIMESTAMP/TEXT for the rest, plus 2 inline CHECK
  constraints (cycle_day 1..=31, points >= 0).
- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql`
  — 11 idempotent DROP COLUMN IF EXISTS reversing every up.sql column addition.
- `Cargo.toml` — single-line change adding `db-diesel2-postgres` to the
  rust_decimal feature list.
- `crates/storage-postgres/src/schema.rs` — accounts block extended with 11
  column declarations (Nullable<Text|Numeric|Timestamp|SmallInt|Date|Integer>);
  no other table block touched.
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` — first row
  of Per-Task Verification Map updated with Plan `03-01`, Task ID `3`, Status
  `✅`.
- `.gitignore` (recovery deviation) — narrowed `db/` to `/db/` so it stops
  matching `crates/storage-postgres/src/db/`.
- `crates/storage-postgres/src/db/mod.rs` (recovery deviation) — restored from
  the main repo working copy because it was excluded from version control by the
  broad `db/` ignore pattern.

## Decisions Made

- Followed plan literally for the 6 money columns: `NUMERIC(20,8)` per D-10,
  even though the legacy `initial_schema` migration stores money as `TEXT`. The
  two patterns intentionally coexist; only NEW Phase 3 columns are NUMERIC.
- Did NOT apply the migration to a live database. The plan's step 5 fallback
  applies: no `DATABASE_URL` was set in this worktree's env and `diesel` CLI was
  not installed. `embed_migrations!()` at server boot will pick up the new
  migration. The schema.rs DSL is still verified at compile time against the
  migration SQL.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking + Rule 1 - Bug] Restored
crates/storage-postgres/src/db/mod.rs and narrowed gitignore**

- **Found during:** Pre-task setup (running baseline
  `cargo check -p whaleit-storage-postgres`)
- **Issue:** The crate failed to compile with
  `error[E0583]: file not found for module 'db'`.
  `crates/storage-postgres/src/db/mod.rs` was missing from this worktree, but
  `lib.rs:22` declares `pub mod db;` and many repositories
  `use crate::db::PgPool`. Investigation showed `.gitignore:90` had a literal
  `db/` pattern that matches `db/` ANYWHERE in the tree, not just at root.
  `git check-ignore -v crates/storage-postgres/src/db/mod.rs` returned
  `.gitignore:90:db/`, confirming the file was silently excluded from version
  control. The intent was almost certainly to ignore a top-level `db/` directory
  used for local PG data.
- **Fix:**
  - Narrowed `.gitignore:90` from `db/` to `/db/` (root-only).
  - Restored `crates/storage-postgres/src/db/mod.rs` by copying it from the
    parent repo's working copy (78 lines, defines `PgPool`, `PgConnection`,
    `create_pool`, `init`, `run_migrations`, and the `embed_migrations!()`
    const).
- **Files modified:** `.gitignore`, `crates/storage-postgres/src/db/mod.rs`
  (created)
- **Verification:** After the fix, `git check-ignore -v` reports no match for
  the path, and `cargo check -p whaleit-storage-postgres` finished clean
  (Finished `dev` profile in 13.45s) before any plan task ran. This established
  the baseline needed to verify Tasks 1 and 3.
- **Committed in:** `53ea8c9f` (separate `fix(storage-postgres):` commit, NOT
  bundled into a plan task — clearly tagged as a recovery action).

---

**Total deviations:** 1 auto-fixed (Rules 3 + 1 — environment recovery).
**Impact on plan:** Recovery deviation was strictly required to make any
verification possible. Scope kept tight: minimal `.gitignore` edit + restoration
of a single file from the parent repo. Did NOT touch any other ignored files.
Did NOT investigate why this file was never committed — that's a project
housekeeping question, not a Phase 3 question.

## Issues Encountered

- `cargo check -p whaleit-storage-postgres` initially reported 301 errors before
  the recovery deviation. After restoring `db/mod.rs`, the count dropped to 0.
  None of those 301 errors were caused by plan tasks; all were the cascade from
  the missing `db` module.
- No `DATABASE_URL` and no `diesel` CLI were available, so the plan's "live
  `diesel migration run`" step (Task 3 step 2) was skipped per the documented
  fallback. `embed_migrations!()` will apply the migration on the next server
  boot. Diesel DSL is still verified at compile time.

## User Setup Required

None — no external service configuration required for this plan.

## Next Phase Readiness

- Plan 03-02 (account domain model) and 03-03 (account repository) can now read
  the new columns through the updated `accounts` DSL and round-trip `Decimal` to
  `Numeric`.
- Plan 03-05 (commands) and Wave 3 UI plans depend on subsequent plans; this
  storage layer is in place for them.
- **Carry-forward note for the next executor:** the
  `crates/storage-postgres/src/db/mod.rs` restoration is now committed to this
  worktree branch. When this branch merges back to the parent repo, the
  `.gitignore` narrowing will follow it. If the parent repo has a separate copy
  of `db/mod.rs` already in the working tree (which it does, as evidenced by
  being able to copy it from there), no conflict — the merge brings the file
  under git's tracking for the first time.
- **Open project housekeeping (out of scope):** worth confirming with the team
  whether the broad `db/` ignore was intentional. If a top-level `db/` directory
  is used for local PG data, `/db/` is the correct scope. If not, line 90 can be
  removed entirely (lines 59-61 already cover `apps/db/` and friends).

---

_Phase: 03-bank-accounts-credit-cards_ _Completed: 2026-04-25_

## Self-Check: PASSED

All 8 claimed files exist on disk:

- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql`
  ✅
- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql`
  ✅
- `Cargo.toml` ✅
- `crates/storage-postgres/src/schema.rs` ✅
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` ✅
- `.gitignore` ✅
- `crates/storage-postgres/src/db/mod.rs` ✅
- `.planning/phases/03-bank-accounts-credit-cards/03-01-SUMMARY.md` (this file)
  ✅

All 4 claimed commits present in git log:

- `53ea8c9f` (recovery deviation) ✅
- `6bb46230` (Task 1) ✅
- `7430ba63` (Task 2) ✅
- `a9f4639f` (Task 3) ✅
