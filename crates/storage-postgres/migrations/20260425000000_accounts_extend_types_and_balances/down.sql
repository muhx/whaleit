ALTER TABLE accounts
    DROP COLUMN IF EXISTS institution,
    DROP COLUMN IF EXISTS opening_balance,
    DROP COLUMN IF EXISTS current_balance,
    DROP COLUMN IF EXISTS balance_updated_at,
    DROP COLUMN IF EXISTS credit_limit,
    DROP COLUMN IF EXISTS statement_cycle_day,
    DROP COLUMN IF EXISTS statement_balance,
    DROP COLUMN IF EXISTS minimum_payment,
    DROP COLUMN IF EXISTS statement_due_date,
    DROP COLUMN IF EXISTS reward_points_balance,
    DROP COLUMN IF EXISTS cashback_balance;
