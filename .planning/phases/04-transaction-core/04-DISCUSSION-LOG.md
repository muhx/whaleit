# Phase 4: Transaction Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution
> agents. Decisions are captured in `04-CONTEXT.md` — this log preserves the
> alternatives considered.

**Date:** 2026-04-30 **Phase:** 04-transaction-core **Areas discussed:**
Transfer modeling, Duplicate detection, Categorization rules, CSV import
templates

---

## Transfer Modeling

### Q1 — How should a transfer between two of the user's own accounts be stored?

| Option                                       | Description                                                                                                                                                                                                                                      | Selected |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------- |
| Paired rows (Recommended)                    | One outflow on source + one inflow on destination, linked by transfer_group_id. Each row has a single account_id. Matches how banks export CSVs and how YNAB / Monarch model it. Per-account balance math stays simple. Deletion has to cascade. | ✓        |
| Single row with from/to fields               | One transaction with from_account_id + to_account_id. One user action = one DB row. Trade-off: every per-account ledger query must union from/to; CSV import (which sees two rows) must merge on import; splits + FX get awkward.                |          |
| Two unlinked transactions, manually pairable | User records each side separately; optional later step links them. Maximum flexibility but no automatic reconciliation; transfers don't get distinct visual treatment without the link.                                                          |          |

**User's choice:** Paired rows **Notes:** Aligns with prior art and CSV export
reality. Decision becomes D-01.

### Q2 — Cross-currency FX handling

| Option                                                                                 | Description                                                                                                                                                                                        | Selected |
| -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Each leg stores its own native amount; FX rate captured at transfer time (Recommended) | Source: -$100 USD. Destination: +€92 EUR. Both rows share transfer_group_id and a snapshotted rate (1.0870) on each side. Historic accuracy preserved. Reuses crates/core/src/fx/.                 | ✓        |
| Single amount + conversion at display time                                             | Store amount once on source; destination side stores a 'mirror' marker. Display layer converts using current FX. Smaller schema but historic transfers shift value retroactively when rates move.  |          |
| User enters both legs explicitly, no auto FX                                           | User types both USD and EUR amounts; system stores exactly what the user typed. Most accurate for real bank wires (with fees), but user has to do mental math when statements don't show the rate. |          |

**User's choice:** Each leg stores its own native amount; FX rate captured at
transfer time **Notes:** Decision becomes D-02. Researcher confirms
`crates/core/src/fx/` exposes a snapshot helper.

### Q3 — Display behavior in /transactions ledger

| Option                                                                       | Description                                                                                                                                                                                                                                          | Selected |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Both sides visible; "Show transfers" toggle hides both at once (Recommended) | Both legs render as Transfer-direction rows in the ledger. Each is paired via transfer_group_id (subtle indicator). Tapping either opens detail sheet showing both sides. UI-SPEC's "Show transfers" toggle hides both legs simultaneously when off. | ✓        |
| Collapse paired rows into one logical row                                    | Display layer renders the pair as a single row 'Transfer: Chase → Amex Savings $100'. Avoids visual duplication. Trade-off: per-account view must show only one side, so rendering rule diverges by view; complicates running balance display.       |          |
| Both sides visible; treat them as independent rows for filter purposes       | Pair link exists for reconciliation only; UI doesn't visually couple them. Simplest. Trade-off: deleting one side without the other could orphan the link; "Show transfers" toggle would have to know about the link anyway.                         |          |

**User's choice:** Both sides visible; "Show transfers" toggle hides both at
once **Notes:** Decision becomes D-03. Per-account view shows only the leg
attached to that account_id (D-05).

### Q4 — Edit semantics on a paired transfer

| Option                                                  | Description                                                                                                                                                                                                                                                                                                                | Selected |
| ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Sync both legs automatically (Recommended)              | Editing date, transfer_group_id, or notes propagates to the sibling. Editing the amount asks: "Apply to both legs (preserves transfer pairing) or only this leg (breaks the link)?". Keeps pairings honest by default; lets user record real-world wire fees / rate-spread asymmetries by intentionally breaking the link. | ✓        |
| Edit each leg independently; warn if amounts diverge    | Both legs independently editable. Subtle warning chip when source.amount ≠ destination.amount × fx_rate. More flexibility, more chances to get out of sync silently.                                                                                                                                                       |          |
| Open a single "transfer editor" that edits both at once | Tapping edit on either leg opens a transfer-specific editor showing source + destination side-by-side with one save button. Cleanest UX but blocks intentional asymmetry (wire fees on one side only).                                                                                                                     |          |

