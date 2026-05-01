//! CSV import template domain models (D-16/17/18).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::csv_parser::CsvFieldMapping;

/// A saved CSV import template (globally scoped, D-18).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTemplate {
    pub id: String,
    pub name: String,
    pub mapping: CsvFieldMapping,
    pub header_signature: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Input for creating a new template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTransactionTemplate {
    pub name: String,
    pub mapping: CsvFieldMapping,
    pub header_signature: Vec<String>,
}

/// Input for updating an existing template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTemplateUpdate {
    pub id: String,
    pub name: Option<String>,
    pub mapping: Option<CsvFieldMapping>,
    pub header_signature: Option<Vec<String>>,
}
