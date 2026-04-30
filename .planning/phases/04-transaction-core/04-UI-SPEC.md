---
phase: 4
slug: transaction-core
status: draft
shadcn_initialized: true
preset:
  existing (packages/ui — baseColor slate, cssVariables, third-party registries
  @animate-ui + @diceui)
created: 2026-04-30
---

# Phase 4 — UI Design Contract

> Visual and interaction contract for the Transaction Core phase. Produced by
> gsd-ui-researcher. Validated by gsd-ui-checker. Consumed by gsd-planner and
> gsd-executor.
>
> Scope: global `/transactions` ledger view, per-account "Recent transactions"
> abridged section, manual transaction add/edit form (with split-category
> editor), CSV/OFX import wizard (forked from existing activity import),
> duplicate-review flow, search/filter UI, multi-currency display + running
> balance, category model wired into the existing `taxonomies` system, empty
>
> - error states, responsive behavior. Out of scope: AI-powered categorization
>   rules editor (Phase 8 supplements TXN-02 — Phase 4 ships only rule-based
>   merchant matching, no AI fallback UI), recurring/subscription detection
>   (Phase 7), budget assignment UI (Phase 5), reports/charts (Phase 6), bank
>   API sync (out of scope per PROJECT.md).

---

## Source of Truth

| Source                                                                                                | What It Locks                                                                                                                  |
| ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `.planning/phases/03-bank-accounts-credit-cards/03-UI-SPEC.md`                                        | All global tokens (spacing, typography, color, weight discipline, accent reservation rules, copywriting tone, registry safety) |
| `.planning/phases/01-codebase-health-rebrand/01-CONTEXT.md`                                           | Brand voice: friendly companion whale, "Your friendly finance companion"                                                       |
| `apps/frontend/src/globals.css` (`@theme` + `:root`)                                                  | Declared color palette (WhaleIt ocean + flexoki semantic tokens), font stack, radius/input/button sizing, toast tokens         |
| `packages/ui/components.json`                                                                         | shadcn preset + registered third-party registries                                                                              |
| `apps/frontend/src/pages/activity/import/activity-import-page.tsx`                                    | Multi-step wizard scaffolding, `ImportProvider` context shape, step indicator UX — fork as the basis for the txn import wizard |
| `apps/frontend/src/pages/activity/import/steps/{upload,mapping-step-unified,review,confirm}-step.tsx` | Step-by-step UX patterns, `FileDropzone`, `MappingTable`, `CSVFileViewer`, template picker                                     |
| `apps/frontend/src/pages/dashboard/accounts-summary.tsx`                                              | Row shape, density, hover, skeleton — referenced for transaction-list row visuals                                              |
| `apps/frontend/src/pages/account/account-page.tsx`                                                    | Per-account detail page scaffolding — extend with "Recent transactions" section                                                |
| `apps/frontend/src/pages/settings/taxonomies/`                                                        | Existing taxonomy domain (`Taxonomy`, `TaxonomyCategory`, `isSystem`, hierarchy support)                                       |
| `apps/frontend/src/lib/types/taxonomy.ts`                                                             | TypeScript shape for taxonomy entries — see Category Model section below                                                       |

---

## Design System

| Property             | Value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tool                 | shadcn (initialized at `packages/ui`) — INHERITED FROM PHASE 3                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Preset               | baseColor `slate`, style `default`, `cssVariables: true`, `tsx: true`, `rsc: false` — INHERITED                                                                                                                                                                                                                                                                                                                                                                                         |
| Component library    | Radix primitives via shadcn, plus `@whaleit/ui` shared components. Phase 4 reuses Phase 3 inventory PLUS the import-wizard primitives already in `apps/frontend/src/pages/activity/import/components/` (`FileDropzone`, `MappingTable`, `CSVFileViewer`, `WizardStepIndicator`, `StepNavigation`, `TemplatePicker`, `HelpTooltip`, `CancelConfirmationDialog`)                                                                                                                          |
| Financial components | `@whaleit/ui` financial → `PrivacyAmount`, `GainAmount`, `AmountDisplay`, `MoneyInput`, `CurrencyInput`. Phase 4 adds NO new financial primitives — only compositions.                                                                                                                                                                                                                                                                                                                  |
| Icon library         | `lucide-react` via `@whaleit/ui` `Icons` map. Phase 4 uses (in addition to Phase 3 set): `ArrowDownLeft` (income), `ArrowUpRight` (expense), `ArrowLeftRight` (transfer), `Search`, `Filter`, `SlidersHorizontal`, `Upload`, `FileText`, `Tag`, `SplitSquareHorizontal` (split editor), `Copy` (duplicate review), `MapPin` (location placeholder for future), `CalendarDays`, `X` (clear filter chip), `Check` (commit split row), `Plus` (add split row), `Trash2` (remove split row) |
| Font                 | Sans: Inter Variable (default body/UI). Mono: IBM Plex Mono (amounts via `PrivacyAmount` / `MoneyInput`). NO new fonts.                                                                                                                                                                                                                                                                                                                                                                 |
| Theme                | Light + dark via `.dark` class — INHERITED. Both palettes already declared. No new tokens introduced.                                                                                                                                                                                                                                                                                                                                                                                   |
| Registry safety      | `@animate-ui` and `@diceui` registered but Phase 4 MUST NOT pull any new blocks from them. Any block needed is already vendored in `packages/ui` or already shipped in Phase 3.                                                                                                                                                                                                                                                                                                         |

---

## Spacing Scale

Inherits the Phase 3 token stack verbatim. No custom spacing tokens introduced
this phase. Declared values used in Phase 4 screens:

| Token | Value                                    | Usage                                                                                                         |
| ----- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| xs    | 4px (`gap-1`)                            | Icon-to-text gap in transaction-row direction icon, filter-chip internal padding, split-row remainder caption |
| sm    | 8px (`gap-2`, `space-y-2`)               | Filter-chip horizontal stack gap, secondary metadata gap (date · category), split-row vertical rhythm         |
| md    | 12px (`gap-3`, `space-y-3`, `px-3 py-3`) | Mobile transaction-row interior padding, stacked form-field gap on mobile, duplicate-pair card interior gap   |
| lg    | 16px (`gap-4`, `p-4`, `space-y-4`)       | Desktop row interior padding, form row gap, importer step content padding, duplicate-pair card outer padding  |
| xl    | 24px (`gap-6`, `p-6`)                    | Between major sections of `/transactions`, between import-wizard step content and step navigation             |
| 2xl   | 32px (`p-8`)                             | Empty-state padding, modal content padding when sparse                                                        |
| 3xl   | 48px                                     | Page top/bottom outer padding on desktop only                                                                 |

Row density (matches existing accounts/activity rows for consistency):

- Desktop transaction row: `px-5 py-3` (20px horizontal, 12px vertical —
  slightly tighter than account rows because transaction lists are denser)
- Mobile transaction row: `px-4 py-3`
- Group/date header inside ledger: `px-5 py-2 md:py-3` with `bg-muted/30`
  background

Touch targets:

- Interactive transaction rows: min height 56px on mobile (44px iOS minimum +
  extra tap area for swipe affordance)
- Filter-chip buttons: min height 32px (`h-8`) — inline-friendly density; meets
  WCAG 2.5.5 inline target relaxation since chips appear in groups
- Primary/secondary buttons: `--button-height` = 2.5rem (40px) desktop, 2.75rem
  (44px) lg/mobile — UNCHANGED
- Inputs: `--input-height` = 2.75rem (44px) — UNCHANGED

Exceptions: filter chips use `h-8` (32px) instead of the full 40px button
height, justified by the inline-cluster pattern (each chip has neighbors, so the
effective tap region exceeds 44px). This is the only deviation from Phase 3
spacing.

---

## Typography

Strictly inherits the Phase 3 contract: exactly **two font weights** (`400`,
`600`), exactly **four font sizes** (12 / 14 / 16 / 18-24px). No new sizes, no
new weights, no new families. Emphasis via color (`text-muted-foreground`) and
`tabular-nums`, never via `font-medium` (500).

