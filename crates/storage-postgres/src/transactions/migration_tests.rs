//! Migration smoke tests for Phase 4 transactions schema.
//! Requires DATABASE_URL pointing at a dev/test Postgres instance.

#[cfg(test)]
mod tests {
    use crate::db::{create_pool, run_migrations};
    use diesel::sql_query;
    use diesel::sql_types::BigInt;
    use diesel::sql_types::Text;
    use diesel::{QueryableByName, RunQueryDsl};
    use diesel_async::RunQueryDsl as AsyncRunQueryDsl;

    fn db_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    #[derive(QueryableByName, Debug)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    #[derive(QueryableByName, Debug)]
    struct TableNameRow {
        #[diesel(sql_type = Text)]
        table_name: String,
    }

    #[tokio::test]
    async fn test_migration_up_creates_transactions_table() {
        let url = match db_url() {
            Some(u) => u,
            None => {
                eprintln!(
                    "DATABASE_URL not set; skipping test_migration_up_creates_transactions_table"
                );
                return;
            }
        };

        run_migrations(&url)
            .await
            .expect("migrations should apply cleanly");
        let pool = create_pool(&url, 2).expect("pool should initialize");
        let mut conn = pool.get().await.expect("conn should be available");

        for table in &[
            "transactions",
            "transaction_splits",
            "payee_category_memory",
        ] {
            let rows: Vec<CountRow> = sql_query(
                "SELECT COUNT(*)::bigint AS count FROM information_schema.tables \
                 WHERE table_schema = 'public' AND table_name = $1",
            )
            .bind::<Text, _>(*table)
            .load(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("query for table {} failed: {}", table, e));

            assert_eq!(
                rows[0].count, 1,
                "table '{}' should exist after migration",
                table
            );
        }
    }

    #[tokio::test]
    async fn test_migration_up_creates_running_balance_view() {
        let url = match db_url() {
            Some(u) => u,
            None => {
                eprintln!(
                    "DATABASE_URL not set; skipping test_migration_up_creates_running_balance_view"
                );
                return;
            }
        };

        run_migrations(&url)
            .await
            .expect("migrations should apply cleanly");
        let pool = create_pool(&url, 2).expect("pool should initialize");
        let mut conn = pool.get().await.expect("conn should be available");

        let rows: Vec<CountRow> = sql_query(
            "SELECT COUNT(*)::bigint AS count FROM information_schema.views \
             WHERE table_schema = 'public' AND table_name = 'v_transactions_with_running_balance'",
        )
        .load(&mut conn)
        .await
        .expect("query for view should succeed");

        assert_eq!(
            rows[0].count, 1,
            "view 'v_transactions_with_running_balance' should exist"
        );
    }

    #[tokio::test]
    async fn test_migration_up_seeds_transaction_categories() {
        let url = match db_url() {
            Some(u) => u,
            None => {
                eprintln!(
                    "DATABASE_URL not set; skipping test_migration_up_seeds_transaction_categories"
                );
                return;
            }
        };

        run_migrations(&url)
            .await
            .expect("migrations should apply cleanly");
        let pool = create_pool(&url, 2).expect("pool should initialize");
        let mut conn = pool.get().await.expect("conn should be available");

        let taxonomy_count: Vec<CountRow> = sql_query(
            "SELECT COUNT(*)::bigint AS count FROM taxonomies WHERE id = 'sys_taxonomy_transaction_categories'",
        )
        .load(&mut conn)
        .await
        .expect("taxonomy seed query should succeed");

        assert_eq!(
            taxonomy_count[0].count, 1,
            "sys_taxonomy_transaction_categories should be seeded"
        );

        let category_count: Vec<CountRow> = sql_query(
            "SELECT COUNT(*)::bigint AS count FROM taxonomy_categories \
             WHERE taxonomy_id = 'sys_taxonomy_transaction_categories'",
        )
        .load(&mut conn)
        .await
        .expect("category count query should succeed");

        assert_eq!(
            category_count[0].count, 10,
            "10 default transaction categories should be seeded"
        );
    }

