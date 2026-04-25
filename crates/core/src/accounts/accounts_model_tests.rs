//! Tests for account domain models including TrackingMode.

#[cfg(test)]
mod tests {
    use crate::accounts::{
        account_kind, default_group_for_account_type, Account, AccountKind, NewAccount,
        TrackingMode,
    };
    use rust_decimal_macros::dec;

    // ==================== TrackingMode Serialization Tests ====================

    #[test]
    fn test_tracking_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&TrackingMode::Transactions).unwrap(),
            "\"TRANSACTIONS\""
        );
        assert_eq!(
            serde_json::to_string(&TrackingMode::Holdings).unwrap(),
            "\"HOLDINGS\""
        );
        assert_eq!(
            serde_json::to_string(&TrackingMode::NotSet).unwrap(),
            "\"NOT_SET\""
        );
    }

    #[test]
    fn test_tracking_mode_deserialization() {
        assert_eq!(
            serde_json::from_str::<TrackingMode>("\"TRANSACTIONS\"").unwrap(),
            TrackingMode::Transactions
        );
        assert_eq!(
            serde_json::from_str::<TrackingMode>("\"HOLDINGS\"").unwrap(),
            TrackingMode::Holdings
        );
        assert_eq!(
            serde_json::from_str::<TrackingMode>("\"NOT_SET\"").unwrap(),
            TrackingMode::NotSet
        );
    }

    #[test]
    fn test_tracking_mode_default() {
        let mode = TrackingMode::default();
        assert_eq!(mode, TrackingMode::NotSet);
    }

    // ==================== Account tracking_mode Field Tests ====================

    #[test]
    fn test_account_tracking_mode_default() {
        let account = Account::default();
        assert_eq!(account.tracking_mode, TrackingMode::NotSet);
    }

    #[test]
    fn test_account_tracking_mode_transactions() {
        let account = create_test_account(TrackingMode::Transactions);
        assert_eq!(account.tracking_mode, TrackingMode::Transactions);
    }

    #[test]
    fn test_account_tracking_mode_holdings() {
        let account = create_test_account(TrackingMode::Holdings);
        assert_eq!(account.tracking_mode, TrackingMode::Holdings);
    }

    #[test]
    fn test_account_is_archived_default() {
        let account = Account::default();
        assert!(!account.is_archived);
    }

    // ==================== Helper Functions ====================

    fn create_test_account(tracking_mode: TrackingMode) -> Account {
        Account {
            id: "test-account-id".to_string(),
            name: "Test Account".to_string(),
            account_type: "SECURITIES".to_string(),
            currency: "USD".to_string(),
            is_active: true,
            tracking_mode,
            ..Default::default()
        }
    }

    // ==================== Wave 0 — AccountKind / default_group / validate ====================

    fn new_account_base(account_type: &str) -> NewAccount {
        NewAccount {
            id: None,
            name: "Test".to_string(),
            account_type: account_type.to_string(),
            group: None,
            currency: "USD".to_string(),
            is_default: false,
            is_active: true,
            platform_id: None,
            account_number: None,
            meta: None,
            provider: None,
            provider_account_id: None,
            is_archived: false,
            tracking_mode: TrackingMode::Transactions,
            institution: Some("Test Bank".to_string()),
            opening_balance: Some(dec!(0)),
            current_balance: None,
            balance_updated_at: None,
            credit_limit: None,
            statement_cycle_day: None,
            statement_balance: None,
            minimum_payment: None,
            statement_due_date: None,
            reward_points_balance: None,
            cashback_balance: None,
        }
    }

    #[test]
    fn test_account_kind() {
        assert_eq!(account_kind("CHECKING"), AccountKind::Asset);
        assert_eq!(account_kind("SAVINGS"), AccountKind::Asset);
        assert_eq!(account_kind("CASH"), AccountKind::Asset);
        assert_eq!(account_kind("CREDIT_CARD"), AccountKind::Liability);
        assert_eq!(account_kind("LOAN"), AccountKind::Liability);
        assert_eq!(account_kind("SECURITIES"), AccountKind::Investment);
        assert_eq!(account_kind("CRYPTOCURRENCY"), AccountKind::Investment);
        assert_eq!(account_kind("UNKNOWN"), AccountKind::Asset);
    }

    #[test]
    fn test_default_group_for_new_types() {
        assert_eq!(default_group_for_account_type("CHECKING"), "Banking");
        assert_eq!(default_group_for_account_type("SAVINGS"), "Banking");
        assert_eq!(
            default_group_for_account_type("CREDIT_CARD"),
            "Credit Cards"
        );
        assert_eq!(default_group_for_account_type("LOAN"), "Loans");
        assert_eq!(default_group_for_account_type("SECURITIES"), "Investments");
        assert_eq!(default_group_for_account_type("CASH"), "Cash");
        assert_eq!(default_group_for_account_type("CRYPTOCURRENCY"), "Crypto");
    }

    #[test]
    fn test_new_account_validate_bank() {
        let bank = new_account_base("CHECKING");
        assert!(bank.validate().is_ok());

        let mut no_opening = new_account_base("CHECKING");
        no_opening.opening_balance = None;
        assert!(no_opening.validate().is_err());

        let mut neg_opening = new_account_base("CHECKING");
        neg_opening.opening_balance = Some(dec!(-1));
        assert!(neg_opening.validate().is_err());
    }

    #[test]
    fn test_new_account_validate_credit_card() {
        let mut cc = new_account_base("CREDIT_CARD");
        cc.credit_limit = Some(dec!(5000));
        cc.statement_cycle_day = Some(15);
        assert!(
            cc.validate().is_ok(),
            "valid CC should pass: {:?}",
            cc.validate()
        );
    }

    #[test]
    fn test_new_account_validate_credit_card_rejects_invalid() {
        let base = || {
            let mut cc = new_account_base("CREDIT_CARD");
            cc.credit_limit = Some(dec!(5000));
            cc.statement_cycle_day = Some(15);
            cc
        };

        let mut no_limit = base();
        no_limit.credit_limit = None;
        assert!(no_limit.validate().is_err());

        let mut zero_limit = base();
        zero_limit.credit_limit = Some(dec!(0));
        assert!(zero_limit.validate().is_err());

        let mut bad_cycle = base();
        bad_cycle.statement_cycle_day = Some(32);
        assert!(bad_cycle.validate().is_err());

        let mut zero_cycle = base();
        zero_cycle.statement_cycle_day = Some(0);
        assert!(zero_cycle.validate().is_err());
    }

    #[test]
    fn test_new_account_validate_non_cc_rejects_cc_fields() {
        let mut bank = new_account_base("CHECKING");
        bank.credit_limit = Some(dec!(5000));
        let err = bank
            .validate()
            .expect_err("non-CC with CC field should fail");
        assert!(
            format!("{err}").contains("Credit card fields are only valid for CREDIT_CARD accounts")
        );
    }
}