| Role             | Size                             | Weight            | Line Height            | Phase 4 Usage                                                                                                                                                                                              |
| ---------------- | -------------------------------- | ----------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Body             | 14px (`text-sm`)                 | 400               | 1.5                    | Default body on desktop, transaction-row payee text, form hints, importer mapping-table cell text                                                                                                          |
| Body (mobile)    | 16px (`text-base`)               | 400               | 1.5                    | Inputs, primary row text on mobile                                                                                                                                                                         |
| Label / Chip     | 12px (`text-xs`)                 | 400               | 1.4                    | Filter-chip labels, date-group header ("Apr 24, 2026 · 4 transactions"), category chip on row, "split" badge, FX-converted sub-amount, balance caption ("Bal $4,200.00 after"), duplicate-confidence badge |
| Heading (row)    | 14-16px (`text-sm md:text-base`) | 600               | 1.25 (`leading-tight`) | Transaction payee/merchant name, importer step heading                                                                                                                                                     |
| Section title    | 16px (`text-md`)                 | 600               | 1.25                   | Importer step titles ("Upload your file", "Map your columns"), "Recent transactions" header on account detail, "Filter transactions" sheet title                                                           |
| Display (amount) | 18-24px (`text-lg md:text-2xl`)  | 600, tabular-nums | 1.2                    | Reserved for transaction-detail-sheet hero amount only. Row amounts use 14-16px body with `tabular-nums` (NOT display weight) to keep ledger calm.                                                         |

Family rules:

- All UI chrome and amounts use `--font-sans` (Inter Variable).
- Monetary values rendered through `PrivacyAmount` / `AmountDisplay` apply
  `tabular-nums` automatically — do not override.
- Mono (`IBM Plex Mono`) only appears inside `MoneyInput` / `CurrencyInput`
  during edit — do NOT use mono for read-only amounts.
- Merriweather serif is not used in Phase 4.

Weight discipline (re-asserted from Phase 3):

- Only `400` and `600` are permitted in Phase 4 components. The split-editor
  remainder indicator MUST NOT use `font-medium` — emphasize with
  `text-destructive` / `text-success` color tokens instead.

---

## Color

Phase 4 uses existing semantic tokens (`--background`, `--card`, `--muted`,
`--primary`, `--destructive`, `--success`, `--warning`, `--border`, `--ring`).
No new colors are introduced. The 60/30/10 split is identical to Phase 3.

| Role            | Token                                                               | Phase 4 Usage                                                                                                                                                                                |
| --------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Dominant (60%)  | `bg-background` (`hsl(165 20% 97%)` light / `hsl(200 29% 8%)` dark) | Page background, form-field backgrounds, ledger row default surface                                                                                                                          |
| Secondary (30%) | `bg-card` + `border-border`                                         | Date-group header strip (`bg-muted/30`), transaction-detail sheet, importer card surfaces, duplicate-pair card surface                                                                       |
| Accent (10%)    | `bg-primary` / `text-primary` (flexoki-tx)                          | Reserved — see explicit list below                                                                                                                                                           |
| Destructive     | `text-destructive` / `bg-destructive/10` (`hsl(3 62% 42%)` light)   | Inline form errors, "split sum mismatch" remainder text, "Delete transaction" confirm button, expense-direction arrow `ArrowUpRight` only when paired with destructive context (NOT default) |
| Success         | `text-success` / `bg-success/10` (`hsl(73 84% 27%)`)                | Income-direction arrow `ArrowDownLeft`, import-success toast, "split sum balanced" remainder check                                                                                           |
| Warning         | `text-warning` / `bg-warning/10` (`hsl(45 99% 34%)`)                | "Possible duplicate" inline banner background, "needs review" badge on imported rows, FX-stale warning chip                                                                                  |

Accent (`--primary`) is reserved for and ONLY for:

1. The single primary CTA per screen: "New transaction" on `/transactions`,
   "Save transaction" / "Save changes" on the transaction form, "Import" on the
   importer wizard final step, "Apply filters" in the filter sheet, "Discard
   new" on the duplicate review sheet.
2. The active/selected state in the transaction-direction `ToggleGroup` (Income
   / Expense / Transfer) on the transaction form (`data-state=on`).
3. The active filter-chip state (when a filter is engaged, the chip uses
   `data-state=on` styling: accent border + accent text on `bg-card`).
4. The current step indicator inside the importer wizard `WizardStepIndicator`
   (matches existing activity-import behavior — do not redesign).

Accent MUST NOT be used on: row backgrounds, date-group headers, category chips,
amount text, payee text, filter sheet labels, duplicate-pair surfaces, or split
editor row borders. Those use neutral tokens (`text-muted-foreground`,
`border-border`, `bg-card`).

Direction-color discipline (transaction rows):

| Direction | Amount color                | Icon                                   | Sign                                       |
| --------- | --------------------------- | -------------------------------------- | ------------------------------------------ |
| Income    | `text-success`              | `ArrowDownLeft text-success`           | implicit "+" via leading sign in formatter |
| Expense   | default (`text-foreground`) | `ArrowUpRight text-muted-foreground`   | implicit "−" via formatter                 |
| Transfer  | default (`text-foreground`) | `ArrowLeftRight text-muted-foreground` | no sign — paired-row indicator instead     |

Rationale for not painting expenses red: this app is a friendly companion, not
an accounting tool. Painting every expense red creates a stress field on the
ledger. Reserve red strictly for genuine error states (split mismatch, delete
confirm, validation). Income gets the `--success` token because positive balance
changes are worth celebrating subtly. Transfers stay neutral because they have
no economic direction.

Duplicate-confidence color ramp (applied to confidence badge background only;
text stays `text-foreground`):

| Confidence | Background               | Meaning                                         |
| ---------- | ------------------------ | ----------------------------------------------- |
| ≥ 95%      | `bg-destructive/10`      | Almost certainly a duplicate — prefer "Discard" |
| 70% – 94%  | `bg-warning/10`          | Likely duplicate — review carefully             |
| 50% – 69%  | `bg-muted/50`            | Possible duplicate — user judgment              |
| < 50%      | (not flagged, not shown) | Below threshold — system does not surface       |

Dark mode parity: all of the above resolve automatically through the `.dark` HSL
overrides already in `globals.css`. No per-component dark override needed.

Chart colors: not used in Phase 4. Phase 6 will introduce spending-by-category
and trend charts.

---

## Category Model

**Decision (Q2 → D):** Phase 4 wires transaction categories into the existing
`taxonomies` system at `apps/frontend/src/pages/settings/taxonomies/`. A new
**system taxonomy** named `"Transaction Categories"` is seeded on first run with
a flat list of 10 default categories. Users may add custom categories via the
existing taxonomies management UI (already exists, no new screen needed) OR
inline via the "Create new category" affordance inside the
`Select`/`Autocomplete` on the transaction form.

**Rationale:** Three reasons drove this over a parallel store:

1. The `Taxonomy` type already supports `isSystem: boolean`,
   `isSingleSelect: boolean`, hierarchical `parentId`, `color`, and a full CRUD
   UI. Building a parallel categories store would duplicate ~80% of this surface
   area.
2. Phase 5 (Budgeting) and Phase 6 (Reporting) both need to slice transactions
   by category. A taxonomy-backed model gives those phases a stable category ID
   rather than a free-text string that needs fuzzy reconciliation.
3. The existing `taxonomies-page.tsx` already presents a "Categories" mental
   model to the user. Extending it for transaction categories aligns with
   precedent and avoids a "two places to manage classifications" UX problem.

**Seeded default categories** (English, Single-select taxonomy, system-flagged,
sorted alphabetically except "Income" and "Other" which pin to top/bottom):

