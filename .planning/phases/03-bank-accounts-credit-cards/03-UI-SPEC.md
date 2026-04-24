---
phase: 3
slug: bank-accounts-credit-cards
status: draft
shadcn_initialized: true
preset:
  existing (packages/ui — baseColor slate, cssVariables, third-party registries
  @animate-ui + @diceui)
created: 2026-04-24
---

# Phase 3 — UI Design Contract

> Visual and interaction contract for the Bank Accounts & Credit Cards phase.
> Produced by gsd-ui-researcher. Validated by gsd-ui-checker. Consumed by
> gsd-planner and gsd-executor.
>
> Scope: `/accounts` unified list, Account create/edit flow (CHECKING / SAVINGS
> / CREDIT_CARD / LOAN), Account detail extensions for credit cards, "Update
> balance" action, archive toggle, empty + error states, responsive behavior.
> Out of scope: transactions (Phase 4), statement history (deferred), rewards
> rules (deferred), bank sync (out of scope per PROJECT.md).

---

## Source of Truth

| Source                                                         | What It Locks                                                                                                           |
| -------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `.planning/phases/03-bank-accounts-credit-cards/03-CONTEXT.md` | Phase 3 domain + implementation decisions (D-01..D-19)                                                                  |
| `.planning/phases/01-codebase-health-rebrand/01-CONTEXT.md`    | Brand: friendly companion whale, soft-illustration style, "Your friendly finance companion" tagline                     |
| `apps/frontend/src/globals.css` (`@theme` + `:root`)           | Declared color palette (WhaleIt ocean + flexoki semantic tokens), font stack, radius/input/button sizing, chart palette |
| `packages/ui/components.json`                                  | shadcn preset + registered third-party registries                                                                       |
| `apps/frontend/src/pages/dashboard/accounts-summary.tsx`       | Existing row shape, grouping UX, skeleton, hover interaction — reuse; do not redesign                                   |
| `apps/frontend/src/pages/account/account-page.tsx`             | Existing per-account detail page scaffolding — extend with CC sections                                                  |
| `apps/frontend/src/lib/constants.ts`                           | `AccountType` + `defaultGroupForAccountType` — extend, do not duplicate                                                 |

---

## Design System

| Property             | Value                                                                                                                                                                                                                                                            |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tool                 | shadcn (initialized at `packages/ui`)                                                                                                                                                                                                                            |
| Preset               | baseColor `slate`, style `default`, `cssVariables: true`, `tsx: true`, `rsc: false`                                                                                                                                                                              |
| Component library    | Radix primitives via shadcn, plus `@whaleit/ui` shared components (`Button`, `Card`, `Dialog`, `Sheet`, `Form`, `Input`, `Select`, `ResponsiveSelect`, `Switch`, `Progress`, `Popover`, `Separator`, `Tooltip`, `Skeleton`, `Tabs`, `Table`, `EmptyPlaceholder`) |
| Financial components | `@whaleit/ui` → `PrivacyAmount`, `GainAmount`, `GainPercent`, `AmountDisplay`, `MoneyInput`, `CurrencyInput`                                                                                                                                                     |
| Icon library         | `lucide-react` (via `@whaleit/ui` `Icons` map). Phase 3 uses: `Wallet`, `Landmark` (fallback to `Building`), `CreditCard`, `Coins`/`HandCoins`, `RefreshCw`, `Plus`, `AlertCircle`, `ShieldCheck`, `FileArchive`, `Sparkles`                                     |
| Font                 | Sans: Inter Variable (default body/UI). Mono: IBM Plex Mono (amounts via `PrivacyAmount` / `MoneyInput`). Serif reserved for marketing/empty-state hero headings only.                                                                                           |
| Theme                | Light + dark via `.dark` class. Both palettes already declared — no new tokens needed for Phase 3.                                                                                                                                                               |
| Registry safety      | `@animate-ui` and `@diceui` registered but Phase 3 MUST NOT pull any new blocks from them. Any block needed is already in `packages/ui`.                                                                                                                         |

---

## Spacing Scale

All spacing uses Tailwind's default 4px-based scale. No custom spacing tokens
introduced this phase. Declared values used in Phase 3 screens:

