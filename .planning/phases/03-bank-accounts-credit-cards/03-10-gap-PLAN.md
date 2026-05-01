---
phase: 03-bank-accounts-credit-cards
plan: 10
type: execute
wave: 5
depends_on: ["03-02", "03-03", "03-04"]
gap_closure: true
files_modified:
  - apps/server/src/models.rs
  - crates/core/src/accounts/accounts_model.rs
  - crates/core/src/accounts/accounts_service.rs
  - crates/core/src/accounts/accounts_service_tests.rs
autonomous: true
requirements: [ACCT-04]
requirements_addressed: [ACCT-04]
threats_addressed: [T-3-02]
tags:
  [
    gap-closure,
    backend,
    rust,
    invariants,
    h-02,
    h-03,
    d-06,
    d-12,
    accounts-update,
  ]

must_haves:
  truths:
    - "Updating a CREDIT_CARD account's account_type to CHECKING / SAVINGS /
      LOAN clears all 7 CC-only columns to NULL in the same UPDATE (D-06
      invariant restored on type transitions)"
    - "A client that submits balanceUpdatedAt with no balance change cannot
      shift the server's last-touched timestamp (D-12 invariant: server is sole
      writer of balance_updated_at)"
    - "A client that submits balanceUpdatedAt during a balance change is ignored
      — the server still stamps the field with chrono::Utc::now()"
    - "AccountUpdate auto-bump test (existing) still passes; two new
      service-level tests pass for D-12 client-supplied-timestamp rejection and
      D-06 type-transition CC-field clearing"
  artifacts:
    - path: "apps/server/src/models.rs"
      provides:
        "HTTP DTOs that no longer accept the server-only balance_updated_at
        field on AccountUpdate or NewAccount; clients cannot inject the
        timestamp"
      contains: "// balance_updated_at is server-only"
    - path: "crates/core/src/accounts/accounts_service.rs"
      provides:
        "update_account that NULLs CC fields when type transitions out of
        CREDIT_CARD AND ignores any client-supplied balance_updated_at on every
        update path (D-12 + D-06)"
      contains: "type_transition_out_of_cc"
    - path: "crates/core/src/accounts/accounts_service_tests.rs"
      provides:
        "Two new tests: D-12 client-cannot-set-balance_updated_at
        (no-balance-change path) and D-06 cc-fields-cleared-on-type-change path"
      contains: "test_update_ignores_client_supplied_balance_updated_at"
  key_links:
    - from: "apps/server/src/models.rs::AccountUpdate"
      to: "crates/core/src/accounts/accounts_model.rs::AccountUpdate"
      via:
        "From<server::AccountUpdate> for core::AccountUpdate impl that discards
        client-supplied balance_updated_at"
      pattern: "balance_updated_at: None"
    - from: "crates/core/src/accounts/accounts_service.rs::update_account"
      to:
        "crates/core/src/accounts/accounts_model.rs::AccountUpdate (CC fields)"
      via:
        "type-transition detection: if existing.account_type == CREDIT_CARD &&
        updated.account_type != CREDIT_CARD, set all 7 CC fields to None on the
        AccountUpdate before passing to repository"
      pattern: "account_types::CREDIT_CARD"
---

<objective>
Close 2 carry-forward High-severity issues from 03-REVIEW.md that are not
goal-blocking today (because H-01 hides them behind a broken edit form) but
become user-reachable the moment Plan 03-09 closes. Bundling now keeps the
phase invariants honest.

- **H-02 (D-06 invariant):** AccountUpdate::validate() does not clear CC fields
  when account_type transitions out of CREDIT_CARD. Diesel AsChangeset's default
  Option<T> = "skip-column on None" semantics leave stale credit_limit /
  statement_cycle_day / statement_balance / minimum_payment / statement_due_date
  / reward_points_balance / cashback_balance in the row.
- **H-03 (D-12 invariant):** HTTP AccountUpdate DTO accepts client-supplied
  balance_updated_at. The service auto-stamp only overrides when current_balance
  changes, so a client can backdate / future-date the field by sending it
  without a balance change.

Purpose: Restore the D-06 and D-12 invariants the migration and service layer
depend on. Both are server-side correctness fixes; no UI changes.

Output: Three Rust files modified (server DTO, core service, core service-tests)
and the existing Phase 3 test ledger extended by 2 tests. </objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-bank-accounts-credit-cards/03-CONTEXT.md
@.planning/phases/03-bank-accounts-credit-cards/03-VERIFICATION.md
@.planning/phases/03-bank-accounts-credit-cards/03-REVIEW.md
@.planning/phases/03-bank-accounts-credit-cards/03-02-SUMMARY.md
@.planning/phases/03-bank-accounts-credit-cards/03-03-SUMMARY.md
@.planning/phases/03-bank-accounts-credit-cards/03-04-SUMMARY.md

