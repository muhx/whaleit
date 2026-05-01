-- Phase 4: CSV import templates (D-16/17/18).
-- D-16: user-saved only, no starter pack.
-- D-17: header_signature TEXT[] enables column-position mismatch detection.
-- D-18: globally-scoped per user (NO account_id column — same template visible across all user accounts).

CREATE TABLE transaction_csv_templates (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    mapping          JSONB NOT NULL,
    header_signature TEXT[] NOT NULL,
    created_at       TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at       TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    CONSTRAINT transaction_csv_templates_name_not_empty CHECK (length(trim(name)) > 0)
);

CREATE UNIQUE INDEX idx_transaction_csv_templates_name ON transaction_csv_templates (name);