| Token | Value                                    | Usage                                                                             |
| ----- | ---------------------------------------- | --------------------------------------------------------------------------------- |
| xs    | 4px (`gap-1`)                            | Icon-to-text gap in inline chips, separator to adjacent text                      |
| sm    | 8px (`gap-2`, `space-y-2`)               | Row-internal vertical rhythm, secondary metric gap, compact chip padding          |
| md    | 12px (`gap-3`, `space-y-3`, `px-3 py-3`) | Mobile card interior padding, stacked form field gap on mobile                    |
| lg    | 16px (`gap-4`, `p-4`, `space-y-4`)       | Desktop card interior padding, form row gap, section-to-section gap inside a card |
| xl    | 24px (`gap-6`, `p-6`)                    | Between major sections of `/accounts`, between CC detail subsections              |
| 2xl   | 32px (`p-8`)                             | Empty-state padding, modal content padding when sparse                            |
| 3xl   | 48px                                     | Page top/bottom outer padding on desktop only                                     |

Row density (matches existing `accounts-summary.tsx`):

- Desktop: `px-5 py-4` (20px horizontal, 16px vertical)
- Mobile: `px-4 py-3` (16px horizontal, 12px vertical)

Touch targets:

- Interactive rows: min height 56px (satisfies 44px iOS minimum)
- Primary/secondary buttons: `--button-height` = 2.5rem (40px) desktop, 2.75rem
  (44px) lg/mobile
- Inputs: `--input-height` = 2.75rem (44px) — already set for mobile parity

Exceptions: none. The existing token stack (`--button-height`, `--input-height`,
`--card-padding`, `--sheet-padding`) is reused as-is.

---

## Typography

Values tie to existing `globals.css` and `@whaleit/ui` patterns. Do not
introduce new font sizes this phase. Exactly **two font weights** are used
across Phase 3: `400` (regular body/label) and `600` (semibold heading/display).
Label and chip emphasis is achieved via color (`text-muted-foreground`), size
(12px), and letter-spacing — never via a distinct weight.

| Role               | Size                             | Weight            | Line Height            | Where Used                                                                                                                                                           |
| ------------------ | -------------------------------- | ----------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Body               | 14px (`text-sm`)                 | 400               | 1.5                    | Default body on desktop (`--font-size-base: 0.875rem`), row secondary text ("institution · currency"), form hints                                                    |
| Body (mobile)      | 16px (`text-base`)               | 400               | 1.5                    | Inputs and primary row text on mobile to avoid iOS zoom                                                                                                              |
| Label / Chip       | 12px (`text-xs`)                 | 400               | 1.4                    | "Available credit" chip, "Archived" tag, balance-updated-at caption, metric captions — emphasis via `text-muted-foreground` + `tracking-wide`, NOT a distinct weight |
| Heading (row name) | 14-16px (`text-sm md:text-base`) | 600               | 1.25 (`leading-tight`) | Account name in list row, group name header                                                                                                                          |
| Section title      | 16px (`text-md`)                 | 600               | 1.25                   | Section titles inside detail page ("Statement", "Rewards"), "Accounts" dashboard header                                                                              |
| Display (balance)  | 18-24px (`text-lg md:text-2xl`)  | 600, tabular-nums | 1.2                    | Large balance on CC detail hero, account detail hero balance                                                                                                         |

Family rules:

- All UI chrome and amounts use `--font-sans` (Inter Variable).
- Monetary values rendered through `PrivacyAmount` / `AmountDisplay` already
  apply `tabular-nums` — do not override.
- Mono (`IBM Plex Mono`) only appears inside `MoneyInput` / `CurrencyInput`
  while editing — existing component behavior, do not change.
- Merriweather serif is not used in Phase 3.

Weight discipline:

- Only `400` and `600` are permitted in Phase 3 components. Do not introduce
  `font-medium` (500) or any other weight. If emphasis is needed on a small
  label, use `text-foreground` (darker) or `tracking-wide` rather than bumping
  weight.

---

## Color

Phase 3 uses existing semantic tokens (`--background`, `--card`, `--muted`,
`--primary`, `--destructive`, `--success`, `--warning`, `--border`, `--ring`).
No new colors are introduced. The 60/30/10 split below maps the app's light
theme onto Phase 3 surfaces; dark theme inherits from the same tokens.

