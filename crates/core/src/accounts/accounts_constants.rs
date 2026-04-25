/// Default account type for new accounts
pub const DEFAULT_ACCOUNT_TYPE: &str = "SECURITIES";

/// Account type constants
pub mod account_types {
    pub const SECURITIES: &str = "SECURITIES";
    pub const CASH: &str = "CASH";
    pub const CRYPTOCURRENCY: &str = "CRYPTOCURRENCY";
    pub const CHECKING: &str = "CHECKING";
    pub const SAVINGS: &str = "SAVINGS";
    pub const CREDIT_CARD: &str = "CREDIT_CARD";
    pub const LOAN: &str = "LOAN";
}

/// Returns the default group name for a given account type.
///
/// # Arguments
/// * `account_type` - The account type string (e.g., "SECURITIES", "CASH")
///
/// # Returns
/// The default group name for the account type
pub fn default_group_for_account_type(account_type: &str) -> &'static str {
    match account_type {
        account_types::SECURITIES => "Investments",
        account_types::CASH => "Cash",
        account_types::CRYPTOCURRENCY => "Crypto",
        account_types::CHECKING | account_types::SAVINGS => "Banking",
        account_types::CREDIT_CARD => "Credit Cards",
        account_types::LOAN => "Loans",
        _ => "Investments",
    }
}

/// Asset / liability / investment classification for an account type.
/// Used by net-worth and reporting consumers (Phase 6) to flip the sign
/// on liability balances. Frontend mirrors this in `apps/frontend/src/lib/constants.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    Asset,
    Liability,
    Investment,
}

/// Returns the AccountKind classification for a given account_type string.
///
/// - CHECKING / SAVINGS / CASH -> Asset
/// - CREDIT_CARD / LOAN -> Liability
/// - SECURITIES / CRYPTOCURRENCY -> Investment
/// - Unknown -> Asset (conservative default for forward-compat)
pub fn account_kind(account_type: &str) -> AccountKind {
    match account_type {
        account_types::CHECKING | account_types::SAVINGS | account_types::CASH => {
            AccountKind::Asset
        }
        account_types::CREDIT_CARD | account_types::LOAN => AccountKind::Liability,
        account_types::SECURITIES | account_types::CRYPTOCURRENCY => AccountKind::Investment,
        _ => AccountKind::Asset,
    }
}
