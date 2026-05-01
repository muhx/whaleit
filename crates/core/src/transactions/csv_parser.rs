//! CSV parser for transaction imports — re-exports activities primitives and
//! adds transaction-specific row mapping.

pub use crate::activities::csv_parser::{parse_csv, ParseConfig, ParseError, ParsedCsvResult};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{errors::ValidationError, Error, Result};

use super::transactions_constants::direction;
use super::transactions_model::NewTransaction;

/// Column mapping for a CSV transaction import template (D-16/17/18).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvFieldMapping {
    pub date_column: String,
    /// Single amount column (positive = income, negative = expense).
    pub amount_column: Option<String>,
    /// Separate debit column (positive number = expense).
    pub debit_column: Option<String>,
    /// Separate credit column (positive number = income).
    pub credit_column: Option<String>,
    pub payee_column: String,
    pub category_column: Option<String>,
    pub notes_column: Option<String>,
    pub currency_column: Option<String>,
    pub external_id_column: Option<String>,
    /// Date format string, e.g. "%m/%d/%Y" (US default, D-19 open question).
    pub date_format: String,
    pub decimal_separator: char,
    pub thousands_separator: Option<char>,
}

/// Maps a parsed CSV row to a `NewTransaction`.
///
/// `row` is the cell values; `headers` is the header row for column lookup.
pub fn map_row_to_new_transaction(
    row: &[String],
    headers: &[String],
    mapping: &CsvFieldMapping,
    account_id: &str,
    account_currency: &str,
) -> Result<NewTransaction> {
    let col = |name: &str| -> Option<&str> {
        headers
            .iter()
            .position(|h| h == name)
            .and_then(|idx| row.get(idx))
            .map(String::as_str)
    };

    // --- Date ---
    let date_str = col(&mapping.date_column).ok_or_else(|| {
        Error::Validation(ValidationError::InvalidInput(format!(
            "Date column '{}' not found",
            mapping.date_column
        )))
    })?;
    let transaction_date = NaiveDate::parse_from_str(date_str.trim(), &mapping.date_format)
        .map_err(|e| {
            Error::Validation(ValidationError::InvalidInput(format!(
                "Date parse failed for '{}' with format '{}': {}",
                date_str, mapping.date_format, e
            )))
        })?;

    // --- Amount ---
    let amount_raw: Decimal;
    let inferred_direction: &str;

    if let Some(amt_col) = &mapping.amount_column {
        let raw = col(amt_col).ok_or_else(|| {
            Error::Validation(ValidationError::InvalidInput(format!(
                "Amount column '{}' not found",
                amt_col
            )))
        })?;
        let parsed =
            parse_amount_string(raw, mapping.decimal_separator, mapping.thousands_separator)?;
        if parsed >= Decimal::ZERO {
            amount_raw = parsed;
            inferred_direction = direction::INCOME;
        } else {
            amount_raw = -parsed;
            inferred_direction = direction::EXPENSE;
        }
    } else {
        // Separate debit + credit columns
        let debit_col = mapping.debit_column.as_deref().unwrap_or("");
        let credit_col = mapping.credit_column.as_deref().unwrap_or("");
        let debit_str = col(debit_col).unwrap_or("").trim().to_string();
        let credit_str = col(credit_col).unwrap_or("").trim().to_string();

        let debit = if debit_str.is_empty() {
            Decimal::ZERO
        } else {
            parse_amount_string(
                &debit_str,
                mapping.decimal_separator,
                mapping.thousands_separator,
            )?
        };
        let credit = if credit_str.is_empty() {
            Decimal::ZERO
        } else {
            parse_amount_string(
                &credit_str,
                mapping.decimal_separator,
                mapping.thousands_separator,
            )?
        };

        if credit > Decimal::ZERO {
            amount_raw = credit;
            inferred_direction = direction::INCOME;
        } else {
            amount_raw = debit.abs();
            inferred_direction = direction::EXPENSE;
        }
    }

    // --- Payee ---
    let payee = col(&mapping.payee_column)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // --- Optional fields ---
    let notes = mapping
        .notes_column
        .as_deref()
        .and_then(|c| col(c))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let category_id = mapping
        .category_column
        .as_deref()
        .and_then(|c| col(c))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let currency = mapping
        .currency_column
        .as_deref()
        .and_then(|c| col(c))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| account_currency.to_string());

    let external_ref = mapping
        .external_id_column
        .as_deref()
        .and_then(|c| col(c))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(NewTransaction {
        account_id: account_id.to_string(),
        direction: inferred_direction.to_string(),
        amount: amount_raw,
        currency,
        transaction_date,
        payee,
        notes,
        category_id,
        has_splits: false,
        fx_rate: None,
        fx_rate_source: None,
        transfer_group_id: None,
        counterparty_account_id: None,
        transfer_leg_role: None,
        idempotency_key: None,
        import_run_id: None,
        source: crate::transactions::transactions_constants::source::CSV.to_string(),
        external_ref,
        is_system_generated: false,
        is_user_modified: false,
        category_source: None,
        splits: vec![],
    })
}

/// Parses an amount string, handling:
/// - parenthesized negatives: `(42.10)` → -42.10
/// - currency symbols stripped: `$42.10` → 42.10
/// - EU thousands: `1.234,56` (when decimal_separator is `,`)
fn parse_amount_string(
    raw: &str,
    decimal_sep: char,
    thousands_sep: Option<char>,
) -> Result<Decimal> {
    let s = raw.trim();

    // Parenthesized negative
    let (negative, s) = if s.starts_with('(') && s.ends_with(')') {
        (true, &s[1..s.len() - 1])
    } else {
        (false, s)
    };

    // Strip currency symbols and spaces
    let s: String = s
        .chars()
        .filter(|c| {
            c.is_ascii_digit()
                || *c == decimal_sep
                || thousands_sep.map_or(false, |t| *c == t)
                || *c == '-'
        })
        .collect();

    // Remove thousands separator
    let s = if let Some(t) = thousands_sep {
        s.replace(t, "")
    } else {
        s
    };

    // Normalize decimal separator to '.'
    let s = if decimal_sep != '.' {
        s.replace(decimal_sep, ".")
    } else {
        s
    };

    let mut value: Decimal = s.parse().map_err(|_| {
        Error::Validation(ValidationError::InvalidInput(format!(
            "Cannot parse amount from '{}'",
            raw
        )))
    })?;

    if negative {
        value = -value;
    }

    Ok(value)
}