| Role            | Token                                                                                | Usage                                                                                  |
| --------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| Dominant (60%)  | `bg-background` (`hsl(165 20% 97%)` light / `hsl(200 29% 8%)` dark)                  | Page background, form field backgrounds (`--input-bg`)                                 |
| Secondary (30%) | `bg-card` (`hsl(168 16% 94%)` light / `hsl(195 20% 12%)` dark) + `border-border`     | Account list row surfaces, detail cards, modal/sheet surface                           |
| Accent (10%)    | `bg-primary` / `text-primary` (flexoki-tx — near-black on light, near-white on dark) | Reserved — see list below                                                              |
| Destructive     | `text-destructive` / `bg-destructive/10` (`hsl(3 62% 42%)` light)                    | Archive confirmation emphasis, CC utilization > 90% warning stripe, inline form errors |
| Success         | `text-success` / `bg-success/10` (`hsl(73 84% 27%)`)                                 | "Balance updated just now" confirmation toast, CC utilization 0-29% meter fill         |
| Warning         | `text-warning` / `bg-warning/10` (`hsl(45 99% 34%)`)                                 | CC utilization 30-69% meter fill, statement-due-within-7-days badge                    |

Accent (`--primary`) is reserved for and ONLY for:

1. The single primary CTA per screen: "New account" on `/accounts`, "Save" on
   account form, "Update balance" in the balance modal.
2. The active/selected state in the account-type radio-group of the new-account
   form (`data-state=on` on the selected `ToggleGroupItem`).
3. The currently-selected group-by control on `/accounts` header (matches
   existing `accounts-summary.tsx` grouping toggle convention).

Accent MUST NOT be used on: group headers, archive toggle, chips, amounts,
institution text, or card borders. Those use neutral tokens
(`text-muted-foreground`, `border-border`).

Credit-card utilization meter color ramp (applied to `<Progress>` fill only;
background stays `--muted`):

| Utilization | Fill Token                       |
| ----------- | -------------------------------- |
| 0% – 29%    | `--success`                      |
| 30% – 69%   | `--warning`                      |
| 70% – 89%   | `--warning` + solid (not tinted) |
| 90% – 100%+ | `--destructive`                  |

Dark mode parity: all of the above resolve automatically through the `.dark` HSL
overrides already in `globals.css`. No per-component dark override is required.

Chart colors: not used in Phase 3 (no chart work planned). Phase 6 will
introduce liability-vs-asset charts for net worth.

---

## Screen Inventory & Layouts

### 1. `/accounts` — Unified Account List (NEW route)

Renders all account types in one scrollable list, grouped by `account.group`.
Keeps the grouping look-and-feel of the dashboard `accounts-summary.tsx`.

Layout (desktop, `≥ md`):

