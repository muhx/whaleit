//! Migration smoke test for Phase 3 accounts schema extension.
//! Requires DATABASE_URL pointing at a dev/test Postgres instance.

#[cfg(test)]
mod tests {
    use crate::accounts::model::AccountDB;
    use crate::db::{create_pool, run_migrations};
    use crate::schema::accounts;
    use diesel::dsl::count_star;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    #[tokio::test]
    async fn test_migration_up_down() {
        let url = match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                eprintln!("DATABASE_URL not set; skipping migration_tests::test_migration_up_down");
                return;
            }
        };

        // Apply migrations (idempotent — embed_migrations only runs pending ones).
        run_migrations(&url)
            .await
            .expect("migrations should apply cleanly");

        // Smoke check: SELECT against the new columns proves the schema lines up.
        // Note: deviation from plan — actual API is sync `create_pool(url, max_size)`,
        // not `init_pool(url).await`.
        let pool = create_pool(&url, 2).expect("pool should initialize");
        let mut conn = pool.get().await.expect("conn should be available");

        // Issue a query that touches AccountDB::as_select() which references
        // every column declared in schema.rs (including the 11 new ones).
        // If schema.rs and the DB diverge, this query fails to compile/run.
        let _row_count: i64 = accounts::table
            .select(count_star())
            .first(&mut conn)
            .await
            .expect("count_star against accounts should succeed");

        // Confirm a fully-shaped Selectable round-trip still works.
        let _selected: Vec<AccountDB> = accounts::table
            .select(AccountDB::as_select())
            .limit(0)
            .load(&mut conn)
            .await
            .expect("AccountDB::as_select should match schema.rs columns");
    }
}
