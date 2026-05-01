DROP VIEW IF EXISTS v_transactions_with_running_balance;
DROP TABLE IF EXISTS transaction_splits;
DROP TABLE IF EXISTS payee_category_memory;
DROP TABLE IF EXISTS transactions;
DELETE FROM taxonomy_categories WHERE taxonomy_id = 'sys_taxonomy_transaction_categories';
DELETE FROM taxonomies WHERE id = 'sys_taxonomy_transaction_categories';