```
┌──────────────────────────────────────────────────────────────┐
│ PageHeader: "Accounts"        [Group/List toggle] [+ New]    │  56px
├──────────────────────────────────────────────────────────────┤
│ Filter bar (sticky, 48px): [ Show archived  ⚪ off ]         │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ▼ Banking · 3 accounts · $12,480.10                         │  Group header (collapsible)
│     ├ Chase Checking      USD        $4,200.00               │  Row
│     ├ HSBC Savings        USD    €3,150.50 · ~$3,405.00      │  Row (FX shown)
│     └ Ally Savings        USD        $4,875.10               │
│                                                              │
│  ▼ Credit Cards · 2 accounts · –$1,820.00                    │
│     ├ Amex Gold      USD    $420.50 · [Available $4,579.50]  │  CC row w/ chip
│     └ Chase Sapphire USD  $1,400.00 · [Available $8,600.00]  │
│                                                              │
│  ▼ Investments · (unchanged)                                 │
│  ▼ Cash · (unchanged)                                        │
│                                                              │
│  — — — — — — — — — — — — — — — — — — — — — — — — — — — — —  │
│  Archived (3)                       [Hide archived]          │  Only when toggle on
│     ├ Old Chase Card (archived)   USD    $0.00               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

Rules:

- Reuse `accounts-summary.tsx` row component shape exactly (name + sub-text
  left, amount stack right, chevron right). Credit-card rows append an
  `"Available credit"` chip in the secondary-metric slot, replacing the
  gain/loss line that's shown for investment rows.
- Group order (top-to-bottom): Banking → Credit Cards → Loans → Investments →
  Cash → Crypto → Uncategorized. Archived group (if visible) is always last and
  visually separated (`<Separator />` + muted heading).
- Group totals: display group-sum in **base currency**. If all accounts in the
  group share one currency, also show native currency in sub-text; otherwise
  omit native.
- Row currency display: always show `current_balance` in the account's native
  currency (primary amount). If native ≠ base, show the base-currency equivalent
  in the sub-text prefix (`~$3,405.00 base` style, matching `GainAmount` tone).
- Liability accounts (`CREDIT_CARD`, `LOAN`) render their amount with a leading
  minus sign and `text-destructive/80` ONLY in the group-total row. Individual
  CC rows keep positive display (per D-13) but the sub-text reads `Owed` instead
  of the currency code when native ≠ base.
- Empty-group collapse: groups with 0 accounts are not rendered.

Responsive (`< md`):

- Header collapses: title stays, "Group/List toggle" and "+ New" move into an
  overflow `Popover` menu triggered by `MoreHorizontal` icon, except the "+ New"
  which is promoted to a floating action button pinned bottom-right with a 56px
  `rounded-full` surface using `--primary`. The FAB respects `pb-safe` inset and
  the mobile nav height (`--mobile-nav-ui-height`).
- Row padding: `px-4 py-3` (matches existing).

Loading state: 4 skeleton rows via existing `AccountSummarySkeleton`.

Error state: reuse existing `AccountsSummary` destructive card (see
`accounts-summary.tsx` lines 383-400).

### 2. "New account" flow — single dynamic form

Opens via `+ New` button → `Sheet` on desktop (right-side, `sheet-padding`),
`Dialog` full-screen on mobile. Uses `@whaleit/ui` `Form` wrapping
react-hook-form + zod. Sections appear in this order; CC/LOAN-specific sections
only render when the selected type matches.

Form sections:

1. **Account type** — `ToggleGroup` with 4 options rendered as icon + label
   cards (2×2 grid on mobile, 4-across on desktop):
   - Checking · `Icons.Wallet`
   - Savings · `Icons.Coins`
   - Credit Card · `Icons.CreditCard`
   - Loan · `Icons.Building` Selected option gets `data-state=on` styling
     (accent border + accent text).
2. **Basics** — always visible once a type is chosen:
   - Name (required, `Input`, placeholder "e.g. Chase Freedom Unlimited")
   - Institution (required, free-text `Input`, placeholder "e.g. Chase, HSBC,
     SoFi"). NO autocomplete. Hint below: "Type the bank or issuer name."
   - Currency (required, `Select`, pre-filled with user's base currency)
   - Group (optional, `Input` with the auto-filled default from
     `defaultGroupForAccountType`; user may override). Hint: "Leave blank to use
     '<defaultGroup>'."
3. **Opening balance** — for all 4 types:
   - Amount (`MoneyInput`, currency tied to the selected Currency above).
   - Helper: "Today's balance becomes your starting point. Phase-4 transactions
     will use this as the opening entry."
   - For CC: allow `≥ 0`; for bank/LOAN: allow `≥ 0`. Zero is valid.
4. **Credit card details** — visible ONLY if type = `CREDIT_CARD`:
   - Credit limit (`MoneyInput`, required, `> 0`)
   - Statement cycle day (`Select` of 1–31, required; helper: "Day the statement
     closes each month.")
   - Statement balance (`MoneyInput`, optional)
   - Minimum payment (`MoneyInput`, optional)
   - Statement due date (`DatePicker`, optional)
   - Reward points balance (`Input` numeric, optional)
   - Cashback balance (`MoneyInput`, optional) A subtle note above the section:
     "You can edit these anytime as each statement closes."
5. **Loan note** — visible ONLY if type = `LOAN`:
   - Single note card, no fields, copy: "Loans in v1 track name, institution,
     currency, and balance only. Amortization and interest tracking are coming
     later."

Footer actions:

- Left: `Cancel` (ghost)
- Right: primary CTA label = "Create account" (accent)
- Submission: single `create_account` mutation; on success, close sheet, toast
  "Account created", and navigate to `/accounts/:id`.

Validation messages (all inline under the field, `text-xs text-destructive`):

- "Name is required."
- "Institution is required."
- "Opening balance must be 0 or greater."
- "Credit limit must be greater than 0."
- "Statement cycle day must be between 1 and 31."
- "Statement balance cannot exceed the credit limit." (soft warning, not
  blocking — since reconciliation may legitimately show over-limit)

### 3. Account detail page — CC extensions

Extend existing `apps/frontend/src/pages/account/account-page.tsx`. For bank
accounts (CHECKING/SAVINGS) and LOAN, render existing hero + balance section but
hide investment-only modules (`AccountHoldings`, `AccountMetrics`,
`AccountContributionLimit`, `HistoryChart` activity markers). For `CREDIT_CARD`,
ADD the following sections beneath the hero:

Section A — **"Credit overview"** card:

```
┌────────────────────────────────────────────────┐
│ Credit overview                                │
│                                                │
│  Balance                    Available credit   │
│  $1,400.00                  $8,600.00          │
│                                                │
│  Utilization  14%                              │
│  [██░░░░░░░░░░░░░░░░]  ← --success fill        │
│                                                │
│  Limit  $10,000.00       [ Update balance ]    │
└────────────────────────────────────────────────┘
```

- "Balance" uses `PrivacyAmount`. "Available credit" uses `PrivacyAmount` with
  `text-success` when `> 20%` of limit remains, `text-warning` when 5–20%,
  `text-destructive` when < 5%.
- "Utilization" label + inline percent + `<Progress value={utilization} />`
  using color ramp from the Color section.
- "Update balance" button opens the Update Balance modal (Section 5 below).

Section B — **"Statement snapshot"** card:

```
┌────────────────────────────────────────────────┐
│ Statement snapshot       [Edit]                │
│                                                │
│  Statement balance      Minimum payment        │
│  $1,400.00              $40.00                 │
│                                                │
│  Due date                                      │
│  May 15, 2026  ·  in 12 days                   │
│                                                │
│  Last updated: Apr 24, 2026 by you             │
└────────────────────────────────────────────────┘
```

- Days-to-due uses `date-fns.formatDistanceToNow`.
- Due-in ≤ 7 days renders the date chip with `bg-warning/10 text-warning` and a
  `ShieldCheck` icon prefix.
- `[Edit]` opens the CC fields section of the edit form (reuses the new-account
  form in edit mode).
- If all statement fields are null, show compact empty placeholder: "No
  statement recorded yet. [Record statement]" where the link opens the edit form
  focused on statement balance.

Section C — **"Rewards"** card (only if `reward_points_balance` or
`cashback_balance` is non-null OR the type is CC — always render in CC detail
with empty-state copy when both are null):

```
┌────────────────────────────────────────────────┐
│ Rewards                  [Edit]                │
│                                                │
│  Points balance         Cashback balance       │
│  12,450 pts             $34.80                 │
└────────────────────────────────────────────────┘
```

- Empty state inside card: "No rewards balance tracked. Add your points or
  cashback to keep an eye on them." + `[Add rewards]` secondary button.

Section ordering on CC detail:

```
Hero (balance + account name + institution + currency)
  ↓