@crates/core/src/accounts/accounts_model.rs
@crates/core/src/accounts/accounts_service.rs
@crates/core/src/accounts/accounts_service_tests.rs
@crates/core/src/accounts/accounts_constants.rs @apps/server/src/models.rs
@crates/storage-postgres/src/accounts/repository.rs

<interfaces>
<!-- Key contracts the executor needs. Pre-extracted so executor can mirror
     existing patterns without scavenger-hunting. -->

From crates/core/src/accounts/accounts_model.rs (lines 200-269) — the core
AccountUpdate struct AND its validate() impl. The fix lives in update_account on
accounts_service.rs (NOT in validate), so this is just to confirm the field
shape:

```rust
pub struct AccountUpdate {
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    // ... non-CC fields ...
    pub current_balance: Option<Decimal>,
    pub balance_updated_at: Option<NaiveDateTime>,
    pub credit_limit: Option<Decimal>,
    pub statement_cycle_day: Option<i16>,
    pub statement_balance: Option<Decimal>,
    pub minimum_payment: Option<Decimal>,
    pub statement_due_date: Option<NaiveDate>,
    pub reward_points_balance: Option<i32>,
    pub cashback_balance: Option<Decimal>,
}

impl AccountUpdate {
    pub fn validate(&self) -> Result<()> {
        // ... existing checks ...
        // D-06: same null-rule on updates.
        let is_credit_card = self.account_type == account_types::CREDIT_CARD;
        if !is_credit_card {
            if self.credit_limit.is_some() || ... {
                return Err(...);  // rejects positive CC fields on non-CC type
            }
        }
        Ok(())
    }
}
```

The validate() rule is correct as far as it goes — it rejects a request that
SENDS positive CC values with a non-CC type. The bug is that `Option<T>::None`
for a Diesel AsChangeset means "skip column", so a caller submitting a non-CC
type with `credit_limit: None` passes validate but leaves the existing column
intact. Fix is in service-layer logic (actively NULL the columns on type
transition), NOT in validate.

From crates/core/src/accounts/accounts_constants.rs — the CREDIT_CARD constant
the service must compare against:

```rust
pub mod account_types {
    pub const CHECKING: &str = "CHECKING";
    pub const SAVINGS: &str = "SAVINGS";
    pub const CREDIT_CARD: &str = "CREDIT_CARD";
    pub const LOAN: &str = "LOAN";
    // ... other types ...
}
```

From crates/core/src/accounts/accounts_service.rs (lines 79-99) — the existing
update_account auto-bump block. The fix extends this block:

```rust
async fn update_account(&self, account_update: AccountUpdate) -> Result<Account> {
    let account_id = account_update.id.as_ref().ok_or_else(|| { ... })?;
    let existing = self.repository.get_by_id(account_id).await?;

    // D-12: auto-stamp balance_updated_at when current_balance changes.
    let mut account_update = account_update;
    if account_update.current_balance.is_some()
        && account_update.current_balance != existing.current_balance
    {
        account_update.balance_updated_at = Some(chrono::Utc::now().naive_utc());
    }

    let result = self.repository.update(account_update).await?;
    // ...
}
```

From apps/server/src/models.rs (lines 110, 189, 203-232) — the HTTP DTOs. Two
locations need the H-03 fix:

1. The AccountUpdate DTO at line 189 declares
   `pub balance_updated_at: Option<NaiveDateTime>,`.
2. The NewAccount DTO at line 110 declares the same.
3. The From<server::AccountUpdate> for core::AccountUpdate impl at lines 203-232
   currently passes `a.balance_updated_at` straight through.
4. The From<server::NewAccount> for core::NewAccount at lines 136-166 does the
   same.

From crates/core/src/accounts/accounts_service_tests.rs (lines 282-381) — the
existing test scaffolding. Pattern for new tests:

```rust
fn make_service(existing: Account) -> (AccountService, Arc<MockAccountRepo>) { ... }
fn existing_cc(current: Option<Decimal>) -> Account { ... }
fn update_with_balance(id: &str, balance: Option<Decimal>) -> AccountUpdate { ... }

#[tokio::test]
async fn test_update_no_bump_when_balance_unchanged() {
    let (service, repo) = make_service(existing_cc(Some(dec!(100))));
    let update = update_with_balance("acc-1", Some(dec!(100)));
    service.update_account(update).await.unwrap();
    let captured = repo.last_update.lock().unwrap().clone().unwrap();
    assert!(captured.balance_updated_at.is_none(), "...");
}
```

The mock repo records the AccountUpdate it receives via
`*self.last_update.lock().unwrap() = Some(account_update.clone());`, which lets
the new tests assert what the service forwarded. </interfaces> </context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add 2 service-level tests for D-12 client-supplied timestamp + D-06 CC-field-clear-on-type-change (RED)</name>