**User's choice:** Sync both legs automatically **Notes:** Decision becomes
D-04. AlertDialog copy for the amount-edit prompt is Claude's discretion.

---

## Duplicate Detection

### Q1 — Match signals

| Option                                                            | Description                                                                                                                                                                                                                                                                  | Selected |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Account + date window + amount, payee as tiebreaker (Recommended) | Required: same account_id, same amount (within $0.01 epsilon for FX rounding), date within tolerance. Payee similarity (Levenshtein / token-set ratio on normalized merchant strings) bumps confidence. Notes ignored. YNAB / Monarch model. Aligns with UI-SPEC §6 buckets. | ✓        |
| Account + amount + payee fuzzy; date is loose                     | Match key is amount + payee, date as tiebreaker only. Catches duplicates re-imported with shifted dates. Trade-off: legitimate same-merchant repeat charges (Spotify monthly) need a strict date-window guard or false-positive.                                             |          |
| External-id first, signals as fallback                            | If imported row carries a stable bank-side ID (FITID in OFX, transaction_id in some CSVs), exact-match. Otherwise fall back to signal-based. Best precision when present, but most CSVs don't have a stable ID.                                                              |          |

**User's choice:** Account + date window + amount, payee as tiebreaker
**Notes:** Decision becomes D-06 (match keys) and D-07 (payee as multiplier, not
gate). External-id fallback (FITID for OFX) covered by D-19 + idempotency
Claude's discretion.

### Q2 — Date tolerance window

| Option                  | Description                                                                                                                                                                                                               | Selected |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| ±3 days (Recommended)   | Catches the common bank pattern where transaction date in one export differs from re-export. Spotify monthly charges are 28-31 days apart, so ±3 won't false-positive recurring billing. Practical sweet spot.            | ✓        |
| Same day only (±0 days) | Only flags duplicates with the same calendar date. Highest precision; cleanest user trust. Misses the common case where CSV re-export shifts the date by 1-2 days.                                                        |          |
| ±7 days                 | Catches more edge cases (delayed authorizations, weekend posting drift). Trade-off: starts flagging legitimate same-merchant weekly purchases (Saturday grocery runs) as possible duplicates. Higher false-positive rate. |          |

**User's choice:** ±3 days **Notes:** Decision becomes D-06 window.

### Q3 — When does detection run?

| Option                                       | Description                                                                                                                                                                                                                   | Selected |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Import only (Recommended)                    | Detector runs as part of CSV/OFX import. UI-SPEC §6 already designs for this — duplicate-review wizard step + post-import banner on /transactions. Manual entry skips. Cheapest, clearest scope.                              | ✓        |
| Import + manual entry both                   | Manual-entry form shows inline warning when about to save a transaction matching an existing one. UX cost: every fast manual entry pays a dedupe-query latency hit; users adding legitimate repeats see a warning every time. |          |
| Import only, but background-scan after edits | Detector runs at import + low-priority background sweep when transactions edited. Trade-off: complicates data flow; introduces async "new duplicates found" notifications.                                                    |          |

**User's choice:** Import only **Notes:** Decision becomes D-08. Per-pair "not a
duplicate" memory and background-scan deferred (D-10).

---

## Categorization Rules

### Q1 — Rule shape for Phase 4