Credit overview
  ↓
Statement snapshot
  ↓
Rewards
  ↓
(Phase 4+ will add: Recent transactions)
```

### 4. Bank / LOAN detail page

Same hero. Below hero:

- Single **"Balance"** card: current_balance, balance_updated_at, and
  `[Update balance]` button. No investment modules.
- Archive/edit moved to existing Page header action palette (unchanged).

### 5. "Update balance" modal

Triggered from: CC "Credit overview" card, bank/LOAN "Balance" card, and the
account edit action palette.

Modal (`Dialog` desktop / `Sheet` mobile bottom sheet):

```
┌────────────────────────────────────────────────┐
│ Update balance                           [×]   │
├────────────────────────────────────────────────┤
│ Current balance  $1,400.00  as of Apr 22       │
│                                                │
│ New balance                                    │
│ ┌────────────────────────────────────┐         │
│ │ USD  1,400.00                      │  ← MoneyInput, autofocus
│ └────────────────────────────────────┘         │
│                                                │
│ Note  (optional)                               │
│ ┌────────────────────────────────────┐         │
│ │                                    │  ← Textarea
│ └────────────────────────────────────┘         │
│                                                │
│  Phase 4 will reconcile this with transactions │  ← small info caption
│  once imported. For now this is a manual       │
│  snapshot.                                     │
│                                                │
│                     [ Cancel ]  [ Save balance ]│
└────────────────────────────────────────────────┘
```

Behavior:

- Submitting writes `current_balance` + `balance_updated_at = now()`.
- Success toast: `"Balance updated just now"` (uses `--toast-success-*`).
- If unchanged, "Save balance" is disabled.
- Note field is stored in future-phase memo table; Phase 3 UI captures it but
  the field may be ignored at the service layer (planner to decide). If the
  service discards it, hide the Note field — do not expose dead UI. Planner
  decision gate.

### 6. Archive / unarchive interaction

- Archive action lives inside the existing account edit action palette (reuse).
  Label: "Archive account". Confirmation `AlertDialog`:
  - Title: "Archive this account?"
  - Body: "Archived accounts stay safe — we keep every balance and history. You
    can unarchive anytime from the 'Show archived' toggle on Accounts."
  - Primary button: `"Archive"` destructive variant.
  - Secondary: `"Keep active"`.
- Unarchive action appears ONLY when the currently-open account is archived.
  Label: "Unarchive account". No confirmation dialog (matches D-19 "no
  un-archive lock").
- List-level "Show archived" toggle: `Switch` component in the `/accounts`
  filter bar. Persisted in settings context as `showArchivedAccounts` (planner
  decides storage). Default: off.
- Archived rows, when shown, render with `opacity-60` + a
  `<span className="text-xs text-muted-foreground border rounded px-1.5 py-0.5">Archived</span>`
  chip inline after the name.

### 7. Selectors (account-selector + mobile variant)

- Both selectors filter out `is_archived=true` accounts by default. No UI change
  visible to the user — behavior change only.
- Group headers inside the selector use the same pre-seeded group names as
  `/accounts`.

---

## Copywriting Contract

Exact copy. Executor uses verbatim.

### Page titles

| Screen                         | Title          | Subtitle                                      |
| ------------------------------ | -------------- | --------------------------------------------- |
| `/accounts`                    | "Accounts"     | (none — page hero only)                       |
| `/accounts/new` (sheet/dialog) | "New account"  | "Track a bank account, credit card, or loan." |
| `/accounts/:id` (CC)           | _account name_ | _institution_ · _currency_                    |

### Primary CTAs

| Location                    | Label               |
| --------------------------- | ------------------- |
| `/accounts` header          | "New account"       |
| New-account form footer     | "Create account"    |
| Account edit form footer    | "Save changes"      |
| Update balance modal footer | "Save balance"      |
| Archive confirmation        | "Archive"           |
| Unarchive (no confirm)      | "Unarchive account" |
| CC "Credit overview" inline | "Update balance"    |
| Statement empty state       | "Record statement"  |
| Rewards empty state         | "Add rewards"       |

Never abbreviate. Never use "+" alone. The `+` icon always pairs with the text
label on desktop; mobile FAB uses icon-only with `aria-label="New account"`.

### Empty states

| Scenario                            | Heading                         | Body                                                                                       | CTA                |
| ----------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------ | ------------------ |
| `/accounts` — no accounts at all    | "Start with your first account" | "Add a bank account, credit card, or loan to see everything in one place."                 | "New account"      |
| `/accounts` — all accounts archived | "Nothing active right now"      | "Every account is archived. Flip 'Show archived' to see them, or add a new account."       | "New account"      |
| Show-archived on, none archived     | "No archived accounts"          | "You haven't archived anything yet. Archived accounts show up here when you do."           | (no CTA)           |
| CC statement snapshot missing       | "No statement recorded yet"     | "When your next statement closes, record the balance, minimum payment, and due date here." | "Record statement" |
| CC rewards unset                    | "No rewards balance tracked"    | "Add your points or cashback to keep an eye on them."                                      | "Add rewards"      |
| Bank/LOAN balance never updated     | "No balance recorded"           | "Record your current balance so we can include this account in your totals."               | "Update balance"   |

### Error states

| Scenario                                      | Copy                                                                                                                                                 |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/accounts` load failure                      | Heading: "We couldn't load your accounts." Body: "Try refreshing. If this keeps happening, restart the app or check that the database is reachable." |
| Create account failure (generic)              | Toast: "Couldn't create that account. Your changes are still here — try again."                                                                      |
| Create account failure (validation at server) | Inline field error using server message; fallback: "Something in this form needs another look."                                                      |
| Update balance failure                        | Toast: "Balance didn't save. Check your connection and try again." Modal stays open with values intact.                                              |
| Archive failure                               | Toast: "Couldn't archive this account just now. Try again in a moment."                                                                              |

