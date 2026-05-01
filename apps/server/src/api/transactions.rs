use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::NaiveDate;
use serde::Deserialize;
use tokio::time::timeout;

use whaleit_core::transactions::{
    CsvImportRequest, DuplicateMatch, ImportResult, NewTransaction, NewTransactionTemplate,
    NewTransferLeg, OfxImportRequest, PayeeCategoryMemory, Transaction, TransactionFilters,
    TransactionSearchResult, TransactionTemplate, TransactionUpdate, TransactionWithRunningBalance,
    TransferEditMode,
};

use crate::{error::ApiResult, main_lib::AppState};

const MAX_IMPORT_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_PARSED_ROWS: usize = 50_000;
const OFX_PARSE_TIMEOUT_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// Request body types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchTransactionsBody {
    page: i64,
    page_size: i64,
    filters: TransactionFilters,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunningBalanceBody {
    account_id: String,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentTransactionsQuery {
    account_id: String,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTransactionBody {
    transaction: TransactionUpdate,
    edit_mode: Option<TransferEditMode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteTransactionQuery {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetTransactionQuery {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetectDuplicatesBody {
    candidates: Vec<NewTransaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTransferBody {
    src: NewTransferLeg,
    dst: NewTransferLeg,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BreakTransferBody {
    leg_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LookupPayeeCategoryBody {
    account_id: String,
    payee: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListPayeeCategoryMemoryQuery {
    account_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateIdQuery {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveTemplateBody {
    template: NewTransactionTemplate,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn search_transactions(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SearchTransactionsBody>,
) -> ApiResult<Json<TransactionSearchResult>> {
    let result = state
        .transaction_service
        .search_transactions(body.filters, body.page, body.page_size)
        .await?;
    Ok(Json(result))
}

async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Query(q): Query<GetTransactionQuery>,
) -> ApiResult<Json<Transaction>> {
    let txn = state.transaction_service.get_transaction(&q.id).await?;
    Ok(Json(txn))
}

async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(new): Json<NewTransaction>,
) -> ApiResult<Json<Transaction>> {
    let created = state.transaction_service.create_transaction(new).await?;
    Ok(Json(created))
}

async fn update_transaction(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateTransactionBody>,
) -> ApiResult<Json<Transaction>> {
    let edit_mode = body.edit_mode.unwrap_or(TransferEditMode::ApplyBoth);
    let updated = state
        .transaction_service
        .update_transaction(body.transaction, edit_mode)
        .await?;
    Ok(Json(updated))
}

async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DeleteTransactionQuery>,
) -> ApiResult<Json<Transaction>> {
    let deleted = state.transaction_service.delete_transaction(&q.id).await?;
    Ok(Json(deleted))
}

async fn list_running_balance(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RunningBalanceBody>,
) -> ApiResult<Json<Vec<TransactionWithRunningBalance>>> {
    let items = state
        .transaction_service
        .list_running_balance(&body.account_id, body.from, body.to)
        .await?;
    Ok(Json(items))
}

async fn get_account_recent_transactions(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RecentTransactionsQuery>,
) -> ApiResult<Json<Vec<Transaction>>> {
    let limit = q.limit.unwrap_or(50);
    let items = state
        .transaction_service
        .search_transactions(
            TransactionFilters {
                account_ids: vec![q.account_id],
                ..Default::default()
            },
            0,
            limit,
        )
        .await?;
    Ok(Json(items.items))
}

async fn import_csv(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ApiResult<Json<ImportResult>> {
    let mut account_id = String::new();
    let mut account_currency = String::new();
    let mut csv_bytes: Vec<u8> = Vec::new();
    let mut mapping_json = String::new();
    let mut import_run_id: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "accountId" => {
                account_id = field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read accountId: {}", e))
                })?;
            }
            "accountCurrency" => {
                account_currency = field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!(
                        "Failed to read accountCurrency: {}",
                        e
                    ))
                })?;
            }
            "file" => {
                let bytes = field.bytes().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read file: {}", e))
                })?;
                if bytes.len() > MAX_IMPORT_BYTES {
                    return Err(crate::error::ApiError::BadRequest(format!(
                        "File too large: {} bytes (max {} bytes)",
                        bytes.len(),
                        MAX_IMPORT_BYTES
                    )));
                }
                csv_bytes = bytes.to_vec();
            }
            "mapping" => {
                mapping_json = field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read mapping: {}", e))
                })?;
            }
            "importRunId" => {
                import_run_id = Some(field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read importRunId: {}", e))
                })?);
            }
            _ => {}
        }
    }

    if account_id.is_empty() {
        return Err(crate::error::ApiError::BadRequest(
            "Missing accountId field".to_string(),
        ));
    }
    if csv_bytes.is_empty() {
        return Err(crate::error::ApiError::BadRequest(
            "Missing file field".to_string(),
        ));
    }

    let mapping = serde_json::from_str(&mapping_json)
        .map_err(|e| crate::error::ApiError::BadRequest(format!("Invalid mapping JSON: {}", e)))?;

    let req = CsvImportRequest {
        account_id,
        account_currency,
        csv_bytes,
        mapping,
        import_run_id,
    };

    let result = state.transaction_service.import_csv(req).await?;
    Ok(Json(result))
}