| Sort | Key             | Display name  | Direction default | Color (taxonomy.color) |
| ---- | --------------- | ------------- | ----------------- | ---------------------- |
| 0    | `income`        | Income        | income            | `--color-green-500`    |
| 1    | `dining`        | Dining        | expense           | `--color-orange-300`   |
| 2    | `entertainment` | Entertainment | expense           | `--color-purple-300`   |
| 3    | `groceries`     | Groceries     | expense           | `--color-base-400`     |
| 4    | `healthcare`    | Healthcare    | expense           | `--color-cyan-300`     |
| 5    | `housing`       | Housing       | expense           | `--color-blue-300`     |
| 6    | `shopping`      | Shopping      | expense           | `--color-magenta-300`  |
| 7    | `transport`     | Transport     | expense           | `--color-amber-300`    |
| 8    | `utilities`     | Utilities     | expense           | `--color-yellow-300`   |
| 99   | `uncategorized` | Uncategorized | (any)             | `--color-base-300`     |

Categories render as compact chips on transaction rows: `Tag` icon (12px) + 12px
chip text. Chip background uses the category's `color` token at 10% alpha
(`bg-{color}/10`); chip text uses the same color at full opacity. If color is
unset, fall back to `bg-muted text-muted-foreground`.

**Inline category creation:** the `Select` component on the transaction form
uses an `Autocomplete` (not plain `Select`). Typing a string that doesn't match
any existing category surfaces a footer affordance:
`+ Create category "{user input}"`. Clicking it opens a tiny inline `Popover`
with: name (pre-filled from input), color (color picker, defaults to a
hashed-by-name pick from the taxonomy palette), and "Create category" button. On
create, the new `TaxonomyCategory` is appended to the system "Transaction
Categories" taxonomy and selected on the form. Planner verifies that the
existing taxonomy adapter exposes a `createCategory` mutation reachable from the
transaction form — if not, planner adds it.

---

## Screen Inventory & Layouts

### 1. `/transactions` — Global ledger view (NEW route)

The primary surface for TXN-01 (manual add), TXN-03 (search/filter), TXN-06
(duplicate review banner), TXN-07 (multi-currency display), TXN-09 (running
balance per account).

Layout (desktop, `≥ md`):

```
┌──────────────────────────────────────────────────────────────────────┐
│ PageHeader: "Transactions"           [Import] [+ New transaction]    │  56px
├──────────────────────────────────────────────────────────────────────┤
│ ┌─Search ──────────────────────────┐  ← persistent, 40px height       │
│ │ 🔍 Search payee, notes, amount   │                                  │
│ └──────────────────────────────────┘                                  │
│ Filter chips row (sticky at top while scrolling, 48px):              │
│   [All accounts ▾] [Last 30 days ▾] [Any category ▾] [Any amount ▾]  │
│   [Show transfers ⚪]    [Clear filters]    ← right-aligned          │
├──────────────────────────────────────────────────────────────────────┤
│ ⚠ 3 possible duplicates from your last import   [Review duplicates]  │  ← inline banner, only when duplicates pending
├──────────────────────────────────────────────────────────────────────┤
│ Apr 24, 2026 · 3 transactions · −$84.20                               │  Date-group header (sticky, bg-muted/30)
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │ ↗ Whole Foods Market                  $42.10  Bal $4,157.90  │    │  Row
│  │   Groceries · Chase Checking · USD                          ▸│    │
│  ├──────────────────────────────────────────────────────────────┤    │
│  │ ↗ Spotify                              $9.99  Bal $4,147.91  │    │
│  │   Entertainment · Chase Checking · USD                      ▸│    │
│  ├──────────────────────────────────────────────────────────────┤    │
│  │ ↗ Uber                  €28.50 · ~$32.11  Bal $4,115.80     │    │  Row with FX
│  │   Transport · Amex Gold · EUR                               ▸│    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│ Apr 23, 2026 · 1 transaction · +$2,400.00                            │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │ ↙ Acme Corp Payroll                  $2,400.00  Bal $4,200.00│    │  Income row (success-tinted amount)
│  │   Income · Chase Checking · USD                             ▸│    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│ ... (infinite scroll / pagination per Phase 4 plan)                  │
└──────────────────────────────────────────────────────────────────────┘
```

Rules:

- **Direction icon** (16px, leading position): `ArrowDownLeft` for income
  (success-tinted), `ArrowUpRight` for expense (muted), `ArrowLeftRight` for
  transfer (muted).
- **Payee/merchant** (14-16px, weight 600). If the transaction has a `notes`
  field set, append a faint `MessageSquare` 12px icon `text-muted-foreground/60`
  with `aria-label="has note"`.
- **Sub-text line** (12px, `text-muted-foreground`):
  `{Category chip} · {Account name} · {Currency}`. Account name is suppressed on
  per-account variant of this row (see screen 2).
- **Amount column** (right-aligned, 14-16px, `tabular-nums`, weight 600). For
  income: `text-success` + `+` sign. For expense: `text-foreground` + `−` sign.
  For multi-currency, primary line shows native currency; sub-line below shows
  `~$32.11` base-currency equivalent in 12px `text-muted-foreground` if native ≠
  base.