### Destructive confirmations

| Action  | Dialog title            | Body                                                                                                                                      | Confirm label |
| ------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| Archive | "Archive this account?" | "Archived accounts stay safe — we keep every balance and history. You can unarchive anytime from the 'Show archived' toggle on Accounts." | "Archive"     |

No hard-delete in Phase 3. Only archive.

### Tone check

- Friendly, present-tense, plural-second-person ("we keep", "you can").
- Match the Phase-1 voice: "Your friendly finance companion." No finance jargon
  where plain words work ("Balance" not "Outstanding amount"; "Due date" not
  "Payment deadline").
- Never scold or guilt-trip on high utilization. The color ramp conveys
  severity; copy stays neutral. Avoid "danger", "warning" in user-facing strings
  unless the state is genuinely actionable.
- Emoji: NONE in UI strings.

### Accessibility labels

- Mobile FAB: `aria-label="New account"`.
- Group-by toggle: `aria-label="Group view"` / `aria-label="List view"` (matches
  existing `accounts-summary.tsx`).
- Show-archived switch: labeled text "Show archived" plus `aria-describedby`
  pointing to "Reveal accounts you've set aside".
- Utilization progress bar: `aria-valuenow={utilization}`, `aria-valuemin={0}`,
  `aria-valuemax={100}`, `aria-label="Credit utilization"`.

