-- Phase 3: Bank accounts, credit cards, and balance fields.
-- Adds 11 nullable columns to accounts. Money columns use NUMERIC(20,8) per
-- decision D-10 (resolved 2026-04-25). This diverges from the existing TEXT
-- pattern used in 20260101000000_initial_schema for money columns; only NEW
-- Phase 3 columns use NUMERIC. Existing TEXT-stored money columns are
-- unchanged.

ALTER TABLE accounts
    ADD COLUMN institution TEXT,
    ADD COLUMN opening_balance NUMERIC(20,8),
    ADD COLUMN current_balance NUMERIC(20,8),
    ADD COLUMN balance_updated_at TIMESTAMP,
    ADD COLUMN credit_limit NUMERIC(20,8),
    ADD COLUMN statement_cycle_day SMALLINT
        CHECK (statement_cycle_day IS NULL OR (statement_cycle_day BETWEEN 1 AND 31)),
    ADD COLUMN statement_balance NUMERIC(20,8),
    ADD COLUMN minimum_payment NUMERIC(20,8),
    ADD COLUMN statement_due_date DATE,
    ADD COLUMN reward_points_balance INTEGER
        CHECK (reward_points_balance IS NULL OR reward_points_balance >= 0),
    ADD COLUMN cashback_balance NUMERIC(20,8);
