//! PG integration tests for the accounts repository (Phase 3 fields).
//! Requires DATABASE_URL.

#[cfg(test)]
mod tests {
    use crate::accounts::repository::PgAccountRepository;
    use crate::db::{create_pool, run_migrations};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::sync::Arc;
    use whaleit_core::accounts::{AccountRepositoryTrait, AccountUpdate, NewAccount, TrackingMode};

    async fn setup() -> Option<Arc<PgAccountRepository>> {
        let url = std::env::var("DATABASE_URL").ok()?;
        run_migrations(&url).await.expect("migrations should apply");
        let pool = create_pool(&url, 4).expect("pool should init");
        Some(Arc::new(PgAccountRepository::new(pool)))
    }

    fn checking_account_input(name: &str) -> NewAccount {
        NewAccount {
            id: None,
            name: name.to_string(),
            account_type: "CHECKING".to_string(),
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
            institution: Some("Chase".to_string()),
            opening_balance: Some(dec!(0)),
            current_balance: Some(dec!(1234.56)),
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

    fn credit_card_input(name: &str) -> NewAccount {
        NewAccount {
            id: None,
            name: name.to_string(),
            account_type: "CREDIT_CARD".to_string(),
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
            institution: Some("Amex".to_string()),
            opening_balance: Some(dec!(0)),
            current_balance: Some(dec!(420.50)),
            balance_updated_at: None,
            credit_limit: Some(dec!(5000.00)),
            statement_cycle_day: Some(15),
            statement_balance: Some(dec!(420.50)),
            minimum_payment: Some(dec!(40.00)),
            statement_due_date: NaiveDate::from_ymd_opt(2026, 5, 15),
            reward_points_balance: Some(12_450),
            cashback_balance: Some(dec!(34.80)),
        }
    }

    #[tokio::test]
    async fn test_create_credit_card_round_trip() {
        let repo = match setup().await {
            Some(r) => r,
            None => return,
        };
        let created = repo.create(credit_card_input("rt-cc-1")).await.unwrap();

        assert_eq!(created.account_type, "CREDIT_CARD");
        assert_eq!(created.credit_limit, Some(dec!(5000.00)));
        assert_eq!(created.statement_cycle_day, Some(15));
        assert_eq!(created.statement_balance, Some(dec!(420.50)));
        assert_eq!(created.minimum_payment, Some(dec!(40.00)));
        assert_eq!(
            created.statement_due_date,
            NaiveDate::from_ymd_opt(2026, 5, 15)
        );
        assert_eq!(created.reward_points_balance, Some(12_450));
        assert_eq!(created.cashback_balance, Some(dec!(34.80)));
        assert_eq!(created.institution.as_deref(), Some("Amex"));
        assert_eq!(created.opening_balance, Some(dec!(0)));
        assert_eq!(created.current_balance, Some(dec!(420.50)));

        let fetched = repo.get_by_id(&created.id).await.unwrap();
        assert_eq!(fetched.credit_limit, Some(dec!(5000.00)));
    }

    #[tokio::test]
    async fn test_create_checking_round_trip() {
        let repo = match setup().await {
            Some(r) => r,
            None => return,
        };
        let created = repo
            .create(checking_account_input("rt-chk-1"))
            .await
            .unwrap();

        assert_eq!(created.account_type, "CHECKING");
        assert_eq!(created.institution.as_deref(), Some("Chase"));
        assert_eq!(created.opening_balance, Some(dec!(0)));
        assert_eq!(created.current_balance, Some(dec!(1234.56)));
        // CC-only fields must remain None on a CHECKING account.
        assert!(created.credit_limit.is_none());
        assert!(created.statement_cycle_day.is_none());
        assert!(created.statement_balance.is_none());
        assert!(created.minimum_payment.is_none());
        assert!(created.statement_due_date.is_none());
        assert!(created.reward_points_balance.is_none());
        assert!(created.cashback_balance.is_none());
    }

    #[tokio::test]
    async fn test_update_preserves_fields() {
        let repo = match setup().await {
            Some(r) => r,
            None => return,
        };
        let created = repo.create(credit_card_input("rt-update-1")).await.unwrap();

        let update = AccountUpdate {
            id: Some(created.id.clone()),
            name: "Renamed CC".to_string(),
            account_type: created.account_type.clone(),
            group: created.group.clone(),
            is_default: created.is_default,
            is_active: created.is_active,
            platform_id: created.platform_id.clone(),
            account_number: created.account_number.clone(),
            meta: created.meta.clone(),
            provider: created.provider.clone(),
            provider_account_id: created.provider_account_id.clone(),
            is_archived: Some(created.is_archived),
            tracking_mode: Some(created.tracking_mode),
            institution: created.institution.clone(),
            opening_balance: created.opening_balance,
            current_balance: created.current_balance,
            balance_updated_at: created.balance_updated_at,
            credit_limit: created.credit_limit,
            statement_cycle_day: created.statement_cycle_day,
            statement_balance: created.statement_balance,
            minimum_payment: created.minimum_payment,
            statement_due_date: created.statement_due_date,
            reward_points_balance: created.reward_points_balance,
            cashback_balance: created.cashback_balance,
        };

        let updated = repo.update(update).await.unwrap();
        assert_eq!(updated.name, "Renamed CC");
        // currency preserved (existing repository.rs:61 logic)
        assert_eq!(updated.currency, "USD");
        // CC fields round-tripped through the update path
        assert_eq!(updated.credit_limit, Some(dec!(5000.00)));
        assert_eq!(updated.statement_cycle_day, Some(15));
    }

    #[tokio::test]
    async fn test_cc_statement_roundtrip() {
        let repo = match setup().await {
            Some(r) => r,
            None => return,
        };
        let created = repo.create(credit_card_input("rt-stmt-1")).await.unwrap();
        let fetched = repo.get_by_id(&created.id).await.unwrap();
        assert_eq!(fetched.statement_balance, Some(dec!(420.50)));
        assert_eq!(fetched.minimum_payment, Some(dec!(40.00)));
        assert_eq!(
            fetched.statement_due_date,
            NaiveDate::from_ymd_opt(2026, 5, 15)
        );
        assert_eq!(fetched.statement_cycle_day, Some(15));
    }

    #[tokio::test]
    async fn test_cc_rewards_roundtrip() {
        let repo = match setup().await {
            Some(r) => r,
            None => return,
        };
        let created = repo.create(credit_card_input("rt-rew-1")).await.unwrap();
        let fetched = repo.get_by_id(&created.id).await.unwrap();
        assert_eq!(fetched.reward_points_balance, Some(12_450));
        assert_eq!(fetched.cashback_balance, Some(dec!(34.80)));
    }

    // Suppress unused-import warning when DATABASE_URL is unset (Decimal is
    // referenced only via dec!() macro expansions).
    #[allow(dead_code)]
    fn _assert_decimal_used(_: Decimal) {}
}