---

## Interaction Contracts

| Interaction         | Contract                                                                                             |
| ------------------- | ---------------------------------------------------------------------------------------------------- |
| Row hover (desktop) | `transition-shadow duration-150 hover:shadow-md` (matches existing)                                  |
| Row tap (mobile)    | Active state: `bg-card/80`; no separate hover layer                                                  |
| Group header click  | Toggles expand/collapse. `Icons.ChevronDown` rotates 180° via `transition-transform duration-200`    |
| Form field focus    | shadcn default `ring-2 ring-ring ring-offset-2`                                                      |
| Button press        | shadcn default; no custom animation                                                                  |
| Dialog/Sheet open   | Default shadcn enter/exit (fade + slide)                                                             |
| Toast               | Uses existing Sonner with `--toast-success-*` variables. 4s auto-dismiss, swipe-to-dismiss on mobile |
| Modal autofocus     | First enabled input in the form                                                                      |
| Keyboard escape     | Closes Dialog/Sheet without save                                                                     |
| Archive confirm     | AlertDialog defaults; `Enter` confirms destructive action only with focus on destructive button      |
| Update balance      | Cmd/Ctrl+Enter submits from within MoneyInput                                                        |
| Privacy mode        | All `PrivacyAmount` usages respect the existing privacy toggle — no per-screen override              |

---

## Component Inventory (Phase 3)

Components that MUST be used (from `@whaleit/ui`):

- `Page`, `PageHeader`, `PageContent` — page scaffolding (same as
  `account-page.tsx`).
- `Button` — default, `variant="ghost"` for icon-only, `variant="destructive"`
  for archive confirm.
- `Card`, `CardHeader`, `CardTitle`, `CardContent` — detail-page sections.
- `Sheet` / `Dialog` — new-account form (Sheet desktop, Dialog mobile
  full-screen).
- `AlertDialog` — archive confirmation only.
- `Form`, `FormField`, `FormItem`, `FormLabel`, `FormControl`, `FormMessage` —
  all form composition.
- `Input`, `Textarea`, `Select`, `ResponsiveSelect`, `DatePickerInput`,
  `ToggleGroup`, `ToggleGroupItem`, `Switch` — field primitives.
- `MoneyInput`, `CurrencyInput` — all money entry.
- `PrivacyAmount`, `AmountDisplay`, `GainAmount` — all money display.
- `Progress` — CC utilization meter.
- `Separator` — row-to-row dividers inside groups, and before Archived section.
- `Skeleton` — loading states (reuse `AccountSummarySkeleton`).
- `EmptyPlaceholder` — empty states listed above.
- `Tooltip`, `TooltipTrigger`, `TooltipContent` — utilization % tooltip with
  exact ratio; "Balance updated X ago" tooltip on caption.
- `Icons` (lucide map): `Wallet`, `Coins`, `CreditCard`, `Building`,
  `RefreshCw`, `Plus`, `AlertCircle`, `ShieldCheck`, `FileArchive`,
  `ChevronDown`, `ChevronRight`, `MoreHorizontal`.

Components that MUST NOT be introduced this phase:

- Any new block from `@animate-ui` or `@diceui` third-party registries. No
  safety gate run → not allowed to land.
- Any new shadcn primitive not already in `packages/ui/src/components/ui/`.
- Any new chart (Phase 6 scope).
- Any new icon library.

---

## Registry Safety

| Registry        | Source                                          | Blocks Used in Phase 3                                                                | Safety Gate                   |
| --------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------- | ----------------------------- |
| shadcn official | `packages/ui` (baseColor slate, local vendored) | All primitives listed in Component Inventory. No new install this phase — reuse only. | not required (local vendored) |
| `@animate-ui`   | `https://animate-ui.com/r/{name}.json`          | none                                                                                  | n/a — no blocks added         |
| `@diceui`       | `https://diceui.com/r/{name}.json`              | none                                                                                  | n/a — no blocks added         |

