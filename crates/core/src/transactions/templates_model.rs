//! CSV import template domain models (D-16/17/18).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// A single CSV column → transaction field mapping entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvFieldMapping {
    /// Map from transaction field name → CSV column index or name.
    pub fields: JsonValue,
}

/// A user-saved CSV import template (D-16/17/18).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTemplate {
    pub id: String,
    pub name: String,
    /// Full mapping as JSON (CsvFieldMapping).
    pub mapping: JsonValue,
    /// Ordered list of header column names from the CSV that was used to save this template.
    pub header_signature: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Input for creating a new template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTransactionTemplate {
    pub name: String,
    pub mapping: JsonValue,
    pub header_signature: Vec<String>,
}

/// Partial update for an existing template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTemplateUpdate {
    pub id: String,
    pub name: Option<String>,
    pub mapping: Option<JsonValue>,
    pub header_signature: Option<Vec<String>>,
}
