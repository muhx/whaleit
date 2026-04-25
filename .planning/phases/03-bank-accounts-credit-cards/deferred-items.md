# Phase 3 — Deferred Items

Out-of-scope discoveries logged during plan execution. Do not address in this
phase unless explicitly scoped in.

## Pre-existing Test Failures (not caused by Phase 3 changes)

- **`portfolio::snapshot::holdings_calculator_tests::tests::test_multi_currency_same_asset_buy_activities`**
  - Discovered during: 03-02 Task 3 verification (`cargo test -p whaleit-core`)
  - Failure:
    `panicked at .../holdings_calculator_tests.rs:2491:10: Cannot start a runtime from within a runtime.`
  - Cause: tokio runtime conflict (calling `block_on` inside an async context).
    Unrelated to Account / NewAccount / AccountUpdate domain changes — touches
    `AccountStateSnapshot`, not `Account` struct.
  - Verification: failure mode is independent of Phase 3 deltas; the test builds
    and runs against unchanged `holdings_calculator` code paths.
  - Action: out of scope for Phase 3. Log here so Phase 6 (snapshot/portfolio)
    or a follow-up infra task can address.