Phase 3 is **net-zero** on third-party registry consumption. If the planner or
executor proposes pulling a new block from `@animate-ui` or `@diceui`, they MUST
(a) stop, (b) return to `gsd-ui-researcher` for a vetting gate
(`npx shadcn view {block} --registry {url}`), and (c) re-open this UI-SPEC for
amendment.

---

## Responsive Breakpoints

Mirror existing app (`tailwind` defaults + mobile-first):

| Breakpoint    | Width      | Notable Layout Shift                                                                                                                |
| ------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| base (mobile) | `< 640px`  | Single column, FAB for "+ New", bottom-sheet modals, account detail sections stack full-width                                       |
| `sm`          | `≥ 640px`  | Form fields side-by-side where two related (e.g., "Statement balance" + "Minimum payment")                                          |
| `md`          | `≥ 768px`  | `/accounts` shows inline "+ New" button + toggle in header; row padding bumps to `px-5 py-4`; CC detail overview uses 2-column grid |
| `lg`          | `≥ 1024px` | Detail page uses the existing 2-column layout (hero + side panel) defined in `account-page.tsx`                                     |

Mobile nav coexistence: the `/accounts` page sits inside the existing
`app-shell` with `data-page-scroll-container`. Phase 3 does not add to the
bottom nav; `/accounts` is reached from the existing navigation entry that was
previously labeled "Holdings" or "Settings › Accounts" — planner to decide
redirection strategy. UI contract only requires that the route exists and is
reachable from both desktop sidebar and mobile bottom nav.

---

## Numerical Precision & Formatting

| Value                      | Format                                                                                                                                 |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Balances, limits, payments | 2 decimal places via `PrivacyAmount` / `AmountDisplay`. Respect settings `baseCurrency`.                                               |
| Utilization                | Integer percent on row (e.g. "14%"); 1 decimal in Tooltip (e.g. "14.2%"). Never round up past 100 — "100%" covers both 100.0 and >100. |
| Points balance             | Locale-formatted integer, suffix `"pts"`.                                                                                              |
| Cashback                   | Same as balances (MoneyInput / PrivacyAmount).                                                                                         |
| Dates                      | `MMM d, yyyy` (e.g. "Apr 24, 2026"). Relative suffix for due dates ≤ 30 days: `·  in 12 days` / `·  today` / `·  2 days ago`.          |
| Balance updated timestamp  | Absolute + tooltip relative: `"Apr 24, 2026"` inline, `"2 hours ago"` in tooltip.                                                      |

---

## Non-Goals (Explicit)

The following are deliberately excluded from Phase 3 UI-SPEC, even though they
may seem natural:

- Transaction lists on account detail (Phase 4).
- Charts of balance-over-time (Phase 6; Phase 3 reuses the existing
  `HistoryChart` on investment-type detail pages only).
- Statement history table UI (deferred).
- Rewards earning rules editor (deferred).
- Institution logo / autocomplete (deferred per D-18).
- Dedicated `/accounts/archived` route (deferred).
- Un-archive confirmation (D-19 explicit).
- Reconciliation widget (Phase 4+).
- Balance alerts / notifications (Phase 7+).

---

## Pre-Population Source Map

For checker/auditor traceability.

| Field                       | Source                                                                  |
| --------------------------- | ----------------------------------------------------------------------- |
| Design System               | `packages/ui/components.json`                                           |
| Spacing                     | Tailwind default + existing `accounts-summary.tsx`                      |
| Typography                  | `apps/frontend/src/globals.css` + existing `account-page.tsx`           |
| Color palette               | `apps/frontend/src/globals.css` `@theme` + `:root`                      |
| Accent reservation          | Derived from Phase 1 brand decisions (D-08..D-11) + 60/30/10 discipline |
| Row shape                   | `apps/frontend/src/pages/dashboard/accounts-summary.tsx`                |
| Group names                 | `03-CONTEXT.md` D-16                                                    |
| Archive UX                  | `03-CONTEXT.md` D-19                                                    |
| "Update balance" separation | `03-CONTEXT.md` specifics + D-12                                        |
| CC fields visible           | `03-CONTEXT.md` D-06..D-09                                              |
| Available-credit chip       | `03-CONTEXT.md` D-17 + specifics                                        |
| Copy tone                   | `01-CONTEXT.md` brand voice: "Your friendly finance companion"          |
| Registry list               | `packages/ui/components.json` `registries` block                        |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
