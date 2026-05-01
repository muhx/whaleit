-- Phase 4: Transaction Core schema (TXN-01..09).
-- Creates `transactions`, `transaction_splits`, `payee_category_memory`,
-- `v_transactions_with_running_balance`. Money columns NUMERIC(20,8) per Phase 3 D-10.

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================================
-- transactions: the ledger source of truth
-- ============================================================================
CREATE TABLE transactions (
    id                      TEXT PRIMARY KEY,
    account_id              TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,

    -- Core fields
    direction               TEXT NOT NULL CHECK (direction IN ('INCOME', 'EXPENSE', 'TRANSFER')),
    amount                  NUMERIC(20,8) NOT NULL CHECK (amount > 0),
    currency                TEXT NOT NULL,
    transaction_date        DATE NOT NULL,
    payee                   TEXT,
    notes                   TEXT,
    category_id             TEXT REFERENCES taxonomy_categories(id) ON DELETE SET NULL,
    has_splits              BOOLEAN NOT NULL DEFAULT FALSE,

    -- Multi-currency (TXN-07, D-02)
    fx_rate                 NUMERIC(20,8),
    fx_rate_source          TEXT CHECK (fx_rate_source IN ('SYSTEM', 'MANUAL_OVERRIDE')),

    -- Transfer pairing (D-01..D-05)
    transfer_group_id       TEXT,
    counterparty_account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    transfer_leg_role       TEXT CHECK (transfer_leg_role IN ('SOURCE', 'DESTINATION')),

    -- Idempotency / import (TXN-04, TXN-05)
    idempotency_key         TEXT UNIQUE,
    import_run_id           TEXT,
    source                  TEXT NOT NULL CHECK (source IN ('MANUAL', 'CSV', 'OFX', 'SYSTEM')),
    external_ref            TEXT,

    -- Audit
    is_system_generated     BOOLEAN NOT NULL DEFAULT FALSE,
    is_user_modified        BOOLEAN NOT NULL DEFAULT FALSE,
    category_source         TEXT CHECK (category_source IN ('USER', 'MEMORY', 'IMPORT')),

    created_at              TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at              TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),

    -- Constraints
    CONSTRAINT transfer_must_have_group_id CHECK (
        (direction = 'TRANSFER' AND transfer_group_id IS NOT NULL)
        OR (direction != 'TRANSFER' AND transfer_group_id IS NULL)
    ),
    CONSTRAINT non_transfer_must_have_payee CHECK (
        direction = 'TRANSFER' OR payee IS NOT NULL
    ),
    CONSTRAINT counterparty_only_for_transfer CHECK (
        (direction = 'TRANSFER' AND counterparty_account_id IS NOT NULL)
        OR (direction != 'TRANSFER' AND counterparty_account_id IS NULL)
    ),
    CONSTRAINT transfer_leg_role_pairs_with_transfer CHECK (
        (direction = 'TRANSFER' AND transfer_leg_role IS NOT NULL)
        OR (direction != 'TRANSFER' AND transfer_leg_role IS NULL)
    ),
    CONSTRAINT fx_rate_pair CHECK (
        (fx_rate IS NULL AND fx_rate_source IS NULL)
        OR (fx_rate IS NOT NULL AND fx_rate_source IS NOT NULL)
    )
);

CREATE INDEX idx_tx_account_date        ON transactions (account_id, transaction_date DESC, created_at DESC);
CREATE INDEX idx_tx_account_idempotency ON transactions (account_id, idempotency_key);
CREATE INDEX idx_tx_transfer_group      ON transactions (transfer_group_id) WHERE transfer_group_id IS NOT NULL;
CREATE INDEX idx_tx_category            ON transactions (category_id) WHERE category_id IS NOT NULL;
CREATE INDEX idx_tx_import_run          ON transactions (import_run_id) WHERE import_run_id IS NOT NULL;
CREATE INDEX idx_tx_payee_trgm          ON transactions USING gin (payee gin_trgm_ops);
CREATE INDEX idx_tx_date                ON transactions (transaction_date DESC);
CREATE INDEX idx_tx_has_splits          ON transactions (account_id, transaction_date DESC) WHERE has_splits = TRUE;

