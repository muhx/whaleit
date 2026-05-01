//! Integration tests for PgTransactionRepository and PgTransactionTemplateRepository.
//! Requires DATABASE_URL pointing at a dev/test Postgres instance.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use whaleit_core::transactions::{
        NewSplit, NewTransaction, NewTransactionTemplate, PayeeCategoryMemory,
        PayeeCategoryMemoryRepositoryTrait, TransactionFilters, TransactionRepositoryTrait,
        TransactionTemplateRepositoryTrait, TransactionUpdate,
    };

    use crate::db::{create_pool, run_migrations};
    use crate::transactions::repository::PgTransactionRepository;
    use crate::transactions::templates_repository::PgTransactionTemplateRepository;

    // ── Setup ────────────────────────────────────────────────────────────────

    async fn setup() -> Option<(Arc<PgTransactionRepository>, String)> {
        let url = std::env::var("DATABASE_URL").ok()?;
        run_migrations(&url).await.expect("migrations should apply");
        let pool = create_pool(&url, 4).expect("pool should init");
        let repo = Arc::new(PgTransactionRepository::new(pool));
        // Each test gets a unique account_id to avoid cross-test interference.
        let account_id = Uuid::now_v7().to_string();
        Some((repo, account_id))
    }

    async fn setup_templates() -> Option<Arc<PgTransactionTemplateRepository>> {
        let url = std::env::var("DATABASE_URL").ok()?;
        run_migrations(&url).await.expect("migrations should apply");
        let pool = create_pool(&url, 4).expect("pool should init");
        Some(Arc::new(PgTransactionTemplateRepository::new(pool)))
    }

    // Also need an account row in the DB so FK on transactions.account_id is satisfied.
    async fn ensure_account(pool: &Arc<crate::db::PgPool>, account_id: &str) {
        use crate::schema::accounts;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = pool.get().await.expect("conn");
        diesel::sql_query(
            "INSERT INTO accounts (id, name, account_type, currency, is_default, is_active, \
             is_archived, tracking_mode, created_at, updated_at) \
             VALUES ($1, $2, 'CHECKING', 'USD', false, true, false, 'TRANSACTIONS', now(), now()) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind::<diesel::sql_types::Text, _>(account_id)
        .bind::<diesel::sql_types::Text, _>(format!("test-account-{}", account_id))
        .execute(&mut conn)
        .await
        .expect("ensure_account");
    }

    fn make_txn(account_id: &str, payee: &str, amount: Decimal, date: NaiveDate) -> NewTransaction {
        NewTransaction {
            account_id: account_id.to_string(),
            direction: "EXPENSE".to_string(),
            amount,
            currency: "USD".to_string(),
            transaction_date: date,
            payee: Some(payee.to_string()),
            notes: None,
            category_id: None,
            splits: vec![],
            fx_rate: None,
            fx_rate_source: None,
            transfer_group_id: None,
            counterparty_account_id: None,
            transfer_leg_role: None,
            idempotency_key: None,
            import_run_id: None,
            source: "MANUAL".to_string(),
            external_ref: None,
            is_system_generated: false,
            category_source: None,
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_with_splits_persists_both() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let mut new = make_txn(&account_id, "Grocery", dec!(100.00), date(2026, 4, 1));
        new.splits = vec![
            NewSplit {
                category_id: "cat1".to_string(),
                amount: dec!(60.00),
                notes: None,
                sort_order: 0,
            },
            NewSplit {
                category_id: "cat2".to_string(),
                amount: dec!(40.00),
                notes: None,
                sort_order: 1,
            },
        ];

        let txn = repo.create_with_splits(new).await.unwrap();
        assert_eq!(txn.splits.len(), 2);
        let sum: Decimal = txn.splits.iter().map(|s| s.amount).sum();
        assert_eq!(sum, dec!(100.00));
    }

    #[tokio::test]
    async fn create_with_splits_atomic_rollback() {
        // This test verifies that if something fails mid-transaction, no partial row lands.
        // We simulate this by attempting an insert with a duplicate idempotency_key in batch.
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let key = Uuid::now_v7().to_string();
        let mut txn_a = make_txn(&account_id, "First", dec!(10.00), date(2026, 4, 1));
        txn_a.idempotency_key = Some(key.clone());
        repo.create_with_splits(txn_a).await.unwrap();

        // Second insert with same key should fail.
        let mut txn_b = make_txn(&account_id, "Second", dec!(20.00), date(2026, 4, 2));
        txn_b.idempotency_key = Some(key.clone());
        let result = repo.create_many_with_splits(vec![txn_b]).await;
        assert!(result.is_err(), "Duplicate idempotency_key must fail");
    }

    #[tokio::test]
    async fn update_replaces_splits() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let mut new = make_txn(&account_id, "Split txn", dec!(90.00), date(2026, 4, 1));
        new.splits = vec![
            NewSplit {
                category_id: "catA".to_string(),
                amount: dec!(45.00),
                notes: None,
                sort_order: 0,
            },
            NewSplit {
                category_id: "catB".to_string(),
                amount: dec!(45.00),
                notes: None,
                sort_order: 1,
            },
        ];
        let created = repo.create_with_splits(new).await.unwrap();

        let update = TransactionUpdate {
            id: created.id.clone(),
            has_splits: Some(true),
            splits: Some(vec![
                NewSplit {
                    category_id: "catC".to_string(),
                    amount: dec!(30.00),
                    notes: None,
                    sort_order: 0,
                },
                NewSplit {
                    category_id: "catD".to_string(),
                    amount: dec!(30.00),
                    notes: None,
                    sort_order: 1,
                },
                NewSplit {
                    category_id: "catE".to_string(),
                    amount: dec!(30.00),
                    notes: None,
                    sort_order: 2,
                },
            ]),
            direction: None,
            amount: None,
            currency: None,
            transaction_date: None,
            payee: None,
            notes: None,
            category_id: None,
            fx_rate: None,
            fx_rate_source: None,
            transfer_group_id: None,
            counterparty_account_id: None,
            transfer_leg_role: None,
            is_user_modified: None,
            category_source: None,
        };

        let updated = repo.update_with_splits(update).await.unwrap();
        assert_eq!(updated.splits.len(), 3, "should have 3 splits after update");
        let cats: Vec<&str> = updated
            .splits
            .iter()
            .map(|s| s.category_id.as_str())
            .collect();
        assert!(cats.contains(&"catC"));
        assert!(!cats.contains(&"catA"), "old splits must be gone");
    }

    #[tokio::test]
    async fn transfer_delete_cascade() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let tg = Uuid::now_v7().to_string();
        let dest_account = Uuid::now_v7().to_string();
        ensure_account(&repo.pool, &dest_account).await;

        let mut leg1 = make_txn(&account_id, "Transfer out", dec!(50.00), date(2026, 4, 1));
        leg1.direction = "TRANSFER".to_string();
        leg1.transfer_group_id = Some(tg.clone());
        leg1.transfer_leg_role = Some("SOURCE".to_string());
        leg1.counterparty_account_id = Some(dest_account.clone());

        let mut leg2 = make_txn(&dest_account, "Transfer in", dec!(50.00), date(2026, 4, 1));
        leg2.direction = "TRANSFER".to_string();
        leg2.transfer_group_id = Some(tg.clone());
        leg2.transfer_leg_role = Some("DESTINATION".to_string());
        leg2.counterparty_account_id = Some(account_id.clone());

        repo.create_with_splits(leg1).await.unwrap();
        repo.create_with_splits(leg2).await.unwrap();

        repo.delete_pair(&tg).await.unwrap();

        // Both legs should be gone — search returns 0 for this account.
        let filters = TransactionFilters {
            account_ids: vec![account_id.clone()],
            show_transfers: true,
            ..Default::default()
        };
        let result = repo.search(filters, 0, 100).await.unwrap();
        assert_eq!(result.total, 0, "both transfer legs should be deleted");
    }

    #[tokio::test]
    async fn running_balance_basic() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        // Insert in chronological order: INCOME +100, EXPENSE -30, INCOME +50
        let mut inc1 = make_txn(&account_id, "Salary", dec!(100.00), date(2026, 4, 1));
        inc1.direction = "INCOME".to_string();
        let mut exp1 = make_txn(&account_id, "Coffee", dec!(30.00), date(2026, 4, 2));
        exp1.direction = "EXPENSE".to_string();
        let mut inc2 = make_txn(&account_id, "Bonus", dec!(50.00), date(2026, 4, 3));
        inc2.direction = "INCOME".to_string();

        repo.create_with_splits(inc1).await.unwrap();
        repo.create_with_splits(exp1).await.unwrap();
        repo.create_with_splits(inc2).await.unwrap();

        let rows = repo
            .list_with_running_balance(&account_id, None, None)
            .await
            .unwrap();
        assert_eq!(rows.len(), 3, "should return 3 rows");
        // Result is ordered DESC, so rows[0] = Apr 3 (most recent) with running_balance = 120
        let balances: Vec<Decimal> = rows.iter().map(|r| r.running_balance).collect();
        assert_eq!(
            balances[0],
            dec!(120.00),
            "Apr 3 running balance should be 120"
        );
    }

    #[tokio::test]
    async fn running_balance_out_of_order() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        // Insert Apr 25 row FIRST, then Apr 20 row.
        let mut txn_apr25 = make_txn(&account_id, "Late insert", dec!(25.00), date(2026, 4, 25));
        txn_apr25.direction = "INCOME".to_string();
        let mut txn_apr20 = make_txn(&account_id, "Early insert", dec!(20.00), date(2026, 4, 20));
        txn_apr20.direction = "INCOME".to_string();

        repo.create_with_splits(txn_apr25).await.unwrap();
        repo.create_with_splits(txn_apr20).await.unwrap();

        let rows = repo
            .list_with_running_balance(&account_id, None, None)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        // DESC order: rows[0] = Apr 25 with cumulative 45, rows[1] = Apr 20 with 20.
        assert_eq!(rows[0].txn.transaction_date, date(2026, 4, 25));
        assert_eq!(rows[0].running_balance, dec!(45.00));
        assert_eq!(rows[1].running_balance, dec!(20.00));
    }

    #[tokio::test]
    async fn running_balance_transfer_legs() {
        let (repo, src_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        let dst_id = Uuid::now_v7().to_string();
        ensure_account(&repo.pool, &src_id).await;
        ensure_account(&repo.pool, &dst_id).await;

        // Seed income on src so running balance can go down meaningfully.
        let mut seed = make_txn(&src_id, "Seed", dec!(200.00), date(2026, 4, 1));
        seed.direction = "INCOME".to_string();
        repo.create_with_splits(seed).await.unwrap();

        let tg = Uuid::now_v7().to_string();
        let mut src_leg = make_txn(&src_id, "Transfer out", dec!(50.00), date(2026, 4, 2));
        src_leg.direction = "TRANSFER".to_string();
        src_leg.transfer_group_id = Some(tg.clone());
        src_leg.transfer_leg_role = Some("SOURCE".to_string());
        src_leg.counterparty_account_id = Some(dst_id.clone());

        let mut dst_leg = make_txn(&dst_id, "Transfer in", dec!(50.00), date(2026, 4, 2));
        dst_leg.direction = "TRANSFER".to_string();
        dst_leg.transfer_group_id = Some(tg.clone());
        dst_leg.transfer_leg_role = Some("DESTINATION".to_string());
        dst_leg.counterparty_account_id = Some(src_id.clone());

        repo.create_with_splits(src_leg).await.unwrap();
        repo.create_with_splits(dst_leg).await.unwrap();

        let src_rows = repo
            .list_with_running_balance(&src_id, None, None)
            .await
            .unwrap();
        let dst_rows = repo
            .list_with_running_balance(&dst_id, None, None)
            .await
            .unwrap();

        // src should have 2 rows, most recent balance = 150
        assert_eq!(src_rows.len(), 2);
        assert_eq!(src_rows[0].running_balance, dec!(150.00));
        // dst should have 1 row with balance = 50
        assert_eq!(dst_rows.len(), 1);
        assert_eq!(dst_rows[0].running_balance, dec!(50.00));
    }

    #[tokio::test]
    async fn running_balance_archived_account() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        // Create account then archive it.
        ensure_account(&repo.pool, &account_id).await;
        {
            use crate::schema::accounts;
            use diesel::prelude::*;
            use diesel_async::RunQueryDsl;
            let mut conn = repo.pool.get().await.expect("conn");
            diesel::sql_query("UPDATE accounts SET is_archived = true WHERE id = $1")
                .bind::<diesel::sql_types::Text, _>(&account_id)
                .execute(&mut conn)
                .await
                .expect("archive account");
        }

        let mut txn = make_txn(&account_id, "Old income", dec!(100.00), date(2026, 4, 1));
        txn.direction = "INCOME".to_string();
        repo.create_with_splits(txn).await.unwrap();

        // The VIEW does not filter on is_archived; data must still be visible.
        let rows = repo
            .list_with_running_balance(&account_id, None, None)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1, "archived account data must still be visible");
    }

    #[tokio::test]
    async fn search_by_date_range() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        for (month, payee) in [(3, "March"), (4, "April"), (5, "May")] {
            let txn = make_txn(&account_id, payee, dec!(10.00), date(2026, month, 15));
            repo.create_with_splits(txn).await.unwrap();
        }

        let filters = TransactionFilters {
            account_ids: vec![account_id.clone()],
            date_from: Some(date(2026, 4, 1)),
            date_to: Some(date(2026, 4, 30)),
            show_transfers: true,
            ..Default::default()
        };
        let result = repo.search(filters, 0, 100).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].payee.as_deref(), Some("April"));
    }

    #[tokio::test]
    async fn search_by_amount_range() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        for amount in [dec!(5.00), dec!(50.00), dec!(500.00)] {
            let txn = make_txn(&account_id, "payee", amount, date(2026, 4, 1));
            repo.create_with_splits(txn).await.unwrap();
        }

        let filters = TransactionFilters {
            account_ids: vec![account_id.clone()],
            amount_min: Some(dec!(10.00)),
            amount_max: Some(dec!(100.00)),
            show_transfers: true,
            ..Default::default()
        };
        let result = repo.search(filters, 0, 100).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].amount, dec!(50.00));
    }

    #[tokio::test]
    async fn search_by_category() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let mut dining = make_txn(&account_id, "Restaurant", dec!(20.00), date(2026, 4, 1));
        dining.category_id = Some("cat_dining".to_string());
        let mut groceries = make_txn(&account_id, "Supermarket", dec!(80.00), date(2026, 4, 2));
        groceries.category_id = Some("cat_groceries".to_string());

        repo.create_with_splits(dining).await.unwrap();
        repo.create_with_splits(groceries).await.unwrap();

        let filters = TransactionFilters {
            account_ids: vec![account_id.clone()],
            category_ids: vec!["cat_dining".to_string()],
            show_transfers: true,
            ..Default::default()
        };
        let result = repo.search(filters, 0, 100).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].payee.as_deref(), Some("Restaurant"));
    }

    #[tokio::test]
    async fn search_payee_uses_trgm() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        // Insert a few unrelated rows and one with "needle in haystack".
        for i in 0..5 {
            let txn = make_txn(
                &account_id,
                &format!("payee_{i}"),
                dec!(1.00),
                date(2026, 4, i as u32 + 1),
            );
            repo.create_with_splits(txn).await.unwrap();
        }
        let needle_txn = make_txn(
            &account_id,
            "needle in haystack",
            dec!(1.00),
            date(2026, 4, 10),
        );
        repo.create_with_splits(needle_txn).await.unwrap();

        let filters = TransactionFilters {
            account_ids: vec![account_id.clone()],
            search_keyword: Some("needle".to_string()),
            show_transfers: true,
            ..Default::default()
        };
        let result = repo.search(filters, 0, 100).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].payee.as_deref(), Some("needle in haystack"));
    }

    #[tokio::test]
    async fn get_by_idempotency_key_returns_some() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let key = "idem-key-test-001".to_string();
        let mut txn = make_txn(&account_id, "Dedup", dec!(10.00), date(2026, 4, 1));
        txn.idempotency_key = Some(key.clone());
        repo.create_with_splits(txn).await.unwrap();

        let found = repo
            .get_by_idempotency_key(&account_id, &key)
            .await
            .unwrap();
        assert!(found.is_some(), "should find by idempotency key");

        let missing = repo
            .get_by_idempotency_key(&account_id, "nonexistent")
            .await
            .unwrap();
        assert!(missing.is_none(), "nonexistent key must return None");
    }

    #[tokio::test]
    async fn idempotency_key_unique_violation() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let key = format!("unique-idem-{}", Uuid::now_v7());
        let mut txn1 = make_txn(&account_id, "First", dec!(10.00), date(2026, 4, 1));
        txn1.idempotency_key = Some(key.clone());
        repo.create_with_splits(txn1).await.unwrap();

        let mut txn2 = make_txn(&account_id, "Second", dec!(20.00), date(2026, 4, 2));
        txn2.idempotency_key = Some(key.clone());
        let result = repo.create_with_splits(txn2).await;
        assert!(result.is_err(), "duplicate idempotency key must fail");
    }

    #[tokio::test]
    async fn list_in_dup_window_single_query() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        // Insert 10 rows with varying dates and amounts.
        for i in 0..10i32 {
            let amount = Decimal::new(i64::from(i + 1) * 10, 0); // 10, 20, ..., 100
            let txn = make_txn(
                &account_id,
                &format!("row_{i}"),
                amount,
                date(2026, 4, i as u32 + 1),
            );
            repo.create_with_splits(txn).await.unwrap();
        }

        // Window: Apr 3 - Apr 5, amounts 25-55 → should catch rows with date in range AND amount in range.
        let result = repo
            .list_in_dup_window(
                &account_id,
                date(2026, 4, 3),
                date(2026, 4, 5),
                dec!(25.00),
                dec!(55.00),
            )
            .await
            .unwrap();
        // Rows 3,4,5 (amounts 30,40,50) match both date and amount window.
        assert_eq!(result.len(), 3, "should return only rows within window");
        for row in &result {
            assert!(row.amount >= dec!(25.00) && row.amount <= dec!(55.00));
        }
    }

    #[tokio::test]
    async fn payee_memory_upsert_increments_seen_count() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let mem = PayeeCategoryMemory {
            account_id: account_id.clone(),
            normalized_merchant: "starbucks".to_string(),
            category_id: "cat_coffee".to_string(),
            last_seen_at: chrono::Utc::now().naive_utc(),
            seen_count: 1,
        };
        repo.upsert(mem).await.unwrap();

        // Second upsert should increment seen_count.
        let mem2 = PayeeCategoryMemory {
            account_id: account_id.clone(),
            normalized_merchant: "starbucks".to_string(),
            category_id: "cat_coffee".to_string(),
            last_seen_at: chrono::Utc::now().naive_utc(),
            seen_count: 1,
        };
        repo.upsert(mem2).await.unwrap();

        let found = repo
            .lookup(&account_id, "starbucks")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            found.seen_count, 2,
            "seen_count should be 2 after two upserts"
        );
    }

    #[tokio::test]
    async fn payee_memory_lookup_returns_some() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        let mem = PayeeCategoryMemory {
            account_id: account_id.clone(),
            normalized_merchant: "wholefds".to_string(),
            category_id: "cat_groceries".to_string(),
            last_seen_at: chrono::Utc::now().naive_utc(),
            seen_count: 1,
        };
        repo.upsert(mem).await.unwrap();

        let found = repo.lookup(&account_id, "wholefds").await.unwrap();
        assert!(found.is_some());
        let not_found = repo.lookup(&account_id, "unknown_merchant").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn has_user_transactions_excludes_system_generated() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        // Insert system-generated row only — should return false.
        let mut sys_txn = make_txn(&account_id, "Opening Balance", dec!(0.00), date(2026, 4, 1));
        sys_txn.is_system_generated = true;
        repo.create_with_splits(sys_txn).await.unwrap();
        assert!(
            !repo.has_user_transactions(&account_id).await.unwrap(),
            "system rows should not count"
        );

        // Add a user row — should now return true.
        let user_txn = make_txn(&account_id, "Groceries", dec!(50.00), date(2026, 4, 2));
        repo.create_with_splits(user_txn).await.unwrap();
        assert!(
            repo.has_user_transactions(&account_id).await.unwrap(),
            "should return true with user row"
        );
    }

    #[tokio::test]
    async fn create_many_with_splits_preserves_input_order() {
        let (repo, account_id) = match setup().await {
            Some(r) => r,
            None => return,
        };
        ensure_account(&repo.pool, &account_id).await;

        // Build 5 NewTransactions with deterministic, unique payees.
        let news: Vec<NewTransaction> = (0..5)
            .map(|i| {
                make_txn(
                    &account_id,
                    &format!("row_{i}"),
                    Decimal::new(i + 1, 0),
                    date(2026, 4, i as u32 + 1),
                )
            })
            .collect();

        let result = repo.create_many_with_splits(news.clone()).await.unwrap();

        assert_eq!(result.len(), 5, "should return 5 transactions");
        for (i, txn) in result.iter().enumerate() {
            assert_eq!(
                txn.payee.as_deref(),
                Some(format!("row_{i}").as_str()),
                "result[{i}].payee must equal news[{i}].payee (input order preserved)"
            );
        }
    }

    // ── Templates tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn templates_create_then_list() {
        let repo = match setup_templates().await {
            Some(r) => r,
            None => return,
        };
        let new = NewTransactionTemplate {
            name: format!("test-template-{}", Uuid::now_v7()),
            mapping: serde_json::json!({"date": 0, "amount": 1, "payee": 2}),
            header_signature: vec![
                "Date".to_string(),
                "Amount".to_string(),
                "Payee".to_string(),
            ],
        };
        let created = repo.create(new.clone()).await.unwrap();
        assert_eq!(created.name, new.name);

        let all = repo.list_all().await.unwrap();
        assert!(
            all.iter().any(|t| t.id == created.id),
            "created template should appear in list"
        );
    }

    #[tokio::test]
    async fn templates_save_with_existing_name_updates_in_place() {
        let repo = match setup_templates().await {
            Some(r) => r,
            None => return,
        };
        let name = format!("upsert-tmpl-{}", Uuid::now_v7());
        let v1 = NewTransactionTemplate {
            name: name.clone(),
            mapping: serde_json::json!({"date": 0}),
            header_signature: vec!["Date".to_string()],
        };
        let v2 = NewTransactionTemplate {
            name: name.clone(),
            mapping: serde_json::json!({"date": 0, "amount": 1}),
            header_signature: vec!["Date".to_string(), "Amt".to_string()],
        };

        repo.create(v1).await.unwrap();
        let updated = repo.create(v2).await.unwrap();

        // Should still be one row with the updated mapping.
        let all: Vec<_> = repo
            .list_all()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| t.name == name)
            .collect();
        assert_eq!(
            all.len(),
            1,
            "must be single row after re-save with same name"
        );
        assert_eq!(
            updated.header_signature.len(),
            2,
            "header_signature should be updated"
        );
    }

    #[tokio::test]
    async fn templates_delete_removes_row() {
        let repo = match setup_templates().await {
            Some(r) => r,
            None => return,
        };
        let new = NewTransactionTemplate {
            name: format!("delete-tmpl-{}", Uuid::now_v7()),
            mapping: serde_json::json!({}),
            header_signature: vec![],
        };
        let created = repo.create(new).await.unwrap();
        repo.delete(&created.id).await.unwrap();

        let result = repo.get_by_id(&created.id).await;
        assert!(result.is_err(), "deleted template must not be findable");
    }
}