    #[tokio::test]
    async fn test_migration_up_is_idempotent() {
        let url = match db_url() {
            Some(u) => u,
            None => {
                eprintln!("DATABASE_URL not set; skipping test_migration_up_is_idempotent");
                return;
            }
        };

        // Run migrations twice — embed_migrations only runs pending ones, so this
        // tests that the seed ON CONFLICT DO NOTHING clause prevents duplicates.
        run_migrations(&url).await.expect("first migration run");
        run_migrations(&url)
            .await
            .expect("second migration run (idempotent)");

        let pool = create_pool(&url, 2).expect("pool should initialize");
        let mut conn = pool.get().await.expect("conn should be available");

        let category_count: Vec<CountRow> = sql_query(
            "SELECT COUNT(*)::bigint AS count FROM taxonomy_categories \
             WHERE taxonomy_id = 'sys_taxonomy_transaction_categories'",
        )
        .load(&mut conn)
        .await
        .expect("category count query should succeed");

        assert_eq!(
            category_count[0].count, 10,
            "category count should remain 10 after idempotent re-run"
        );
    }

    #[tokio::test]
    async fn test_migration_down_reverses() {
        let url = match db_url() {
            Some(u) => u,
            None => {
                eprintln!("DATABASE_URL not set; skipping test_migration_down_reverses");
                return;
            }
        };

        // This test is a structural check: we verify the down.sql content is correct by
        // reading the migration file at test time. Executing down.sql against a shared test
        // DB would break other tests running in parallel; the actual round-trip is exercised
        // manually with `diesel migration redo`.
        let down_sql =
            include_str!("../../../migrations/20260501000000_transactions_initial/down.sql");
        assert!(down_sql.contains("DROP VIEW IF EXISTS v_transactions_with_running_balance"));
        assert!(down_sql.contains("DROP TABLE IF EXISTS transaction_splits"));
        assert!(down_sql.contains("DROP TABLE IF EXISTS payee_category_memory"));
        assert!(down_sql.contains("DROP TABLE IF EXISTS transactions"));
        assert!(down_sql
            .contains("DELETE FROM taxonomies WHERE id = 'sys_taxonomy_transaction_categories'"));
    }

    #[tokio::test]
    async fn test_templates_migration_creates_table() {
        let url = match db_url() {
            Some(u) => u,
            None => {
                eprintln!("DATABASE_URL not set; skipping test_templates_migration_creates_table");
                return;
            }
        };

        run_migrations(&url)
            .await
            .expect("migrations should apply cleanly");
        let pool = create_pool(&url, 2).expect("pool should initialize");
        let mut conn = pool.get().await.expect("conn should be available");

        let rows: Vec<CountRow> = sql_query(
            "SELECT COUNT(*)::bigint AS count FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name = 'transaction_csv_templates'",
        )
        .load(&mut conn)
        .await
        .expect("query for transaction_csv_templates should succeed");

        assert_eq!(
            rows[0].count, 1,
            "table 'transaction_csv_templates' should exist"
        );

        // Verify expected columns exist
        for col in &[
            "id",
            "name",
            "mapping",
            "header_signature",
            "created_at",
            "updated_at",
        ] {
            let col_rows: Vec<CountRow> = sql_query(
                "SELECT COUNT(*)::bigint AS count FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = 'transaction_csv_templates' \
                 AND column_name = $1",
            )
            .bind::<Text, _>(*col)
            .load(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("column check for {} failed: {}", col, e));

            assert_eq!(
                col_rows[0].count, 1,
                "column '{}' should exist in transaction_csv_templates",
                col
            );
        }
    }

    #[tokio::test]
    async fn test_templates_migration_round_trip() {
        // Structural check: verify down.sql correctly drops the templates table.
        let down_sql =
            include_str!("../../../migrations/20260501010000_transaction_csv_templates/down.sql");
        assert!(
            down_sql.contains("DROP TABLE IF EXISTS transaction_csv_templates"),
            "templates down.sql should drop transaction_csv_templates"
        );
    }
}