-- ============================================================================
-- transaction_splits: child of transactions (TXN-08)
-- ============================================================================
CREATE TABLE transaction_splits (
    id             TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    category_id    TEXT NOT NULL REFERENCES taxonomy_categories(id) ON DELETE RESTRICT,
    amount         NUMERIC(20,8) NOT NULL CHECK (amount > 0),
    notes          TEXT,
    sort_order     INTEGER NOT NULL DEFAULT 0,
    created_at     TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at     TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE INDEX idx_tx_splits_tx_id    ON transaction_splits (transaction_id);
CREATE INDEX idx_tx_splits_category ON transaction_splits (category_id);

-- ============================================================================
-- payee_category_memory: D-12 lookup
-- ============================================================================
CREATE TABLE payee_category_memory (
    account_id          TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    normalized_merchant TEXT NOT NULL,
    category_id         TEXT NOT NULL REFERENCES taxonomy_categories(id) ON DELETE CASCADE,
    last_seen_at        TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    seen_count          INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (account_id, normalized_merchant)
);

CREATE INDEX idx_payee_mem_last_seen ON payee_category_memory (last_seen_at DESC);

-- ============================================================================
-- v_transactions_with_running_balance: window-function VIEW (TXN-09)
-- ============================================================================
CREATE VIEW v_transactions_with_running_balance AS
SELECT
    t.*,
    SUM(
        CASE
            WHEN t.direction = 'INCOME' THEN t.amount
            WHEN t.direction = 'EXPENSE' THEN -t.amount
            WHEN t.direction = 'TRANSFER' AND t.transfer_leg_role = 'DESTINATION' THEN t.amount
            WHEN t.direction = 'TRANSFER' AND t.transfer_leg_role = 'SOURCE' THEN -t.amount
            ELSE 0
        END
    ) OVER (
        PARTITION BY t.account_id
        ORDER BY t.transaction_date ASC, t.created_at ASC
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    ) AS running_balance
FROM transactions t;

-- ============================================================================
-- Seed: "Transaction Categories" system taxonomy
-- ============================================================================
INSERT INTO taxonomies (id, name, color, description, is_system, is_single_select, sort_order, created_at, updated_at)
VALUES (
    'sys_taxonomy_transaction_categories',
    'Transaction Categories',
    '#8abceb',
    'System-managed transaction categories',
    TRUE,
    TRUE,
    100,
    NOW() AT TIME ZONE 'utc',
    NOW() AT TIME ZONE 'utc'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO taxonomy_categories (id, taxonomy_id, parent_id, name, key, color, description, sort_order, created_at, updated_at)
VALUES
    ('cat_income',        'sys_taxonomy_transaction_categories', NULL, 'Income',        'income',        '#36b81e', NULL,  0, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_dining',        'sys_taxonomy_transaction_categories', NULL, 'Dining',        'dining',        '#f4a06b', NULL,  1, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_entertainment', 'sys_taxonomy_transaction_categories', NULL, 'Entertainment', 'entertainment', '#cba5e1', NULL,  2, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_groceries',     'sys_taxonomy_transaction_categories', NULL, 'Groceries',     'groceries',     '#a29c8a', NULL,  3, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_healthcare',    'sys_taxonomy_transaction_categories', NULL, 'Healthcare',    'healthcare',    '#73d7e6', NULL,  4, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_housing',       'sys_taxonomy_transaction_categories', NULL, 'Housing',       'housing',       '#85b4d6', NULL,  5, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_shopping',      'sys_taxonomy_transaction_categories', NULL, 'Shopping',      'shopping',      '#e6a8c8', NULL,  6, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_transport',     'sys_taxonomy_transaction_categories', NULL, 'Transport',     'transport',     '#f5c873', NULL,  7, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_utilities',     'sys_taxonomy_transaction_categories', NULL, 'Utilities',     'utilities',     '#f5d770', NULL,  8, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_uncategorized', 'sys_taxonomy_transaction_categories', NULL, 'Uncategorized', 'uncategorized', '#b8b3a8', NULL, 99, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc')
ON CONFLICT (id) DO NOTHING;