async fn import_ofx(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ApiResult<Json<ImportResult>> {
    let mut account_id = String::new();
    let mut account_currency = String::new();
    let mut ofx_bytes: Vec<u8> = Vec::new();
    let mut import_run_id: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "accountId" => {
                account_id = field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read accountId: {}", e))
                })?;
            }
            "accountCurrency" => {
                account_currency = field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!(
                        "Failed to read accountCurrency: {}",
                        e
                    ))
                })?;
            }
            "file" => {
                let bytes = field.bytes().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read file: {}", e))
                })?;
                if bytes.len() > MAX_IMPORT_BYTES {
                    return Err(crate::error::ApiError::BadRequest(format!(
                        "File too large: {} bytes (max {} bytes)",
                        bytes.len(),
                        MAX_IMPORT_BYTES
                    )));
                }
                ofx_bytes = bytes.to_vec();
            }
            "importRunId" => {
                import_run_id = Some(field.text().await.map_err(|e| {
                    crate::error::ApiError::BadRequest(format!("Failed to read importRunId: {}", e))
                })?);
            }
            _ => {}
        }
    }

    if account_id.is_empty() {
        return Err(crate::error::ApiError::BadRequest(
            "Missing accountId field".to_string(),
        ));
    }
    if ofx_bytes.is_empty() {
        return Err(crate::error::ApiError::BadRequest(
            "Missing file field".to_string(),
        ));
    }

    let req = OfxImportRequest {
        account_id,
        account_currency,
        ofx_bytes,
        import_run_id,
    };

    let result = timeout(
        Duration::from_secs(OFX_PARSE_TIMEOUT_SECS),
        state.transaction_service.import_ofx(req),
    )
    .await
    .map_err(|_| crate::error::ApiError::BadRequest("OFX parse timed out (30s)".to_string()))??;

    Ok(Json(result))
}

async fn preview_import(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DetectDuplicatesBody>,
) -> ApiResult<Json<Vec<DuplicateMatch>>> {
    // Preview = detect duplicates against existing data
    let matches = state
        .transaction_service
        .detect_import_duplicates(body.candidates)
        .await?;
    Ok(Json(matches))
}

async fn detect_duplicates(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DetectDuplicatesBody>,
) -> ApiResult<Json<Vec<DuplicateMatch>>> {
    if body.candidates.len() > MAX_PARSED_ROWS {
        return Err(crate::error::ApiError::BadRequest(format!(
            "Too many candidates: {} (max {})",
            body.candidates.len(),
            MAX_PARSED_ROWS
        )));
    }
    let matches = state
        .transaction_service
        .detect_import_duplicates(body.candidates)
        .await?;
    Ok(Json(matches))
}

async fn list_templates(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<TransactionTemplate>>> {
    let templates = state.template_service.list_templates().await?;
    Ok(Json(templates))
}

async fn save_template(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SaveTemplateBody>,
) -> ApiResult<Json<TransactionTemplate>> {
    let template = state.template_service.save_template(body.template).await?;
    Ok(Json(template))
}

async fn delete_template(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TemplateIdQuery>,
) -> ApiResult<StatusCode> {
    state.template_service.delete_template(&q.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_template(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TemplateIdQuery>,
) -> ApiResult<Json<TransactionTemplate>> {
    let template = state.template_service.get_template(&q.id).await?;
    Ok(Json(template))
}

async fn create_transfer(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTransferBody>,
) -> ApiResult<Json<(Transaction, Transaction)>> {
    let pair = state
        .transaction_service
        .create_transfer(body.src, body.dst)
        .await?;
    Ok(Json(pair))
}

async fn update_transfer_leg(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateTransactionBody>,
) -> ApiResult<Json<Transaction>> {
    let edit_mode = body.edit_mode.unwrap_or(TransferEditMode::ThisLegOnly);
    let updated = state
        .transaction_service
        .update_transaction(body.transaction, edit_mode)
        .await?;
    Ok(Json(updated))
}

async fn break_transfer_pair(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BreakTransferBody>,
) -> ApiResult<Json<Transaction>> {
    let txn = state
        .transaction_service
        .break_transfer_pair(&body.leg_id)
        .await?;
    Ok(Json(txn))
}

async fn lookup_payee_category(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LookupPayeeCategoryBody>,
) -> ApiResult<Json<Option<PayeeCategoryMemory>>> {
    let mem = state
        .transaction_service
        .lookup_payee_category(&body.account_id, &body.payee)
        .await?;
    Ok(Json(mem))
}

async fn list_payee_category_memory(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListPayeeCategoryMemoryQuery>,
) -> ApiResult<Json<Vec<PayeeCategoryMemory>>> {
    let items = state
        .transaction_service
        .list_payee_category_memory(&q.account_id)
        .await?;
    Ok(Json(items))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Search
        .route("/transactions/search", post(search_transactions))
        // Single transaction CRUD
        .route("/transactions/item", get(get_transaction))
        .route("/transactions", post(create_transaction))
        .route("/transactions", put(update_transaction))
        .route("/transactions", delete(delete_transaction))
        // Running balance + recent
        .route("/transactions/running-balance", post(list_running_balance))
        .route(
            "/transactions/by-account/recent",
            get(get_account_recent_transactions),
        )
        // Import
        .route("/transactions/import/csv", post(import_csv))
        .route("/transactions/import/ofx", post(import_ofx))
        .route("/transactions/import/preview", post(preview_import))
        .route("/transactions/import/duplicates", post(detect_duplicates))
        // Templates (D-16/17/18)
        .route(
            "/transactions/import/templates",
            get(list_templates)
                .post(save_template)
                .delete(delete_template),
        )
        .route("/transactions/import/templates/item", get(get_template))
        // Transfers
        .route("/transactions/transfer", post(create_transfer))
        .route("/transactions/transfer/leg", put(update_transfer_leg))
        .route("/transactions/transfer/break", post(break_transfer_pair))
        // Payee category memory
        .route(
            "/transactions/payee-category-memory/lookup",
            post(lookup_payee_category),
        )
        .route(
            "/transactions/payee-category-memory",
            get(list_payee_category_memory),
        )
}