- **Running balance** ("Bal $X,XXX.XX"): 12px caption, `text-muted-foreground`,
  right-aligned beneath the amount. Computed per account as the balance AFTER
  this transaction posts (in the account's native currency). Hidden on the
  global ledger view when the user has a multi-account filter selected (the
  number would be ambiguous); shown when filtered to a single account.
- **Split badge**: when a transaction has 2+ category splits, replace the single
  category chip with a chip reading `Split · {N} categories` using the
  `SplitSquareHorizontal` 12px icon. Tapping the row opens the detail sheet with
  all splits visible.
- **Date-group header**: format `MMM d, yyyy · {N} transactions · {± total}`.
  Total is the SUM of amounts in the group, signed (income positive, expense
  negative); displayed using the same direction-color rules as rows
  (`text-success` if positive, default if negative).
- **Group order**: most-recent date at top, descending. Within a group, order by
  `created_at DESC` (NOT activity-time) so the most recently added transaction
  is visible at the top of its day.
- **Row hover (desktop)**: `hover:bg-card/50 transition-colors duration-150`. No
  shadow elevation (rows are dense; shadow is too heavy here unlike the
  account-list where rows are sparse).
- **Row tap**: opens transaction-detail sheet (screen 4).

Filter bar:

- **Search input** (40px height, `Search` 16px leading icon, placeholder "Search
  payee, notes, amount", `text-sm`, debounced 250ms): always visible.
- **Filter chips row** (sticky at scroll top with
  `bg-background/80 backdrop-blur` blur): account, date range, category, amount
  range, "Show transfers" toggle, "Clear filters" button. Each chip uses
  `Popover` to expose the filter UI inline. Engaged chips display in active
  state (`data-state=on` → accent border + accent text per Color section).
- **Mobile filter pattern**: chips collapse into a single
  `[Filters · {N active}]` button with `SlidersHorizontal` icon. Tapping opens a
  bottom sheet with all filter sections stacked. Apply/Clear at sheet footer.

Loading state:

- Skeleton: 6 transaction-row skeletons (2 date groups of 3 rows each), reuse
  Phase 3 `Skeleton` primitive. Filter bar renders normally with disabled chips
  during initial load.

Empty states:

- See Copywriting Contract.

Error state:

- Reuse the destructive card pattern from `accounts-summary.tsx` lines 383-400 —
  heading `"We couldn't load your transactions."`, body, and Retry button.

Responsive (`< md`):

- Header: `[+ New transaction]` becomes a floating action button (FAB) pinned
  bottom-right, identical pattern to Phase 3 `/accounts` FAB. `[Import]` moves
  into the existing `MoreHorizontal` overflow menu.
- Search input: full-width, sticky beneath the page header.
- Filter chips: collapse to a single `[Filters]` button (see above).
- Row interior: `px-4 py-3`. Direction icon and amount stay; running-balance
  caption is preserved on a single-account filter and hidden otherwise (mobile
  width too narrow for ambiguity).

---

### 2. Per-account "Recent transactions" abridged section

Embedded inside the existing `apps/frontend/src/pages/account/account-page.tsx`,
beneath the account hero (and beneath CC sections from Phase 3 if the account is
`CREDIT_CARD`). Provides TXN-09 running balance in the most natural context.

Layout:

```
┌────────────────────────────────────────────────────┐
│ Recent transactions                  [View all →]  │
├────────────────────────────────────────────────────┤
│ Apr 24, 2026                                       │  Date-group header (compact, no count/total — too noisy in this card)
│  ↗ Whole Foods       $42.10   Bal $4,157.90        │
│  ↗ Spotify            $9.99   Bal $4,147.91        │
│ Apr 23, 2026                                       │
│  ↙ Payroll        $2,400.00   Bal $4,200.00        │
│ ... (max 10 rows)                                  │
├────────────────────────────────────────────────────┤
│ [View all transactions in this account →]         │  Footer link
└────────────────────────────────────────────────────┘
```

Rules:

- Renders the **last 10 transactions** for this account. Less than 10 → render
  all available; 0 → render the empty state below the header (no footer link).
- Same row component as global ledger, but with `account-suppressed` variant
  (the sub-text line drops the account name since it would be redundant).
- "View all" footer link: navigates to `/transactions?accountId={id}` — the
  global ledger pre-filtered to this account. The pre-applied account filter
  chip displays in active state and is removable like any other.
- No date-group counts/totals at this density (would be visually noisy in the
  card surface). Just the date header.
- No filter bar, no search — this section is "snapshot only".

Empty state inside this card:

- Heading: `"No transactions yet"`
- Body:
  `"Add your first transaction or import a CSV to start tracking this account."`
- CTA: `[+ New transaction]` (secondary button, opens the transaction form
  pre-scoped to this account).

---

### 3. New / Edit transaction form (Sheet desktop, Dialog mobile full-screen)

Triggered by:

- `[+ New transaction]` on `/transactions` (no pre-fill)
- `[+ New transaction]` on the per-account empty state (pre-fills `accountId`)
- Clicking a transaction row in the ledger and choosing "Edit transaction" from
  the detail sheet (loads existing values; submit calls `update` mutation)

Form sections (in render order):

1. **Direction** — `ToggleGroup` (3 options, single-select, required):
   - Income · `ArrowDownLeft` icon · label "Income"
   - Expense · `ArrowUpRight` icon · label "Expense"
   - Transfer · `ArrowLeftRight` icon · label "Transfer" Selected option uses
     `data-state=on` → accent border + accent text + filled icon. Direction is
     locked once a category is picked (changing direction resets category).
     Default: Expense.

2. **Amount** — `MoneyInput`, required, currency tied to the selected account
   below. `data-autofocus` on this field when the form opens fresh. Helper
   below: `"Enter as a positive number — direction handles the sign."`

3. **Account** — `Select` (or `ResponsiveSelect` on mobile), required. Lists
   non-archived accounts (per Phase 3 D-19 rule). Pre-fills with user's
   last-used account or pre-scoped account when entering from per-account
   context. For Direction = Transfer, this becomes the **Source** account; a
   second field appears beneath: `Destination` (required, must differ from
   source).

4. **Date** — `DatePickerInput`, required, default `today`. Constrained to not
   be more than 1 day in the future (allows scheduling next-day, blocks typo of
   2027). Below 30 days, render relative suffix in caption: `"3 days ago"`.

5. **Payee / Merchant** — `Input`, required for Income/Expense, hidden for
   Transfer. Free-text. Placeholder: Income → `"e.g. Acme Corp Payroll"`;
   Expense → `"e.g. Whole Foods Market"`. No autocomplete in Phase 4 (Phase 8
   adds AI-suggested payees per AI-06 — explicitly out of scope here).

6. **Category** — `Autocomplete` (custom-create-enabled, see Category Model
   section), required for Income/Expense, hidden for Transfer. Filtered to
   categories matching the current Direction by default; `Show all categories`
   collapse-toggle to reveal off-direction categories. Inline
   `+ Create category "{input}"` affordance.

7. **Split transaction** (Income/Expense only) — collapsed by default. A small
   `[+ Split transaction]` text button beneath the Category field. When clicked,
   the Category field is replaced by a vertical stack of split-rows (see Section
   6 below for the split editor contract).

8. **Notes** — `Textarea`, optional, 3-row min, 200-char soft limit.
   Placeholder: `"Add a note about this transaction (optional)"`.

9. **More details** (collapsible `Collapsible` block, default closed for
   simplicity):
   - **FX rate override** — appears only if
     `transaction.currency !== account.currency`. Displays the live system FX
     rate as read-only by default, with `[Override]` toggle. When overridden:
     `MoneyInput` with `tabular-nums`. Helper:
     `"Use the rate from your statement or receipt if it differs from the system rate."`
   - **Reference / external ID** — `Input`, optional. Helper:
     `"Bank reference, check number, or order ID."`
   - **Tags** — deferred. Phase 4 does not expose a tag editor (taxonomies
     handle the categorization need; freelance/business tags arrive in Phase
     12).

Footer actions:

- Left: `[Discard]` (ghost variant)
- Right: primary CTA — `[Save transaction]` on create (accent), `[Save changes]`
  on edit. Disabled when form is invalid OR (in edit mode) when no field has
  changed from the loaded values.

Submission:

- Single `create_transaction` or `update_transaction` mutation.
- Success → close form, toast `"Transaction saved"`, navigate or refresh the
  underlying list (planner decides cache invalidation strategy).
- Validation messages (inline below field, `text-xs text-destructive`):
  - `"Amount must be greater than 0."`
  - `"Account is required."`
  - `"Source and destination accounts must be different."`
  - `"Date can't be more than a day in the future."`
  - `"Payee is required."`
  - `"Category is required."`
  - `"Split totals must equal the transaction amount."` (live, non-blocking
    until submit)

---

### 4. Transaction detail sheet

Opens when a row in the ledger is tapped. Reuses Phase 3 `Sheet` (desktop right
side) / mobile `Dialog`. Read-only display with action buttons.

Layout:

```
┌─────────────────────────────────────────────┐
│ ↗ Whole Foods Market               [×]      │  Header: direction icon + payee + close
│ Apr 24, 2026 · Chase Checking · USD         │  Sub-header
├─────────────────────────────────────────────┤
│                                             │
│         $42.10                              │  Hero amount (18-24px display)
│         Groceries                           │  Category chip
│                                             │
│ Notes                                       │
│ Picked up dinner stuff for the weekend.     │
│                                             │
│ Account                                     │
│ Chase Checking ›                            │  Tappable, navigates to account
│                                             │
│ Created                                     │
│ Apr 24, 2026 · 7:42 PM                      │
│                                             │
├─────────────────────────────────────────────┤
│ [Delete transaction]       [Edit transaction]│  Footer
└─────────────────────────────────────────────┘
```

Rules:

- Hero amount: 18-24px display weight 600, `tabular-nums`, color per direction.
- For Transfer: hero shows `Source → Destination · amount`, with two account
  links beneath.
- For Split: render each split row beneath the amount as
  `{Category chip} · ${amount}` with the per-row notes if any.
- `[Delete transaction]` → opens `AlertDialog` (see Copywriting → Destructive).
  On confirm: hard delete via `delete_transaction` mutation, close sheet, toast.
- `[Edit transaction]` → closes detail sheet, opens transaction form pre-filled
  with current values.

---

### 5. CSV/OFX import wizard (forked from existing activity import)

**Decision (Q3 → A):** Reuse and fork
`apps/frontend/src/pages/activity/import/activity-import-page.tsx`. Build a
parallel `transaction-import-page.tsx` in
`apps/frontend/src/pages/transactions/import/` that mirrors the existing file
structure (context, steps, components, hooks, utils) but with
transaction-specific types in place of activity types.

**Step list (4 steps, 1 fewer than activity import):**

| #   | ID        | Label               | Component                                 | Notes                                                                      |
| --- | --------- | ------------------- | ----------------------------------------- | -------------------------------------------------------------------------- |
| 1   | `upload`  | Upload              | `UploadStep` (reused; CSV+OFX MIME types) | Drag-drop file or pick. OFX is parsed server-side; UI is identical to CSV. |
| 2   | `mapping` | Mapping             | `TransactionMappingStep` (forked)         | Map CSV columns → transaction fields (see below)                           |
| 3   | `review`  | Review transactions | `TransactionReviewStep` (forked)          | Editable preview table; mark each row valid/skip                           |
| 4   | `confirm` | Import              | `TransactionConfirmStep` (forked)         | Summary, idempotency confirmation, "Import" CTA                            |

The existing `assets` step from activity import is **explicitly removed** —
transactions don't need asset resolution. The `result` "thanks" step from
activity import IS retained as the post-confirm landing.

**Mapping step required fields** (planner derives these from
`IMPORT_REQUIRED_FIELDS` constant):

| Required | Field                            | Source CSV column example                     |
| -------- | -------------------------------- | --------------------------------------------- |
| Yes      | `date`                           | "Posted Date", "Transaction Date"             |
| Yes      | `amount` OR (`debit` + `credit`) | "Amount", or split "Debit" + "Credit" columns |
| Yes      | `payee`                          | "Description", "Payee", "Merchant"            |
| No       | `category`                       | "Category" (auto-matches existing taxonomy)   |
| No       | `notes`                          | "Notes", "Memo"                               |
| No       | `currency`                       | "Currency" (defaults to account currency)     |
| No       | `external_id`                    | "Reference", "ID"                             |

Account selection happens in the upload step (file is scoped to a single account
at import time). User-chosen `accountId` flows through context to all downstream
steps.

**Step indicator**: reuse `WizardStepIndicator` exactly. Active step uses
`--primary` (per Phase 3 inheritance).

**Help affordance**: reuse `ImportHelpPopover`. Add transaction-specific copy
explaining CSV vs OFX differences:

> CSV files give you full column-mapping control. OFX files are parsed
> automatically — we'll do our best to detect dates, amounts, and payees. Either
> way, you'll get to review every transaction before it's saved.

**Cancel confirmation**: reuse `CancelConfirmationDialog`. Copy:

> Title: "Discard this import?" Body: "Your file and column choices will be
> cleared. You can start over anytime." Confirm label: "Discard" Cancel label:
> "Keep editing"

**Import success**: After `Import` mutation succeeds, navigate to
`/transactions` with a success toast: `"{N} transactions imported"`. If the
import surfaced potential duplicates, the duplicate-review banner (screen 6
below) appears at the top of the ledger.

---

### 6. Duplicate review

**Decision (Q4 → A):** Inline banner inside `/transactions` ledger.

**Banner** (renders only when `pendingDuplicateCount > 0`):

```
┌──────────────────────────────────────────────────────────────────────┐
│ ⚠ 3 possible duplicates from your last import   [Review duplicates →]│
└──────────────────────────────────────────────────────────────────────┘
```

- Background: `bg-warning/10`, border: `border-warning/30`, icon:
  `AlertCircle 16px text-warning`, text: `text-sm text-foreground`.
- Position: directly beneath the filter chips row, above the first date-group
  header.
- Persistent across page reloads (driven by query of pending-duplicates).
- Dismiss is implicit — banner disappears when count reaches 0.

**Review sheet** (opens from the banner `[Review duplicates]` button — `Sheet`
desktop / full-screen `Dialog` mobile):

```
┌──────────────────────────────────────────────────────────────────────┐
│ Review duplicates · 1 of 3                                    [×]    │
├──────────────────────────────────────────────────────────────────────┤
│ ┌────────────────────────────┐  ┌────────────────────────────┐      │
│ │ EXISTING                   │  │ NEW (from import)          │      │  Two cards side-by-side desktop
│ │ Apr 24, 2026               │  │ Apr 24, 2026               │      │  Stack vertically mobile
│ │ Whole Foods Market         │  │ Whole Foods                │      │
│ │ Groceries · Chase Checking │  │ Groceries · Chase Checking │      │
│ │ −$42.10                    │  │ −$42.10                    │      │
│ │ ref: 0x4a8b                │  │ ref: 0x4a8b                │      │
│ └────────────────────────────┘  └────────────────────────────┘      │
│                                                                      │
│           [≥95% match · almost certainly duplicate]                  │  Confidence chip
│                                                                      │
│ ┌──────────────────────────────────────────────────────────────┐    │
│ │ How would you like to resolve this?                          │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  [Discard new]                                  [Keep both]          │  Footer actions
│                                                                      │
│  [‹ Previous]    [Skip for now]    [Next ›]                          │  Pagination
└──────────────────────────────────────────────────────────────────────┘
```

Rules:

- **Two-card layout**: existing on left, new on right. Each card uses
  `bg-card border-border rounded-md p-4`. The card whose action wins
  (`Discard new` → existing wins; `Keep both` → both highlighted) animates a
  green border briefly post-action.
- **Confidence chip**: centered between the two cards. Background per
  duplicate-confidence ramp (Color section). Text:
  `"≥95% match · almost certainly duplicate"`, `"Likely duplicate (85% match)"`,
  or `"Possible duplicate (62% match)"`.
- **Primary actions**:
  - `[Discard new]` (accent — this is the most likely action; tied to the
    confidence ramp recommendation): deletes the new transaction. Single click,
    no further confirmation.
  - `[Keep both]` (secondary/outline): leaves both transactions in place. Mark
    this pair as resolved.
- **Pagination**:
  - `[‹ Previous]`, `[Next ›]` cycle through pending pairs.
  - `[Skip for now]` advances without resolving — pair remains in the queue,
    counter on the banner stays.
  - Reaching the last unresolved pair and clicking `[Discard new]` /
    `[Keep both]` closes the sheet automatically and updates the banner count.
- **Empty resolution state**: if user resolves the final pair, sheet shows
  `"All caught up"` heading + `"Nice work — every duplicate has been reviewed."`
  - `[Done]` button before closing.
- **Direction symmetry**: the user can also choose `[Keep both]` from the
  EXISTING side (same button — neutral framing). There's no "discard existing"
  option — Phase 4 does not let import data overwrite manually-entered records.

---

### 7. Split-category editor (inline)

**Decision (Q5 → A):** Inline expansion inside the transaction form, replacing
the single Category field.

Layout (when split mode is active):

```
Category
 ┌──────────────────────────────────────────────────────────────┐
 │ Split row 1                                            [🗑]   │
 │  Category: [Groceries ▾]      Amount: $30.00                 │
 ├──────────────────────────────────────────────────────────────┤
 │ Split row 2                                            [🗑]   │
 │  Category: [Household ▾]      Amount: $12.10                 │
 ├──────────────────────────────────────────────────────────────┤
 │ [+ Add another split]                                        │
 └──────────────────────────────────────────────────────────────┘

  Total split: $42.10  ✓ matches transaction amount        ← live indicator
                                       (or)
  Total split: $40.00  Remaining $2.10 ⚠                   ← live remainder
                                       (or)
  Total split: $43.50  Over by $1.40 ⚠                     ← live overage

 [← Back to single category]
```

Rules:

- **Initial state**: clicking `[+ Split transaction]` from the form's
  Category-field row creates 2 default split rows (1 row alone is degenerate;
  the user is committing to a split). Row 1 inherits whatever category was
  selected before, with amount = transaction amount; row 2 starts empty. User
  adjusts amounts manually.
- **Each split row**: bordered `border border-border rounded-md p-3 space-y-2`,
  with `[Trash2]` icon button at top-right (12px icon, 32px tap target,
  `aria-label="Remove split row"`). Trash is disabled when only 2 rows remain
  (minimum split).
- **Category picker per row**: same `Autocomplete` as the single-category case,
  with the same inline `+ Create category` affordance.
- **Amount input per row**: `MoneyInput`, currency locked to the transaction
  currency. Each row's amount must be `> 0`.
- **Add row affordance**: `[+ Add another split]` ghost button, full-width
  inside the split container. Adds an empty row.
- **Live remainder indicator** (beneath the split container, 12px caption):
  - `Sum === amount` → `"Total split: ${sum} ✓ matches transaction amount"` in
    `text-success` with `Check` 12px icon.
  - `Sum < amount` → `"Total split: ${sum}  Remaining ${diff} ⚠"` in
    `text-warning`.
  - `Sum > amount` → `"Total split: ${sum}  Over by ${diff} ⚠"` in
    `text-destructive`.
- **Submit gating**: form is invalid (submit disabled) until `sum === amount`
  exactly (to the cent — penny rounding errors block submit; user must fix).
- **Back to single category**: `[← Back to single category]` ghost button
  beneath the indicator. Confirmation: if any split row has a non-default value,
  show inline confirm `"Discard splits and use a single category?"` with [Yes,
  discard] / [Keep splits]. On accept, the form reverts to single-Category mode with
  row 1's category as the active value.

Storage shape: planner determines the persistence model (likely a
`transaction_splits` table with `transaction_id`, `category_id`, `amount`,
`notes` per row), but the UI contract treats splits as a child collection of the
transaction, not a separate entity. Sub-row notes are NOT exposed in the inline
editor in Phase 4 (deferred to a "split detail" affordance in a future phase if
user demand emerges).

---

## Copywriting Contract

Exact copy. Executor uses verbatim.

### Page titles

| Screen                            | Title                 | Subtitle                                        |
| --------------------------------- | --------------------- | ----------------------------------------------- |
| `/transactions`                   | "Transactions"        | (none — page hero only)                         |
| `/transactions/import`            | "Import transactions" | "Bring in your CSV or OFX file from your bank." |
| Transaction form (sheet)          | "New transaction"     | (none — direction toggle conveys intent)        |
| Transaction form (edit mode)      | "Edit transaction"    | _payee_ · _date_                                |
| Transaction detail sheet          | _payee/merchant_      | _date_ · _account_ · _currency_                 |
| Duplicate review sheet            | "Review duplicates"   | "{current} of {total}"                          |
| Per-account "Recent transactions" | "Recent transactions" | (none — embedded as card title only)            |

### Primary CTAs

| Location                                 | Label                                                                                             |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `/transactions` header                   | "New transaction"                                                                                 |
| `/transactions` header (secondary)       | "Import"                                                                                          |
| Per-account empty state                  | "New transaction"                                                                                 |
| Per-account "Recent transactions" footer | "View all transactions in this account"                                                           |
| Transaction form footer (create)         | "Save transaction"                                                                                |
| Transaction form footer (edit)           | "Save changes"                                                                                    |
| Transaction form footer (discard)        | "Discard"                                                                                         |
| Transaction detail sheet                 | "Edit transaction"                                                                                |
| Transaction detail sheet (destructive)   | "Delete transaction"                                                                              |
| Importer step 1                          | "Continue to mapping"                                                                             |
| Importer step 2                          | "Continue to review"                                                                              |
| Importer step 3                          | "Continue to confirm"                                                                             |
| Importer step 4                          | "Import {N} transactions"                                                                         |
| Importer step navigation — cancel        | "Cancel import" (ghost, bottom-left of wizard footer — opens AlertDialog "Discard this import?") |
| Duplicate banner                         | "Review duplicates"                                                                               |
| Duplicate review sheet — primary action  | "Discard new"                                                                                     |
| Duplicate review sheet — secondary       | "Keep both"                                                                                       |
| Duplicate review sheet — pagination      | "Skip for now"                                                                                    |
| Split editor                             | "+ Split transaction" (entry); "+ Add another split" (within); "← Back to single category" (exit) |
| Filter sheet (mobile)                    | "Apply filters"                                                                                   |
| Filter sheet — clear                     | "Clear filters"                                                                                   |
| Inline category create                   | "Create category"                                                                                 |

Never abbreviate. Never use "+" alone — pair with text label on desktop. Mobile
FAB uses icon-only with `aria-label="New transaction"`.

### Empty states

| Scenario                                             | Heading                               | Body                                                                                                                              | CTA                                                       |
| ---------------------------------------------------- | ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `/transactions` — zero transactions ever             | "Nothing here yet"                    | "Add your first transaction or import a CSV from your bank to start tracking."                                                    | "New transaction" (primary) + "Import" (secondary, ghost) |
| `/transactions` — filters return zero                | "No transactions match these filters" | "Try widening your date range, clearing a filter, or searching for a different payee."                                            | "Clear filters" (ghost)                                   |
| `/transactions` — search returns zero                | "Nothing matches that search"         | "Try a different word, or check the date range filter."                                                                           | (no CTA)                                                  |
| Per-account "Recent transactions" — none for account | "No transactions yet"                 | "Add your first transaction or import a CSV to start tracking this account."                                                      | "New transaction"                                         |
| Importer — invalid file                              | "We couldn't read that file"          | "Make sure it's a CSV or OFX file from your bank. If it is, try opening it in a text editor first to confirm it's not corrupted." | "Choose a different file"                                 |
| Importer — zero transactions parsed                  | "No transactions found in this file"  | "Double-check that the file has rows beneath the headers, or try a different file."                                               | "Back to upload"                                          |
| Duplicate review — all resolved                      | "All caught up"                       | "Nice work — every duplicate has been reviewed."                                                                                  | "Done"                                                    |
| Filter chip — no categories yet                      | "No categories yet"                   | "Categories will appear here as you add them. Create one inline when you save your next transaction."                             | (no CTA)                                                  |

### Error states

| Scenario                                             | Copy                                                                                                                                                     |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/transactions` load failure                         | Heading: "We couldn't load your transactions." Body: "Try refreshing. If this keeps happening, restart the app or check that the database is reachable." |
| Create transaction failure (generic)                 | Toast: "Couldn't save that transaction. Your changes are still here — try again."                                                                        |
| Create transaction failure (validation, server)      | Inline field error using server message; fallback: "Something in this form needs another look."                                                          |
| Update transaction failure                           | Toast: "Changes didn't save. Check your connection and try again." Sheet stays open with values intact.                                                  |
| Delete transaction failure                           | Toast: "Couldn't delete that transaction just now. Try again in a moment."                                                                               |
| Import failure (parse error)                         | Inline alert at the top of the wizard step: "We hit a snag reading this file. {error message from parser}" — always includes the parser-supplied detail. |
| Import failure (mid-write)                           | Toast: "Some transactions didn't save. The first {N} are in your ledger; the rest are still here for review." Wizard returns to review step.             |
| Import failure (network)                             | Toast: "Lost the connection mid-import. Nothing was saved — try again when you're back online."                                                          |
| Duplicate resolve failure                            | Toast: "Couldn't update that pair just now. Try again in a moment."                                                                                      |
| Split sum mismatch (live, not blocking until submit) | Indicator beneath split rows: "Total split: ${sum} Remaining ${diff} ⚠" / "Total split: ${sum} Over by ${diff} ⚠"                                        |
| FX rate fetch failure                                | Caption beneath FX field: "We couldn't fetch today's rate — using the most recent stored rate." (NOT a blocking error)                                   |

### Destructive confirmations

| Action                       | Dialog title                     | Body                                                                                                                      | Confirm label  | Dismiss label    |
| ---------------------------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | -------------- | ---------------- |
| Delete transaction           | "Delete this transaction?"       | "This will permanently remove the transaction and any splits attached to it. Account balances will update automatically." | "Delete"       | "Keep editing"   |
| Discard import               | "Discard this import?"           | "Your file and column choices will be cleared. You can start over anytime."                                               | "Discard"      | "Keep editing"   |
| Discard splits (within form) | (inline confirm, no AlertDialog) | "Discard splits and use a single category?"                                                                               | "Yes, discard" | "Keep splits"    |

Note: Phase 4 IS hard-delete-capable for transactions (unlike Phase 3 accounts
which only archive). This matches user expectations — transactions are
single-row records the user routinely cleans up. Account hierarchy retains
archive-only semantics from Phase 3.

### Tone check (re-asserted from Phase 3)

- Friendly, present-tense, plural-second-person ("we keep", "you can").
- Match Phase-1 voice: "Your friendly finance companion." No finance jargon
  where plain words work ("Payee" instead of "Payer/Payee", "Notes" instead of
  "Memo", "Category" instead of "Classification", "Bal" abbreviation in caption
  is acceptable for density).
- Never scold for high spending. The color discipline (income green, expense
  neutral) does the work; copy stays neutral. Avoid "danger", "warning", "alert"
  in user-facing strings unless the state is actionable (parser errors, network
  errors, split sum mismatches all qualify).
- Emoji: NONE in UI strings. The unicode arrows (↗ ↙ ↔) used in row icons are
  rendered as `lucide` icon components, not emoji characters in copy.

### Accessibility labels

- Mobile FAB: `aria-label="New transaction"`.
- Direction toggle group: `aria-label="Transaction direction"`. Each toggle:
  `aria-label="Income"` / `aria-label="Expense"` / `aria-label="Transfer"`.
- Direction icons in rows: `aria-hidden="true"` (the surrounding row's
  accessible name communicates direction via amount sign).
- Filter chips: `aria-label="Filter by {filter name}"`. Active state surfaces
  via `aria-pressed="true"`.
- Search input: `aria-label="Search transactions"`.
- Duplicate review sheet pagination: `aria-label="Previous duplicate pair"`,
  `aria-label="Next duplicate pair"`, `aria-label="Skip this pair for now"`.
- Split editor remainder indicator: `aria-live="polite"` so screen readers
  announce the running total as the user types.
- Split editor row remove button: `aria-label="Remove split row"`.
- Date-group headers: `role="heading" aria-level="2"`.

---

## Interaction Contracts

| Interaction                  | Contract                                                                                                                                                                                                          |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Row hover (desktop)          | `hover:bg-card/50 transition-colors duration-150`. NO shadow elevation (rows are dense).                                                                                                                          |
| Row tap (mobile)             | Active state: `bg-card/80`; opens detail sheet                                                                                                                                                                    |
| Row swipe (mobile)           | DEFERRED. Phase 4 does not implement swipe-to-delete or swipe-to-edit. Detail sheet is the entry.                                                                                                                 |
| Filter chip click            | Toggles popover; engaged chip uses `data-state=on`                                                                                                                                                                |
| Filter chip clear (X)        | `Icons.X` 12px appears on engaged chip; clicking removes that single filter only                                                                                                                                  |
| Search input                 | Debounced 250ms; clears via `Esc` key or `Icons.X` button at right edge                                                                                                                                           |
| Date-group header click      | DEFERRED — date groups are NOT collapsible in Phase 4 (would complicate running-balance scroll math). Phase 6 may add.                                                                                            |
| Form direction change        | Selecting a different direction resets Category but preserves Amount/Account/Date/Payee/Notes                                                                                                                     |
| Form submit                  | Cmd/Ctrl+Enter from anywhere in the form submits                                                                                                                                                                  |
| Form escape                  | Closes sheet; if dirty, opens unsaved-changes confirm `AlertDialog`: title "Discard your changes?", body "Anything you typed will be lost.", confirm "Discard", cancel "Keep editing"                             |
| Importer step navigation     | `[Continue to {next step}]` button advances; `[Cancel import]` opens cancel-confirmation dialog; `[← Back]` regresses without confirm                                                                                    |
| Importer file drop           | Reuse `FileDropzone` interaction patterns from activity import — drag-over highlights with `border-primary`                                                                                                       |
| Duplicate sheet keyboard nav | `←` / `→` cycle pairs (matches `[‹ Previous]` / `[Next ›]`); `Esc` closes; `Enter` triggers focused button                                                                                                        |
| Split row add                | New row animates in with `motion/react` slide-down (already used in `activity-import-page.tsx` line 12) — duration 150ms                                                                                          |
| Split row remove             | Row animates out with same motion (slide-up 150ms)                                                                                                                                                                |
| Split sum live update        | Recalculates on each keystroke in any amount input; indicator under the container updates immediately with `aria-live="polite"`                                                                                   |
| Toast                        | Reuse existing Sonner with `--toast-success-*` / `--toast-error-*` / `--toast-warning-*` tokens. 4s auto-dismiss, swipe-to-dismiss on mobile                                                                      |
| Privacy mode                 | All `PrivacyAmount` usages respect the existing privacy toggle. The hero amount in detail sheet, all row amounts, all running-balance captions, all date-group totals, and all FX sub-amounts use `PrivacyAmount` |
| Modal autofocus              | First enabled input in the form. For new transaction: the Amount field. For edit: no autofocus (user is reviewing existing data).                                                                                 |
| Keyboard escape              | Closes Dialog/Sheet via the unsaved-changes path described above                                                                                                                                                  |

---

## Component Inventory (Phase 4)

Components that MUST be used (from `@whaleit/ui`):

- `Page`, `PageHeader`, `PageContent` — page scaffolding (same as Phase 3).
- `Button` — default, `variant="ghost"` for icon-only and "Back to single
  category", `variant="destructive"` for "Delete transaction" only.
- `Card`, `CardHeader`, `CardTitle`, `CardContent` — duplicate-pair surfaces,
  detail-sheet sections, "Recent transactions" container.
- `Sheet` / `Dialog` — transaction form, transaction detail, duplicate review,
  filter sheet (mobile).
- `AlertDialog` — delete-transaction confirm, discard-import confirm,
  unsaved-changes confirm.
- `Form`, `FormField`, `FormItem`, `FormLabel`, `FormControl`, `FormMessage` —
  all form composition.
- `Input`, `Textarea`, `Select`, `ResponsiveSelect`, `DatePickerInput`,
  `DateRangePicker`, `ToggleGroup`, `ToggleGroupItem`, `Switch`, `Checkbox`,
  `Autocomplete` — field primitives. `Autocomplete` is the pillar of the
  category picker.
- `MoneyInput`, `CurrencyInput` — all money entry (transaction amount, split row
  amounts, FX override).
- `PrivacyAmount`, `AmountDisplay` — all money display.
- `Popover`, `PopoverTrigger`, `PopoverContent` — filter chip popovers, inline
  category-create popover.
- `Collapsible`, `CollapsibleContent`, `CollapsibleTrigger` — "More details"
  block on the transaction form, "Show all categories" toggle inside the
  Category Autocomplete.
- `Tabs`, `TabsContent`, `TabsList`, `TabsTrigger` — importer mapping step may
  use Tabs to switch between mapping table and CSV preview (matches existing
  `mapping-step-unified.tsx`).
- `Separator` — between date groups in dense layouts; also separates duplicate
  pair cards from confidence chip.
- `Skeleton` — loading states.
- `EmptyPlaceholder` — empty states listed above.
- `Tooltip`, `TooltipTrigger`, `TooltipContent` — date-group total tooltip
  showing per-currency breakdown when group spans currencies; FX-rate tooltip
  showing rate source.
- `Icons` (lucide map): the full Phase 3 set PLUS `ArrowDownLeft`,
  `ArrowUpRight`, `ArrowLeftRight`, `Search`, `Filter`, `SlidersHorizontal`,
  `Upload`, `FileText`, `Tag`, `SplitSquareHorizontal`, `Copy`, `CalendarDays`,
  `X`, `Check`, `Plus`, `Trash2`, `MessageSquare`.

Components to FORK from existing activity-import (do NOT duplicate code; planner
extracts shared shells where reasonable):

- `FileDropzone`
  (`apps/frontend/src/pages/activity/import/components/file-dropzone.tsx`) —
  fork or share.
- `MappingTable`
  (`apps/frontend/src/pages/activity/import/components/mapping-table.tsx`) —
  fork; the field set is different.
- `CSVFileViewer`
  (`apps/frontend/src/pages/activity/import/components/csv-file-viewer.tsx`) —
  share verbatim.
- `WizardStepIndicator`
  (`apps/frontend/src/pages/activity/import/components/wizard-step-indicator.tsx`)
  — share verbatim.
- `StepNavigation`
  (`apps/frontend/src/pages/activity/import/components/step-navigation.tsx`) —
  share verbatim.
- `TemplatePicker`
  (`apps/frontend/src/pages/activity/import/components/template-picker.tsx`) —
  fork; templates are now transaction-mapping templates.
- `HelpTooltip`
  (`apps/frontend/src/pages/activity/import/components/help-tooltip.tsx`) —
  share verbatim.
- `CancelConfirmationDialog` — share verbatim with parameterized copy.
- `ImportProvider` context — fork. Transaction state shape differs from activity
  (no asset resolution).

Planner decision: whether to extract a shared `import-wizard-shell` package or
keep two parallel directories with selective sharing. UI-SPEC requires only that
the visual contract is identical between the two wizards.

Components that MUST NOT be introduced this phase:

- Any new block from `@animate-ui` or `@diceui` third-party registries. No
  safety gate run → not allowed to land. (Inherited from Phase 3.)
- Any new shadcn primitive not already in `packages/ui/src/components/ui/`.
- Any new chart (Phase 6 scope).
- Any new icon library.
- Any AI-suggestion UI (Phase 8 scope).
- Any subscription/recurring-detection UI (Phase 7 scope).
- Any budget assignment UI (Phase 5 scope).
- Any swipe-row gesture (deferred).

---

## Registry Safety

| Registry        | Source                                          | Blocks Used in Phase 4                                                                | Safety Gate                   |
| --------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------- | ----------------------------- |
| shadcn official | `packages/ui` (baseColor slate, local vendored) | All primitives listed in Component Inventory. No new install this phase — reuse only. | not required (local vendored) |
| `@animate-ui`   | `https://animate-ui.com/r/{name}.json`          | none                                                                                  | n/a — no blocks added         |
| `@diceui`       | `https://diceui.com/r/{name}.json`              | none                                                                                  | n/a — no blocks added         |

Phase 4 is **net-zero** on third-party registry consumption (matches Phase 3).
If the planner or executor proposes pulling a new block from `@animate-ui` or
`@diceui`, they MUST (a) stop, (b) return to `gsd-ui-researcher` for a vetting
gate (`npx shadcn view {block} --registry {url}`), and (c) re-open this UI-SPEC
for amendment.

---

## Responsive Breakpoints

Inherits Phase 3 breakpoints. Phase 4 specifics:

| Breakpoint    | Width      | Notable Phase 4 Layout Shift                                                                                                                                                                                                                                  |
| ------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| base (mobile) | `< 640px`  | Single column, FAB for "New transaction", filter chips collapse to single `[Filters]` button + bottom-sheet, transaction form full-screen, duplicate-review sheet stacks the two pair cards vertically, importer is full-screen with step nav fixed to bottom |
| `sm`          | `≥ 640px`  | Form fields side-by-side where two related (split row category + amount); filter sheet content tightens                                                                                                                                                       |
| `md`          | `≥ 768px`  | Filter chips render inline in the header bar, "+ New transaction" promotes to inline button (FAB hides), duplicate-pair cards render side-by-side                                                                                                             |
| `lg`          | `≥ 1024px` | Detail sheet uses fixed 480px width on the right; ledger row accommodates a 4th column for "Bal" running-balance display when single-account-filtered                                                                                                         |

Mobile nav coexistence: `/transactions` sits inside the existing `app-shell`
with `data-page-scroll-container`. Phase 4 adds a new bottom-nav entry for
"Transactions" — planner verifies the bottom-nav config
(`apps/frontend/src/components/header.tsx` or equivalent) accommodates the new
entry without breaking existing layout. The FAB respects `pb-safe` inset and the
mobile nav height (`--mobile-nav-ui-height`).

---

## Numerical Precision & Formatting

| Value                           | Format                                                                                                                                                                |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Transaction amount              | 2 decimal places via `PrivacyAmount` / `AmountDisplay`. Native currency primary; base-currency equivalent in 12px caption when native ≠ base.                         |
| Running balance ("Bal")         | Same as amount. Caption density is 12px `tabular-nums text-muted-foreground`.                                                                                         |
| Date                            | `MMM d, yyyy` (e.g. "Apr 24, 2026"). Group headers use this; rows display only time below 24h-old (e.g. "7:42 PM"); else date.                                        |
| Date-group total                | Same currency precision as transaction amount. Mixed-currency groups: render base-currency total only, with a Tooltip listing per-currency sub-totals.                |
| Split row amount                | Same as transaction amount (2 decimals).                                                                                                                              |
| Split sum indicator             | "Total split: $42.10 ✓ matches transaction amount" (success); "Total split: $40.00 Remaining $2.10 ⚠" (warning); "Total split: $43.50 Over by $1.40 ⚠" (destructive). |
| Duplicate-confidence percentage | Integer percent on the chip (e.g. "≥95% match"). For 50-94, show the actual rounded integer ("85% match", "62% match").                                               |
| FX rate display                 | 4 decimal places (e.g. "1.1284"). Tooltip on the rate shows source ("System rate · Apr 24, 2026" or "Manual override").                                               |
| Search debounce                 | 250ms.                                                                                                                                                                |
| Filter sheet "active count"     | Integer count of engaged filter chips, e.g. "Filters · 3 active".                                                                                                     |

---

## Non-Goals (Explicit)

The following are deliberately excluded from Phase 4 UI-SPEC, even though they
may seem natural:

- AI-powered category/payee suggestion UI (Phase 8 — covers AI-05, AI-06).
- AI-powered conversational transaction entry (Phase 8 — AI-03).
- Receipt OCR upload + extraction UI (Phase 8 — AI-04).
- Recurring transaction / subscription detection UI (Phase 7 — SUBS-01).
- Budget assignment UI inside the transaction form (Phase 5 — BUDG-05).
- Spending-by-category charts on `/transactions` (Phase 6 — RPT-01).
- Income vs expense bars (Phase 6 — RPT-03).
- Net worth ribbon at the top of `/transactions` (Phase 6 — RPT-04).
- Per-transaction tag editor (deferred; Phase 12 freelancer mode handles
  business/personal toggle).
- Bulk edit UI (multi-select rows + bulk recategorize). Deferred — single-row
  edit covers v1 needs.
- Swipe-row gestures (deferred — see Interaction Contracts).
- Date-group collapse/expand (deferred — running-balance math interplay).
- Duplicate-detection rule editor (Phase 4 ships system-driven detection only;
  surfacing thresholds to the user is deferred).
- Reconciliation widget (Phase 4 ships the data foundation per Phase 3 D-14; the
  user-facing reconcile UI is deferred to a future phase).

---

## Pre-Population Source Map

For checker/auditor traceability.

| Field                      | Source                                                                                                                   |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Design System tokens       | `packages/ui/components.json` + Phase 3 UI-SPEC (`03-UI-SPEC.md`) inheritance                                            |
| Spacing                    | Tailwind default + Phase 3 UI-SPEC inheritance                                                                           |
| Typography                 | `apps/frontend/src/globals.css` + Phase 3 UI-SPEC inheritance                                                            |
| Color palette              | `apps/frontend/src/globals.css` `@theme` + `:root` (flexoki tokens)                                                      |
| Accent reservation rules   | Phase 3 UI-SPEC §Color (60/30/10 discipline) + Phase 1 brand decisions                                                   |
| Direction-color discipline | New for Phase 4 — derived from "friendly companion" brand voice (no red-everywhere stress field)                         |
| Row shape                  | `apps/frontend/src/pages/dashboard/accounts-summary.tsx` + `apps/frontend/src/pages/activity/components/activity-table/` |
| Importer wizard scaffold   | `apps/frontend/src/pages/activity/import/activity-import-page.tsx` + steps + components (Q3 → A)                         |
| Category model decision    | User answer Q2 → D; rationale derived from `apps/frontend/src/lib/types/taxonomy.ts` capability survey                   |
| Duplicate review pattern   | User answer Q4 → A                                                                                                       |
