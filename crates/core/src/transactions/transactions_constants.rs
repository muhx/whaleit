//! Phase 4 string constants. Mirror crates/core/src/accounts/accounts_constants.rs.

pub mod direction {
    pub const INCOME: &str = "INCOME";
    pub const EXPENSE: &str = "EXPENSE";
    pub const TRANSFER: &str = "TRANSFER";
    pub const ALL: [&str; 3] = [INCOME, EXPENSE, TRANSFER];
}

pub mod source {
    pub const MANUAL: &str = "MANUAL";
    pub const CSV: &str = "CSV";
    pub const OFX: &str = "OFX";
    pub const SYSTEM: &str = "SYSTEM";
}

pub mod transfer_leg_role {
    pub const SOURCE: &str = "SOURCE";
    pub const DESTINATION: &str = "DESTINATION";
}

pub mod fx_rate_source {
    pub const SYSTEM: &str = "SYSTEM";
    pub const MANUAL_OVERRIDE: &str = "MANUAL_OVERRIDE";
}

pub mod category_source {
    pub const USER: &str = "USER";
    pub const MEMORY: &str = "MEMORY";
    pub const IMPORT: &str = "IMPORT";
}

/// Stable seeded category IDs (must match migration 20260501000000_transactions_initial seed)
pub mod seeded_categories {
    pub const SYSTEM_TAXONOMY_ID: &str = "sys_taxonomy_transaction_categories";
    pub const INCOME: &str = "cat_income";
    pub const DINING: &str = "cat_dining";
    pub const ENTERTAINMENT: &str = "cat_entertainment";
    pub const GROCERIES: &str = "cat_groceries";
    pub const HEALTHCARE: &str = "cat_healthcare";
    pub const HOUSING: &str = "cat_housing";
    pub const SHOPPING: &str = "cat_shopping";
    pub const TRANSPORT: &str = "cat_transport";
    pub const UTILITIES: &str = "cat_utilities";
    pub const UNCATEGORIZED: &str = "cat_uncategorized";
}