| Option                                                                   | Description                                                                                                                                                                                                                                                                         | Selected |
| ------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Implicit memory only — remember last category per merchant (Recommended) | When user assigns Category X to merchant Y, system stores (normalized_merchant, account_id) → category_id. Future transactions auto-fill (still editable). On CSV import, learned mappings pre-fill the category column on Review. No new settings page. Builds value over time.    | ✓        |
| Explicit rule manager (new Settings page)                                | Adds Settings → Categorization rules page where user sees and edits patterns: 'Payee matches "AMZN" → Shopping', 'Amount > $1000 on Chase → Housing'. More power, more schema (new rules table + UI). Phase 4 cost climbs noticeably; Phase 8 (AI) will likely re-skin this anyway. |          |
| Both — implicit memory + a hidden "manage rules" Settings page           | Implicit learning by default; Settings page surfaces what was learned and lets the user delete/override. Compromise: less new UI than option B (it just lists what option A is already storing), but still requires a settings screen and edit affordances.                         |          |

**User's choice:** Implicit memory only **Notes:** Decision becomes D-11.
Amount-pattern half of TXN-02 explicitly deferred to Phase 8.

### Q2 — Merchant string normalization

| Option                                                                                       | Description                                                                                                                                                                                                                                                    | Selected |
| -------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Lowercase + strip leading/trailing whitespace + collapse runs of digits/spaces (Recommended) | 'WHOLEFDS GRP #10403' and 'WHOLEFDS GRP #10477' both normalize to 'wholefds grp #' — same memory. Captures common bank-formatting drift. No NLP. Implementable in pure Rust without dependencies.                                                              | ✓        |
| Lowercase only — exact match on the rest                                                     | Conservative: only matches identical merchant strings (case-insensitive). Cleaner semantics; lower hit rate. User has to teach the system per-store-number.                                                                                                    |          |
| Token-level fuzzy matching (Levenshtein / Jaccard)                                           | Approximate matching using string similarity. Maximum hit rate. Trade-off: false positives ('Whole Foods' vs 'Wholesale Foods'); harder to debug; more compute on every form open. Probably more than Phase 4 needs given AI categorization is Phase 8 anyway. |          |

**User's choice:** Lowercase + strip + collapse runs of digits/spaces **Notes:**
Decision becomes D-13.

### Q3 — Edit semantics on category change

| Option                                                      | Description                                                                                                                                                                                                                                           | Selected |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Yes — last write wins, silently update memory (Recommended) | Editing a transaction's category overwrites the (merchant → category) memory immediately. Subsequent new transactions pre-fill with the new choice. No prompts, no "apply to historical?" dialog. Historical transactions are NOT bulk-recategorized. | ✓        |
| Yes, but ask whether to apply retroactively                 | After category change, show: 'Apply this to past transactions for Whole Foods? (47 transactions)'. Useful when first cleaning up imports; nagging during normal edits. Adds confirm-dialog scope.                                                     |          |
| No — manual edits never update the auto-fill memory         | Memory only learned from initial entry, not from edits. User can override per transaction without 'training' the system. Trade-off: if first guess was wrong, user has to keep correcting it forever; implicit-learning value evaporates.             |          |

**User's choice:** Yes — last write wins, silently update memory **Notes:**
Decision becomes D-14.

---

## CSV Import Templates

### Q1 — Template strategy

| Option                                                    | Description                                                                                                                                                                                                                                                                   | Selected |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| User-saved only — same as activities import (Recommended) | User maps columns once for a given bank, names + saves the mapping. Subsequent imports of the same bank's CSV pre-pick that template. No starter pack. Mirrors existing activity import flow exactly. Lowest delivery cost, lowest maintenance burden.                        | ✓        |
| Starter pack of 5-10 popular banks/issuers                | Ship pre-built templates for Chase, Wells Fargo, Bank of America, Amex, Capital One, Discover, Citi, USAA, Apple Card. Adds first-import polish. Trade-off: bundled mappings drift when banks tweak exports; ongoing maintenance owed; non-US banks excluded from the polish. |          |
| Starter pack + community-contributable templates          | Starter pack + 'community templates' picker pulling from JSON in repo (or remote source). Trade-off: introduces remote-fetch path with privacy implications. Way over Phase 4 scope.                                                                                          |          |

**User's choice:** User-saved only **Notes:** Decision becomes D-16.

### Q2 — Header drift on saved templates

