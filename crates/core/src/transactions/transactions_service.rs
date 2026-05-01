//! Transaction service implementation (Phase 4, plan 04-04).

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::Result;

use super::{
    duplicate_detector::{detect_duplicates, DuplicateMatch},
    idempotency::compute_transaction_idempotency_key,
    merchant_normalizer::normalize_merchant,
    transactions_model::{
        NewSplit, NewTransaction, PayeeCategoryMemory, Transaction, TransactionUpdate,
    },
    transactions_traits::{
        CsvImportRequest, ImportResult, NewTransferLeg, OfxImportRequest,
        PayeeCategoryMemoryRepositoryTrait, TransactionFilters, TransactionRepositoryTrait,
        TransactionSearchResult, TransactionServiceTrait, TransactionWithRunningBalance,
        TransferEditMode,
    },
};

/// Concrete transaction service — thin orchestration layer over the repository.
pub struct TransactionService {
    repo: Arc<dyn TransactionRepositoryTrait>,
    payee_memory: Arc<dyn PayeeCategoryMemoryRepositoryTrait>,
}

impl TransactionService {
    pub fn new(
        repo: Arc<dyn TransactionRepositoryTrait>,
        payee_memory: Arc<dyn PayeeCategoryMemoryRepositoryTrait>,
    ) -> Self {
        Self { repo, payee_memory }
    }
}

#[async_trait]
impl TransactionServiceTrait for TransactionService {
    async fn create_transaction(&self, new: NewTransaction) -> Result<Transaction> {
        new.validate()?;
        self.repo.create_with_splits(new).await
    }

    async fn update_transaction(
        &self,
        update: TransactionUpdate,
        _edit_mode: TransferEditMode,
    ) -> Result<Transaction> {
        update.validate()?;
        self.repo.update_with_splits(update).await
    }

    async fn delete_transaction(&self, id: &str) -> Result<Transaction> {
        let txn = self.repo.get_by_id(id).await?;
        self.repo.delete(id).await?;
        Ok(txn)
    }

    async fn get_transaction(&self, id: &str) -> Result<Transaction> {
        self.repo.get_by_id(id).await
    }

    async fn search_transactions(
        &self,
        filters: TransactionFilters,
        page: i64,
        page_size: i64,
    ) -> Result<TransactionSearchResult> {
        self.repo.search(filters, page, page_size).await
    }

    async fn list_running_balance(
        &self,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<TransactionWithRunningBalance>> {
        self.repo
            .list_with_running_balance(account_id, from, to)
            .await
    }

    async fn create_transfer(
        &self,
        src: NewTransferLeg,
        dst: NewTransferLeg,
    ) -> Result<(Transaction, Transaction)> {
        use crate::transactions::transactions_constants::direction;

        let group_id = Uuid::new_v4().to_string();

        let src_txn = NewTransaction {
            account_id: src.account_id.clone(),
            direction: direction::TRANSFER.to_string(),
            amount: src.amount,
            currency: src.currency.clone(),
            transaction_date: src.transaction_date,
            payee: None,
            notes: src.notes.clone(),
            category_id: src.category_id.clone(),
            has_splits: false,
            fx_rate: src.fx_rate,
            fx_rate_source: src.fx_rate_source.clone(),
            transfer_group_id: Some(group_id.clone()),
            counterparty_account_id: Some(dst.account_id.clone()),
            transfer_leg_role: Some("SRC".to_string()),
            idempotency_key: None,
            import_run_id: None,
            source: "USER".to_string(),
            external_ref: None,
            is_system_generated: false,
            is_user_modified: false,
            category_source: None,
            splits: Vec::<NewSplit>::new(),
        };

        let dst_txn = NewTransaction {
            account_id: dst.account_id.clone(),
            direction: direction::TRANSFER.to_string(),
            amount: dst.amount,
            currency: dst.currency.clone(),
            transaction_date: dst.transaction_date,
            payee: None,
            notes: dst.notes.clone(),
            category_id: dst.category_id.clone(),
            has_splits: false,
            fx_rate: dst.fx_rate,
            fx_rate_source: dst.fx_rate_source.clone(),
            transfer_group_id: Some(group_id),
            counterparty_account_id: Some(src.account_id.clone()),
            transfer_leg_role: Some("DST".to_string()),
            idempotency_key: None,
            import_run_id: None,
            source: "USER".to_string(),
            external_ref: None,
            is_system_generated: false,
            is_user_modified: false,
            category_source: None,
            splits: Vec::<NewSplit>::new(),
        };

        let src_result = self.repo.create_with_splits(src_txn).await?;
        let dst_result = self.repo.create_with_splits(dst_txn).await?;
        Ok((src_result, dst_result))
    }

    async fn break_transfer_pair(&self, leg_id: &str) -> Result<Transaction> {
        let txn = self.repo.get_by_id(leg_id).await?;
        let update = TransactionUpdate {
            id: leg_id.to_string(),
            direction: None,
            amount: None,
            currency: None,
            transaction_date: None,
            payee: txn.payee.clone(),
            notes: txn.notes.clone(),
            category_id: txn.category_id.clone(),
            has_splits: None,
            fx_rate: None,
            fx_rate_source: None,
            // Clear transfer linkage fields
            transfer_group_id: Some(String::new()),
            counterparty_account_id: Some(String::new()),
            transfer_leg_role: Some(String::new()),
            idempotency_key: None,
            import_run_id: None,
            source: None,
            external_ref: None,
            is_system_generated: None,
            is_user_modified: None,
            category_source: None,
            splits: None,
        };
        self.repo.update_with_splits(update).await
    }

    async fn detect_import_duplicates(
        &self,
        candidates: Vec<NewTransaction>,
    ) -> Result<Vec<DuplicateMatch>> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }

        let mut all_existing: Vec<Transaction> = Vec::new();
        let mut seen_accounts: HashSet<String> = HashSet::new();
        for candidate in &candidates {
            if seen_accounts.insert(candidate.account_id.clone()) {
                let date_lo = candidate.transaction_date - chrono::Duration::days(3);
                let date_hi = candidate.transaction_date + chrono::Duration::days(3);
                let amount_lo = candidate.amount - rust_decimal::Decimal::new(1, 2);
                let amount_hi = candidate.amount + rust_decimal::Decimal::new(1, 2);
                let existing = self
                    .repo
                    .list_in_dup_window(
                        &candidate.account_id,
                        date_lo,
                        date_hi,
                        amount_lo,
                        amount_hi,
                    )
                    .await
                    .unwrap_or_default();
                all_existing.extend(existing);
            }
        }

        Ok(detect_duplicates(&candidates, &all_existing))
    }

    async fn import_csv(&self, req: CsvImportRequest) -> Result<ImportResult> {
        use crate::transactions::csv_parser::{map_row_to_new_transaction, parse_csv, ParseConfig};

        let import_run_id = req
            .import_run_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let config = ParseConfig::default();
        let parsed = parse_csv(&req.csv_bytes, &config)?;

        let mut new_transactions: Vec<NewTransaction> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for (i, row) in parsed.rows.iter().enumerate() {
            match map_row_to_new_transaction(
                row,
                &parsed.headers,
                &req.mapping,
                &req.account_id,
                &req.account_currency,
            ) {
                Ok(mut txn) => {
                    let ikey = compute_transaction_idempotency_key(
                        &txn.account_id,
                        &txn.direction,
                        txn.transaction_date,
                        txn.amount,
                        &txn.currency,
                        txn.payee.as_deref(),
                        txn.external_ref.as_deref(),
                    );
                    txn.idempotency_key = Some(ikey);
                    txn.import_run_id = Some(import_run_id.clone());
                    new_transactions.push(txn);
                }
                Err(e) => {
                    errors.push(format!("Row {}: {}", i + 1, e));
                }
            }
        }

        let mut inserted_row_ids: Vec<String> = vec![String::new(); new_transactions.len()];
        let mut inserted = 0usize;
        let mut skipped_duplicates = 0usize;

        for (idx, txn) in new_transactions.into_iter().enumerate() {
            if let Some(ref key) = txn.idempotency_key {
                if let Ok(Some(existing)) =
                    self.repo.get_by_idempotency_key(&txn.account_id, key).await
                {
                    inserted_row_ids[idx] = existing.id.clone();
                    skipped_duplicates += 1;
                    continue;
                }
            }
            match self.repo.create_with_splits(txn).await {
                Ok(created) => {
                    // Update payee memory if payee + category are set
                    if let (Some(ref payee), Some(ref cat_id)) =
                        (&created.payee, &created.category_id)
                    {
                        let mem = PayeeCategoryMemory {
                            account_id: created.account_id.clone(),
                            normalized_merchant: normalize_merchant(payee),
                            category_id: cat_id.clone(),
                            last_seen_at: created.created_at,
                            seen_count: 1,
                        };
                        let _ = self.payee_memory.upsert(mem).await;
                    }
                    inserted_row_ids[idx] = created.id.clone();
                    inserted += 1;
                }
                Err(e) => {
                    errors.push(format!("Insert failed: {}", e));
                }
            }
        }

        Ok(ImportResult {
            import_run_id,
            inserted,
            skipped_duplicates,
            errors,
            inserted_row_ids,
            duplicate_matches: Vec::new(),
        })
    }

    async fn import_ofx(&self, req: OfxImportRequest) -> Result<ImportResult> {
        // OFX parser is a stub (plan 04-02 task 3 TODO). Return empty result.
        // The timeout wrapper is applied at the Axum handler layer.
        let import_run_id = req
            .import_run_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Ok(ImportResult {
            import_run_id,
            inserted: 0,
            skipped_duplicates: 0,
            errors: vec!["OFX import not yet implemented (plan 04-02 task 3 TODO)".to_string()],
            inserted_row_ids: Vec::new(),
            duplicate_matches: Vec::new(),
        })
    }

    async fn lookup_payee_category(
        &self,
        account_id: &str,
        payee: &str,
    ) -> Result<Option<PayeeCategoryMemory>> {
        let normalized = normalize_merchant(payee);
        self.payee_memory.lookup(account_id, &normalized).await
    }

    async fn list_payee_category_memory(
        &self,
        account_id: &str,
    ) -> Result<Vec<PayeeCategoryMemory>> {
        self.payee_memory.list_for_account(account_id).await
    }
}