<read_first> - crates/core/src/accounts/accounts_service_tests.rs (full file —
copy the existing test scaffolding pattern, especially `existing_cc`,
`update_with_balance`, and `MockAccountRepo`) -
crates/core/src/accounts/accounts_service.rs (lines 79-99 — current
update_account body, the location of the fix in Task 2) -
crates/core/src/accounts/accounts_model.rs (lines 200-269 — AccountUpdate field
shape) - crates/core/src/accounts/accounts_constants.rs (account_types module —
to import account_types::CREDIT_CARD / CHECKING) </read_first>

  <files>
    crates/core/src/accounts/accounts_service_tests.rs
  </files>

  <behavior>
    - Test 1: `test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged`
      * Arrange: existing CC with current_balance Some(dec!(100)).
      * Act: update_account with current_balance: Some(dec!(100))
        (unchanged) AND balance_updated_at: Some(<a fixed past timestamp,
        e.g., NaiveDateTime::from_timestamp_opt(0, 0).unwrap()>).
      * Assert: the AccountUpdate captured by the mock repo has
        balance_updated_at == None (server discarded the client value;
        no auto-stamp because balance is unchanged).

    - Test 2: `test_update_clears_cc_fields_on_type_transition_out_of_cc`
      * Arrange: existing CC with current_balance Some(dec!(50)),
        credit_limit Some(dec!(5000)), statement_cycle_day Some(15),
        statement_balance Some(dec!(123.45)), minimum_payment
        Some(dec!(25)), statement_due_date Some(NaiveDate::from_ymd_opt
        (2026, 5, 1).unwrap()), reward_points_balance Some(1000),
        cashback_balance Some(dec!(12.34)).
      * Act: update_account with account_type "CHECKING" AND all 7 CC
        fields explicitly set to None on the AccountUpdate (a normal
        update payload from a client that doesn't know to NULL them).
      * Assert: the AccountUpdate captured by the mock repo has ALL 7 CC
        fields == None. (The fix in Task 2 actively sets them to None
        when the type transitions out of CREDIT_CARD; this test
        currently fails because today's service passes them through
        as-supplied, which means a client could leave them populated by
        not zeroing them.)
      * Note: This test is also valuable as a forward-compat guard. It
        does NOT assert what the repository writes to PG (that's
        repository_tests.rs territory and skipped without DATABASE_URL);
        it asserts the SERVICE-LAYER intent, which is the correct
        contract boundary.

  </behavior>

  <action>
    Append two new `#[tokio::test]` functions to the existing
    `mod tests { ... }` block in
    crates/core/src/accounts/accounts_service_tests.rs, AFTER the
    `test_update_no_bump_when_no_balance_in_update` function (which is the
    last test today, ending around line 381).

    Reuse the existing `make_service`, `existing_cc`,
    `update_with_balance` builders. Add two new helpers if needed:

    ```rust
    fn existing_cc_full() -> Account {
        Account {
            id: "acc-1".to_string(),
            name: "Card".to_string(),
            account_type: "CREDIT_CARD".to_string(),
            currency: "USD".to_string(),
            is_active: true,
            tracking_mode: TrackingMode::Transactions,
            current_balance: Some(dec!(50)),
            credit_limit: Some(dec!(5000)),
            statement_cycle_day: Some(15),
            statement_balance: Some(dec!(123.45)),
            minimum_payment: Some(dec!(25)),
            statement_due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
            reward_points_balance: Some(1000),
            cashback_balance: Some(dec!(12.34)),
            ..Default::default()
        }
    }

    fn update_with_type_change(id: &str, new_type: &str) -> AccountUpdate {
        AccountUpdate {
            id: Some(id.to_string()),
            name: "Card".to_string(),
            account_type: new_type.to_string(),
            // Client doesn't actively NULL the CC fields — relies on
            // server to enforce D-06 invariant.
            credit_limit: None,
            statement_cycle_day: None,
            statement_balance: None,
            minimum_payment: None,
            statement_due_date: None,
            reward_points_balance: None,
            cashback_balance: None,
            // ... all other fields = sensible defaults / None ...
            // (Mirror the shape of update_with_balance() at line 313;
            //  copy it and change the account_type field.)
            ..Default::default()
        }
    }
    ```

    Note: AccountUpdate may not derive Default (verify by reading
    accounts_model.rs:184-234). If it does NOT derive Default, you MUST
    spell out every field explicitly per the existing
    update_with_balance() pattern at line 313-340. Do NOT add a Default
    derive to AccountUpdate as drive-by; it's a behavioral change outside
    this gap's scope. Spell the fields.

    Then add the two tests:

    ```rust
    #[tokio::test]
    async fn test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged() {
        let (service, repo) = make_service(existing_cc(Some(dec!(100))));
        let mut update = update_with_balance("acc-1", Some(dec!(100))); // unchanged
        // Client tries to backdate the timestamp:
        update.balance_updated_at = Some(
            chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
        );
        service.update_account(update).await.unwrap();

        let captured = repo.last_update.lock().unwrap().clone().unwrap();
        assert!(
            captured.balance_updated_at.is_none(),
            "D-12: server must discard client-supplied balance_updated_at \
             when balance is unchanged. Got {:?}",
            captured.balance_updated_at
        );
    }

    #[tokio::test]
    async fn test_update_clears_cc_fields_on_type_transition_out_of_cc() {
        let (service, repo) = make_service(existing_cc_full());
        let update = update_with_type_change("acc-1", "CHECKING");
        service.update_account(update).await.unwrap();

        let captured = repo.last_update.lock().unwrap().clone().unwrap();
        assert!(captured.credit_limit.is_none(),         "D-06: credit_limit must be None on type transition out of CC");
        assert!(captured.statement_cycle_day.is_none(),  "D-06: statement_cycle_day must be None on type transition out of CC");
        assert!(captured.statement_balance.is_none(),    "D-06: statement_balance must be None on type transition out of CC");
        assert!(captured.minimum_payment.is_none(),      "D-06: minimum_payment must be None on type transition out of CC");
        assert!(captured.statement_due_date.is_none(),   "D-06: statement_due_date must be None on type transition out of CC");
        assert!(captured.reward_points_balance.is_none(),"D-06: reward_points_balance must be None on type transition out of CC");
        assert!(captured.cashback_balance.is_none(),     "D-06: cashback_balance must be None on type transition out of CC");
    }
    ```

    Imports needed at the top of the existing `mod tests` (most are already
    there; verify and add only what's missing):
    - `chrono::NaiveDate` — already imported at line 19.
    - `chrono::NaiveDateTime` — needed for the new test 1; add to the
      `use chrono::NaiveDate;` line: `use chrono::{NaiveDate, NaiveDateTime};`.
    - `rust_decimal_macros::dec` — already imported.

    Run the tests once and CONFIRM they FAIL with the expected RED signal:
    - Test 1: assertion fails because the current service passes
      `account_update.balance_updated_at = Some(<client past time>)`
      through unchanged when current_balance is unchanged.
    - Test 2: assertions fail because the current service does not clear
      CC fields on type transition; the captured AccountUpdate has all
      original None values BUT the existing CC fields would have leaked
      via the repository's UPDATE statement (the test asserts at the
      service-layer boundary, where today's behavior is to pass the
      None values through — which means the client decides whether to
      clear them; the fix MAKES the server enforce the clear).
    - Important check: Test 2's assertion-set may already pass today
      since the client sends None and the mock returns None. Re-read
      the assertion logic carefully:
        * The update payload supplies `credit_limit: None`.
        * The mock repo's `update` impl captures the AccountUpdate
          verbatim.
        * `captured.credit_limit.is_none()` is therefore TRUE today
          (the client supplied None).
      To make this test meaningful, INVERT the precondition: the test
      must drive a payload where the client supplies the CC fields as
      Some(...) — simulating a buggy client that "forgot" to clear them
      when changing type — and assert that the server FORCES them to
      None. Adjust `update_with_type_change` accordingly:

      ```rust
      fn update_with_type_change_keeping_cc_values(id: &str, new_type: &str) -> AccountUpdate {
          AccountUpdate {
              id: Some(id.to_string()),
              name: "Card".to_string(),
              account_type: new_type.to_string(),
              // BUGGY CLIENT: leaves CC fields populated even though the
              // type is no longer CREDIT_CARD. The server MUST override.
              credit_limit: Some(dec!(5000)),
              statement_cycle_day: Some(15),
              statement_balance: Some(dec!(123.45)),
              minimum_payment: Some(dec!(25)),
              statement_due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
              reward_points_balance: Some(1000),
              cashback_balance: Some(dec!(12.34)),
              // ... other fields default ...
          }
      }
      ```

      AND validate() will REJECT this payload today (because non-CC type
      with CC fields is_some). To bypass validate() and exercise only the
      service's auto-clear logic, the simpler route is:

      a) Have the test's update payload submit None for all CC fields
         (which passes validate today).
      b) Update the assertion semantics: the test asserts that the
         AccountUpdate forwarded to the repository has all 7 CC fields
         set to None AND the test's purpose is to PIN this contract so
         a future refactor that re-orders the field handling cannot
         silently regress (preventing the AsChangeset skip-on-None
         data-leak surface).

      In other words, today's behavior accidentally passes the assertion
      (because client sent None and mock captured None). The test is
      still RED in spirit because the service does NOT actively guarantee
      the None-on-transition contract — but to make this RED-then-GREEN
      cycle observable to the test runner, structure the test as
      follows:

      Test 2 (revised, RED-observable):
      * Arrange: existing CC with all 7 CC fields populated.
      * Act: update_account with account_type "CHECKING" and ALL 7 CC
        fields set to Some(<original value>) — i.e., the client thinks
        the type didn't change OR has stale form state.
      * Today's behavior: validate() rejects with "Credit card fields
        are only valid for CREDIT_CARD accounts" → service returns Err.
      * Therefore, today this test FAILS at `await.unwrap()` (panic on
        Err).
      * Task 2 fix: update_account must FIRST detect the type transition
        and CLEAR (force to None) the 7 CC fields on the AccountUpdate
        BEFORE calling validate (or, equivalently, before forwarding to
        the repository). After the fix, validate() sees all 7 CC fields
        as None on a non-CC type → passes → repository receives None
        for each → Diesel writes NULL.
      * Assert (after fix): service.update_account returns Ok, AND the
        captured AccountUpdate has all 7 CC fields as None.

      This is the RED-observable signal. Pin it in code:

      ```rust
      #[tokio::test]
      async fn test_update_clears_cc_fields_on_type_transition_out_of_cc() {
          let (service, repo) = make_service(existing_cc_full());
          // Client sends a "stale" update: type changed to CHECKING but
          // CC fields still populated. Today's service rejects this via
          // validate(); the fix should sanitize the update FIRST.
          let update = update_with_type_change_keeping_cc_values("acc-1", "CHECKING");
          let result = service.update_account(update).await;
          assert!(
              result.is_ok(),
              "D-06: service must sanitize CC fields when type transitions \
               out of CREDIT_CARD before validation, not reject the update. \
               Got {:?}", result.err()
          );

          let captured = repo.last_update.lock().unwrap().clone().unwrap();
          assert!(captured.credit_limit.is_none(),         "D-06: credit_limit must be None after type transition");
          assert!(captured.statement_cycle_day.is_none(),  "D-06: statement_cycle_day must be None after type transition");
          assert!(captured.statement_balance.is_none(),    "D-06: statement_balance must be None after type transition");
          assert!(captured.minimum_payment.is_none(),      "D-06: minimum_payment must be None after type transition");
          assert!(captured.statement_due_date.is_none(),   "D-06: statement_due_date must be None after type transition");
          assert!(captured.reward_points_balance.is_none(),"D-06: reward_points_balance must be None after type transition");
          assert!(captured.cashback_balance.is_none(),     "D-06: cashback_balance must be None after type transition");
      }
      ```

      Today: `result.is_ok()` is FALSE → test fails with explicit message.
      After Task 2 fix: `result.is_ok()` is TRUE → assertions on captured
      fields pass.

    Run `cargo test -p whaleit-core accounts::accounts_service_tests::tests`
    and CONFIRM both new tests fail. Existing 3 tests must continue to
    pass. Commit as
    `test(03-10): add D-06 + D-12 invariant tests for update_account`.

    Reference existing code: lines 282-340 of the same file for builder
    patterns; line 87-97 of accounts_service.rs for the existing auto-bump
    block (Task 2's fix lives adjacent to it). Gap reasons: H-02 + H-03
    from 03-REVIEW.md.

  </action>

  <verify>
    <automated>
      cargo test -p whaleit-core accounts::accounts_service_tests::tests 2>&1 | grep -E "(FAILED|test result|test_update_(ignores|clears))"
    </automated>
  </verify>

<acceptance_criteria> - [ ] `crates/core/src/accounts/accounts_service_tests.rs`
contains both new test functions. Verify:
`grep -c "fn test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged" crates/core/src/accounts/accounts_service_tests.rs`
== 1 AND
`grep -c "fn test_update_clears_cc_fields_on_type_transition_out_of_cc" crates/core/src/accounts/accounts_service_tests.rs`
== 1. - [ ] `cargo test -p whaleit-core accounts::accounts_service_tests::tests`
reports BOTH new tests as FAILED. Existing 3 tests
(`test_update_bumps_balance_timestamp`,
`test_update_no_bump_when_balance_unchanged`,
`test_update_no_bump_when_no_balance_in_update`) MUST still pass. Capture in
commit body: `3 passed; 2 failed (expected RED for H-02/H-03)`. - [ ] No changes
to any file other than `accounts_service_tests.rs`. Verify: `git diff --stat`
shows exactly 1 file changed. - [ ] No new `#[allow(dead_code)]`, `#[ignore]`,
or `#[should_panic]` attributes added (these would mask the RED state). - [ ]
Commit message:
`test(03-10): add D-06 + D-12 invariant tests for update_account` — body cites
H-02 + H-03 from 03-REVIEW.md and includes the test runner output snippet.
</acceptance_criteria>

  <done>
    Two new failing tests pin the D-06 (CC-field clearing on type
    transition) and D-12 (server-only balance_updated_at) contracts. The
    3 pre-existing tests still pass. RED commit landed; ready for GREEN
    in Task 2.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Make update_account enforce D-06 + D-12 invariants and drop client-writable balance_updated_at from inbound DTOs (GREEN)</name>

<read_first> - crates/core/src/accounts/accounts_service.rs (lines 79-126 —
current update_account; the fix lives between line 87 (after `existing` is
fetched) and line 99 (before `repository.update`)) -
crates/core/src/accounts/accounts_constants.rs (account_types::CREDIT_CARD
constant) - crates/core/src/accounts/accounts_model.rs (lines 200-269 —
AccountUpdate field shape; do NOT modify validate()) - apps/server/src/models.rs
(lines 87-122 NewAccount DTO + From impl, 168-232 AccountUpdate DTO + From impl)
— the H-03 surface - crates/core/src/accounts/accounts_service_tests.rs — the
failing tests from Task 1 (do NOT modify; the fix must turn them GREEN)
</read_first>

  <files>
    apps/server/src/models.rs
    crates/core/src/accounts/accounts_service.rs
  </files>

  <action>

    **Part A — H-03 fix in `apps/server/src/models.rs` (drop client-writable balance_updated_at):**

    Two parallel edits, one for NewAccount and one for AccountUpdate. The
    field exists on the DTO struct so the OpenAPI schema reflects it as a
    response field — but on the From<DTO> for core::DTO impl, the value
    is DISCARDED. (Removing the field outright is also acceptable but
    risks breaking any client that reads it back on response — the safer
    surgical fix is to keep the field on the struct but force-discard on
    inbound conversion.)

    **Recommended approach (force-discard on inbound):**

    1. Locate `impl From<NewAccount> for core_accounts::NewAccount` (lines
       136-166). Change the `balance_updated_at: a.balance_updated_at,`
       line to `balance_updated_at: None, // D-12: server-only field,
       client value discarded`.

    2. Locate `impl From<AccountUpdate> for core_accounts::AccountUpdate`
       (lines 203-232). Change the `balance_updated_at: a.balance_updated_at,`
       line to `balance_updated_at: None, // D-12: server-only field,
       client value discarded`.

    Do NOT remove the field from the DTO structs themselves. Removing
    from `pub struct NewAccount` (line 110) or `pub struct AccountUpdate`
    (line 189) would change the OpenAPI/serde shape and break any
    response-side consumer that reads the field. The `From` impl is the
    correct seam for sanitizing inbound values without altering the
    surface.

    Per CLAUDE.md "Surgical Changes": this is a 2-line edit in 2 places.
    Do not refactor the surrounding code, do not rename fields, do not
    add helper functions.

    **Part B — H-02 fix in `crates/core/src/accounts/accounts_service.rs`:**

    Insert the type-transition CC-clearing logic BEFORE the existing
    auto-bump block. After `existing = self.repository.get_by_id(...)`
    (line 87) and BEFORE the `let mut account_update = account_update;`
    line (line 92), add:

    ```rust
    use crate::accounts::accounts_constants::account_types;

    // ... (existing get_by_id call at line 87) ...

    let mut account_update = account_update;

    // D-06: when account type transitions out of CREDIT_CARD, sanitize
    // the update payload to NULL all CC-only columns. Diesel's default
    // AsChangeset skips columns when the Option is None, so we must
    // ALSO set them to None here to ensure they get included in the
    // UPDATE statement. The repository's update path is responsible for
    // honoring None-as-NULL; this service-layer sanitation guarantees
    // the AccountUpdate has the correct shape regardless of what the
    // client supplied.
    let type_transition_out_of_cc =
        existing.account_type == account_types::CREDIT_CARD
            && account_update.account_type != account_types::CREDIT_CARD;
    if type_transition_out_of_cc {
        account_update.credit_limit = None;
        account_update.statement_cycle_day = None;
        account_update.statement_balance = None;
        account_update.minimum_payment = None;
        account_update.statement_due_date = None;
        account_update.reward_points_balance = None;
        account_update.cashback_balance = None;
    }

    // D-12: auto-stamp balance_updated_at when current_balance changes.
    // The client never gets to set this field — server is the source of
    // truth for "when was the balance last touched". Defense-in-depth:
    // the inbound DTO already sets this to None per H-03 fix in
    // apps/server/src/models.rs, but core code is the contract for ALL
    // callers (Tauri IPC, future MCP, tests), so we re-assert here.
    if account_update.current_balance.is_some()
        && account_update.current_balance != existing.current_balance
    {
        account_update.balance_updated_at =
            Some(chrono::Utc::now().naive_utc());
    } else {
        // Belt and suspenders for D-12: discard any inbound value that
        // bypassed the DTO sanitation (e.g., a Tauri caller passing the
        // core type directly). Server is the SOLE writer of this field.
        account_update.balance_updated_at = None;
    }

    let result = self.repository.update(account_update).await?;
    ```

    Note on the `else` branch — Test 1 from Task 1 specifically asserts
    that a client-supplied `balance_updated_at` with no balance change is
    DISCARDED. Without the `else`, today's code passes the client value
    through (the H-03 hole). With the `else`, the server unconditionally
    rejects any non-server-stamp.

    Validate after the fix:
    1. Run `cargo test -p whaleit-core accounts::accounts_service_tests::tests`
       — expect ALL 5 tests to pass (3 existing + 2 new from Task 1).
    2. Run `cargo test -p whaleit-core accounts::` — full Phase 3 test
       suite (16 existing) should still pass.
    3. Run `cargo check -p whaleit-server` — clean.
    4. Run `cargo check -p whaleit-storage-postgres --tests` — clean.
    5. Run `cargo clippy -p whaleit-core -- -D warnings` (or repo's
       configured clippy command) — no new lints introduced.

    Reference existing code: REVIEW.md H-02 §"Suggested fix" Part 1 + Part
    2 (specifies the type-transition + service-or-repo enforcement
    point); REVIEW.md H-03 §"Suggested fix" (specifies the From-impl
    discard pattern).

    Gap reasons: H-02 (D-06 invariant on type transitions) + H-03 (D-12
    server-only timestamp).

  </action>

  <verify>
    <automated>
      cargo test -p whaleit-core accounts::accounts_service_tests::tests 2>&1 | tail -10 &&
      cargo check -p whaleit-server 2>&1 | tail -5 &&
      cargo check -p whaleit-storage-postgres --tests 2>&1 | tail -5
    </automated>
  </verify>

<acceptance_criteria> - [ ] `apps/server/src/models.rs` From<NewAccount> impl
has the literal `balance_updated_at: None,` (not `a.balance_updated_at,`).
Verify: `grep -nE "balance_updated_at" apps/server/src/models.rs | wc -l`
reports the same count as before, but the From-impl lines are `None`. More
precise:
`grep -A1 "impl From<NewAccount> for core_accounts::NewAccount" apps/server/src/models.rs | grep -A30 "fn from" | grep "balance_updated_at:" | grep -c "None"`
→ 1. - [ ] `apps/server/src/models.rs` From<AccountUpdate> impl has the literal
`balance_updated_at: None,`. Verify: equivalent grep for the AccountUpdate impl
→ 1. - [ ] `apps/server/src/models.rs` did NOT remove `balance_updated_at` from
the NewAccount or AccountUpdate struct declarations. Verify:
`grep -c "pub balance_updated_at: Option<NaiveDateTime>" apps/server/src/models.rs`
≥ 2 (one for NewAccount, one for AccountUpdate). - [ ]
`crates/core/src/accounts/accounts_service.rs` contains the literal string
`type_transition_out_of_cc` AND the literal string `account_types::CREDIT_CARD`.
Verify:
`grep -c "type_transition_out_of_cc" crates/core/src/accounts/accounts_service.rs`
== 1. - [ ] `crates/core/src/accounts/accounts_service.rs` always discards a
client-supplied `balance_updated_at` when balance is unchanged. Verify:
`grep -A2 "else {" crates/core/src/accounts/accounts_service.rs | grep -c "balance_updated_at = None"`
≥ 1. - [ ] `cargo test -p whaleit-core accounts::accounts_service_tests::tests`
reports `5 passed; 0 failed`. The 2 RED tests from Task 1 are now GREEN; the 3
pre-existing tests still pass. - [ ] `cargo test -p whaleit-core accounts::`
reports `16 passed` (or more — adding tests is fine; subtracting is not). - [ ]
`cargo check -p whaleit-server` exits 0 (clean compile). - [ ]
`cargo check -p whaleit-storage-postgres --tests` exits 0. - [ ] No changes to
`accounts_model.rs` validate() — the fix lives in the service layer, not the
validator. Verify: `git diff crates/core/src/accounts/accounts_model.rs` is
empty. - [ ] No changes to `repository.rs` — Diesel AsChangeset's None-skip
semantics are preserved as-is; the service guarantees None-on-transition before
calling repository.update. Verify:
`git diff crates/storage-postgres/src/accounts/repository.rs` is empty. - [ ]
Commit message:
`fix(03-10): enforce D-06 + D-12 invariants on update_account (H-02, H-03)` —
body cites both gap IDs and the test transition RED → GREEN.
</acceptance_criteria>

  <done>
    update_account NULLs all 7 CC fields when account_type transitions out
    of CREDIT_CARD (D-06 invariant restored). Client-supplied
    balance_updated_at is discarded both at the DTO seam (H-03 fix in
    server/models.rs) and in the core service (defense-in-depth for
    non-HTTP callers). The 2 H-02/H-03 regression tests from Task 1 turn
    GREEN. All 16 Phase 3 core tests still pass. cargo check on server +
    storage-postgres clean.
  </done>
</task>

</tasks>

<threat_model>

## Trust Boundaries

| Boundary                  | Description                                                                                                   |
| ------------------------- | ------------------------------------------------------------------------------------------------------------- |
| HTTP client → Axum server | Untrusted JSON (the source of the H-03 attack: client-supplied `balanceUpdatedAt`)                            |
| Server DTO → Core domain  | The From impl is the sanitization seam (H-03 fix lives here)                                                  |
| Core service → Repository | The service is the sole writer for the D-06 / D-12 invariants the repository must honor (H-02 fix lives here) |

## STRIDE Threat Register (carry-forward + this plan's mitigations)

| Threat ID | Category               | Component                                                                                          | Disposition | Mitigation Plan                                                                                                                                                                                                                                       |
| --------- | ---------------------- | -------------------------------------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-3-02    | Tampering              | `apps/server/src/models.rs::AccountUpdate.balance_updated_at`                                      | mitigate    | H-03 fix: From-impl discards client value (`balance_updated_at: None`); core service belt-and-suspenders else-branch always discards on no-balance-change. Pinned by `test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged`. |
| T-3-02    | Tampering              | `crates/core/src/accounts/accounts_service.rs::update_account` (CC-field leak via type transition) | mitigate    | H-02 fix: service-layer detects CREDIT_CARD → non-CC transition and force-NULLs the 7 CC columns on the AccountUpdate before forwarding to the repository. Pinned by `test_update_clears_cc_fields_on_type_transition_out_of_cc`.                     |
| T-3-04    | Information Disclosure | Stale CC fields on a CHECKING row                                                                  | mitigate    | Same H-02 fix: clearing CC columns on type transition prevents downstream consumers from rendering stale credit_limit / statement_balance / etc. on a non-CC account.                                                                                 |

</threat_model>

<verification>
After both tasks land, the gap closure is verified by:

1. **Full Phase 3 core test suite green:**
   `cargo test -p whaleit-core accounts::` → 16 passed (or more if Task 1 added
   cleanly; final tally should be 18 passed = 16 + 2 new).

2. **Server + storage-postgres compile clean:** `cargo check -p whaleit-server`
   → exit 0. `cargo check -p whaleit-storage-postgres --tests` → exit 0.

3. **Grep audits:**
   - `grep -c "type_transition_out_of_cc" crates/core/src/accounts/accounts_service.rs`
     == 1.
   - In apps/server/src/models.rs: the `From<NewAccount>` and
     `From<AccountUpdate>` impls each have a `balance_updated_at: None,` line
     (NOT `a.balance_updated_at,`). Verify by reading the diff.

4. **No collateral damage:**
   - `git diff crates/core/src/accounts/accounts_model.rs` → empty.
   - `git diff crates/storage-postgres/src/accounts/repository.rs` → empty.
   - `git diff --stat HEAD~2` (this plan's 2 commits) shows exactly 4 files
     changed: models.rs, accounts_service.rs, accounts_service_tests.rs (Task
     1's RED), and the same accounts_service.rs (Task 2's GREEN). The first
     commit touches only the test file; the second touches the 2 production
     files.

5. **D-06 + D-12 invariants restored:**
   - D-06: type transition out of CREDIT_CARD → all 7 CC fields written as NULL.
   - D-12: server is sole writer of balance_updated_at; client value
     unconditionally discarded. </verification>

<human_verification> The following items from 03-VERIFICATION.md
`human_verification` section remain pending after completion of this plan AND
Plan 03-09:

1. **PG integration tests with DATABASE_URL.**
   `cargo test -p whaleit-storage-postgres accounts` against a real PG instance.
   This plan's H-02 fix produces correct AccountUpdate values at the
   service-layer boundary; the round-trip assertion that PG ends up with NULL
   columns for the 7 CC fields requires DATABASE_URL.

   Recommended addition for the next CI cycle: extend
   `crates/storage-postgres/src/accounts/repository_tests.rs` with a
   `test_update_clears_cc_columns_on_type_change_pg` integration test that
   exercises the full UPDATE statement. Out of scope for this gap plan because
   the failing test already exists at the service layer and is the more direct
   contract.

2. **E2E spec on a clean host.** Same as 03-09's open item — port 8088 conflict
   on the verifier host. Recipe in 03-08-SUMMARY.md.

3. **Manual smoke test:** edit a CC account, change type to CHECKING, submit,
   then verify in the DB (or via account detail page) that credit_limit /
   statement_cycle_day / etc. are now NULL. This is the user-visible smoke
   equivalent of the service-level test. </human_verification>

<success_criteria>

- D-06 invariant restored: a CREDIT_CARD account changed to non-CC has all 7 CC
  columns written as NULL by the service-layer fix. Verified via Task 1's
  regression test transitioning RED → GREEN in Task 2.
- D-12 invariant restored: server is sole writer of balance_updated_at.
  Client-supplied values are discarded at both the DTO seam (HTTP) and the
  service-layer else-branch (defense in depth for Tauri / future MCP callers).
- 16 → 18 passing Phase 3 core tests (2 new tests added).
- cargo check on whaleit-server and whaleit-storage-postgres clean.
- No changes to accounts_model.rs or repository.rs (the fix lives at the correct
  seam — the service).
- Two atomic commits in git log (TDD RED + GREEN). </success_criteria>

<output>
After completion, create
`.planning/phases/03-bank-accounts-credit-cards/03-10-SUMMARY.md` following
the standard summary template. Include:

- TDD cycle commits (test commit + fix commit)
- Test count delta (16 → 18 in whaleit-core accounts:: test scope)
- The exact code snippet for the type-transition block and the else-branch
  balance_updated_at discard
- Which gaps (H-02, H-03) were closed and the invariants restored (D-06, D-12)
- Confirmation that the PG-level integration assertion (item 1 in
  human_verification above) remains routed to the next CI cycle with a
  recommended new repository_tests.rs case </output>