| Option                                                                 | Description                                                                                                                                                                                                                                                     | Selected |
| ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Auto-detect mismatch on Mapping step + prompt re-mapping (Recommended) | When previously-saved template selected, validate file's headers still match. On mismatch: 'Your saved Chase CSV template doesn't match this file's columns. Re-map?'. Existing 'mapping-step-unified' already detects header issues; this extends the pattern. | ✓        |
| Silently apply template; user catches errors at Review step            | Template applies blindly; Review step shows preview rows where wrong column mappings would be visible. Cheaper to implement. Trade-off: easy to miss subtle mismatches (two amount-like columns); user might commit a corrupt import.                           |          |
| Force users to confirm/re-save the template every import               | Treats each import as fresh mapping; saved templates are starting suggestions only. Most defensive. Trade-off: defeats the purpose of saved templates; user pays the mapping cost every import. Unnecessary.                                                    |          |

**User's choice:** Auto-detect mismatch + prompt re-mapping **Notes:** Decision
becomes D-17.

### Q3 — Per-account vs global scope

| Option                                                              | Description                                                                                                                                                                                                                                               | Selected |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Shared across accounts; user picks from a global list (Recommended) | User saves a template named 'Chase Checking CSV'; appears in picker on any account import. Same Chase format works for both Chase Checking and Chase Savings exports. Mirrors how activity-import-page.tsx scopes templates today.                        | ✓        |
| Per-account scoping                                                 | Templates tied to the account they were created for. Imports on a different account start fresh even if the bank is the same. Trade-off: more friction for users with multiple accounts at the same bank; doubles the mapping work for no schema benefit. |          |
| Per-account by default, with "copy to other account" affordance     | Templates default to per-account, with a button to clone to another account. Compromise. Trade-off: adds UI surface for the clone affordance; users still pay friction the first time.                                                                    |          |

**User's choice:** Shared across accounts; global list **Notes:** Decision
becomes D-18. OFX template handling clarified separately (D-19 — OFX skips
templates entirely).

---

## Claude's Discretion

The user explicitly punted these to downstream agents (researcher / planner):

- Exact `transactions` table schema (columns, indexes, foreign keys,
  partial-index strategy)
- Splits storage shape: separate `transaction_splits` table vs splits JSONB
  column
- Running balance computation strategy (window function vs materialized column
  vs snapshot table)
- OFX parsing crate selection (`ofx-rs`, `sgmlish`, or hand-roll)
- Confidence-score formula details under the bucket constraint (D-09)
- Storage shape for `(normalized_merchant, account_id) → category_id` memory
  (D-12)
- Per-row idempotency key for re-imports (CSV row hash vs OFX FITID)
- Audit metadata for "category was learned vs user-typed"
- Visual treatment of paired-transfer rows beyond the icon (D-03 candidates)
- Bottom-nav placement of the "Transactions" entry
- AlertDialog copy for the amount-edit-prompt on transfer pairs (D-04)
- Auto-fill timing detail (client-side cached vs per-request) for category
  pre-fill (D-15)

## Deferred Ideas

Captured during discussion for future phases:

- AI-powered category / payee suggestion UI → Phase 8
- AI-powered conversational transaction entry → Phase 8
- Receipt OCR upload + extraction UI → Phase 8
- Recurring / subscription detection UI → Phase 7
- Budget assignment UI inside transaction form → Phase 5
- Spending-by-category charts on /transactions → Phase 6
- Income vs expense bars → Phase 6
- Net worth ribbon at top of /transactions → Phase 6
- Per-transaction tag editor → Phase 12 (freelancer mode)
- Bulk multi-row edit UI → deferred
- Swipe-row gestures → deferred (UI-SPEC explicit non-goal)
- Date-group collapse/expand → deferred (running-balance scroll math)
- Duplicate-detection rule editor → deferred
- Reconciliation widget → deferred (data foundation only in Phase 4)
- Settings → Categorization-rules manager UI → Phase 8
- TXN-02 amount-pattern rules → Phase 8
- Per-pair "not a duplicate" memory → deferred
- Background-scan duplicate detection on edits → deferred
- Starter-pack CSV templates → deferred (D-16)
- Community-contributable templates → deferred (D-16)
- Token-level fuzzy merchant matching → Phase 8
- CSV transfer auto-pair detection across two account imports → deferred
- Apply-category-to-historical prompt on edit → deferred (D-14)
