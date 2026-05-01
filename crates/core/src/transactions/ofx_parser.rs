//! OFX 1.x SGML and OFX 2.x XML parser (TXN-05, D-19).
//!
//! Detection: the parser is format-agnostic — both 1.x SGML (unclosed tags,
//! `DATA:OFXSGML` header) and 2.x XML (well-formed, `<?xml ?>` prolog) share
//! the `<TAG>value` shape, so a single permissive field-extractor handles both.
//! The header preamble is stripped up to the first `<OFX>` element.
//!
//! `FITID` is the bank-stable per-row id; the importer service uses it as the
//! `external_ref` and as the idempotency key — banks promise FITID stability
//! across re-exports, which dominates a row-payload hash for de-duping.

use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{errors::ValidationError, Error, Result};

/// One transaction extracted from an OFX `<STMTTRN>` block.
#[derive(Debug, Clone, PartialEq)]
pub struct OfxTransaction {
    /// `FITID` — bank-issued unique transaction id.
    pub fitid: String,
    /// `TRNAMT` — signed amount (positive = inflow / income, negative = outflow / expense).
    pub amount: Decimal,
    /// `DTPOSTED` — transaction date (only the `YYYYMMDD` prefix is consumed).
    pub date: NaiveDate,
    /// `NAME` (preferred) or `PAYEE.NAME` — merchant / payer name.
    pub name: Option<String>,
    /// `MEMO` — optional notes.
    pub memo: Option<String>,
    /// `TRNTYPE` — OFX-defined transaction type (CREDIT, DEBIT, INT, DIV, FEE, etc.).
    pub trntype: Option<String>,
}

/// Parse an OFX document (1.x SGML or 2.x XML) and return the
/// `<STMTTRN>` blocks as a list of typed records.
pub fn parse_ofx(input: &str) -> Result<Vec<OfxTransaction>> {
    let body = strip_to_ofx_root(input);
    let mut out: Vec<OfxTransaction> = Vec::new();
    for (idx, block) in iter_stmttrn_blocks(body).into_iter().enumerate() {
        match parse_stmttrn(block) {
            Ok(Some(txn)) => out.push(txn),
            Ok(None) => {} // missing required field — skip silently
            Err(e) => {
                return Err(invalid(format!(
                    "OFX parse error in STMTTRN #{}: {}",
                    idx + 1,
                    e
                )));
            }
        }
    }
    Ok(out)
}

/// Map an OFX TRNAMT sign to a domain `direction` constant.
///
/// Sign of TRNAMT is authoritative (positive → INCOME, negative → EXPENSE).
/// `TRNTYPE` is informational only — institutions disagree on its exact set
/// of values, but the sign convention is universal across OFX exports.
pub fn direction_from_amount(amount: Decimal) -> &'static str {
    if amount.is_sign_negative() {
        "EXPENSE"
    } else {
        "INCOME"
    }
}

/// Strip the OFX header block (1.x DATA:OFXSGML, 2.x `<?xml ...?>` + `<?OFX ...?>`)
/// up to the first `<OFX>` element. Tolerant of whitespace and CRLF.
fn strip_to_ofx_root(input: &str) -> &str {
    if let Some(idx) = input.find("<OFX>") {
        return &input[idx..];
    }
    if let Some(idx) = input.find("<OFX ") {
        return &input[idx..];
    }
    input
}

/// Find every `<STMTTRN>...</STMTTRN>` (or unclosed `<STMTTRN>...next-open`)
/// block. Returns slices over `input`.
fn iter_stmttrn_blocks(input: &str) -> Vec<&str> {
    const OPEN: &str = "<STMTTRN>";
    const CLOSE: &str = "</STMTTRN>";
    let mut blocks: Vec<&str> = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = input[cursor..].find(OPEN) {
        let body_start = cursor + rel + OPEN.len();
        // Closed form (XML / well-formed SGML): scan for </STMTTRN>.
        if let Some(end_rel) = input[body_start..].find(CLOSE) {
            let body_end = body_start + end_rel;
            blocks.push(&input[body_start..body_end]);
            cursor = body_end + CLOSE.len();
            continue;
        }
        // Unclosed (SGML 1.x): scan to the next <STMTTRN>, the </BANKTRANLIST>
        // terminator, or end-of-input — whichever comes first.
        let next_open = input[body_start..].find(OPEN).map(|n| body_start + n);
        let next_terminator = input[body_start..]
            .find("</BANKTRANLIST>")
            .map(|n| body_start + n);
        let stop = match (next_open, next_terminator) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => input.len(),
        };
        blocks.push(&input[body_start..stop]);
        cursor = stop;
    }
    blocks
}

/// Parse a single `<STMTTRN>` block body. Returns `Ok(None)` when a required
/// field is missing (FITID / TRNAMT / DTPOSTED) — the row is skipped.
fn parse_stmttrn(block: &str) -> std::result::Result<Option<OfxTransaction>, String> {
    let fitid = match extract_field(block, "FITID") {
        Some(s) => s,
        None => return Ok(None),
    };
    let amount_str = match extract_field(block, "TRNAMT") {
        Some(s) => s,
        None => return Ok(None),
    };
    let amount = Decimal::from_str(&amount_str)
        .map_err(|e| format!("invalid TRNAMT '{}': {}", amount_str, e))?;
    let date_str = match extract_field(block, "DTPOSTED") {
        Some(s) => s,
        None => return Ok(None),
    };
    let date = parse_ofx_date(&date_str)?;
    let name = extract_field(block, "NAME").or_else(|| extract_field(block, "PAYEE.NAME"));
    let memo = extract_field(block, "MEMO");
    let trntype = extract_field(block, "TRNTYPE");
    Ok(Some(OfxTransaction {
        fitid,
        amount,
        date,
        name,
        memo,
        trntype,
    }))
}

/// Extract the first occurrence of `<TAG>value` in `block`. Reads the value
/// from after the open tag up to the next `<` (which may be the close tag in
/// XML or the next open tag in SGML). Returns `None` if the tag is absent or
/// the value is empty after trimming.
pub(super) fn extract_field(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let start = block.find(&open)? + open.len();
    let rest = &block[start..];
    let end = rest.find('<').unwrap_or(rest.len());
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Parse an OFX date string (`YYYYMMDD[HHMMSS[.xxx]][TZ]`). Only the first
/// 8 characters are consumed.
fn parse_ofx_date(s: &str) -> std::result::Result<NaiveDate, String> {
    if s.len() < 8 {
        return Err(format!("OFX date too short: '{}'", s));
    }
    NaiveDate::parse_from_str(&s[..8], "%Y%m%d")
        .map_err(|e| format!("invalid OFX date '{}': {}", s, e))
}

fn invalid(msg: String) -> Error {
    Error::Validation(ValidationError::InvalidInput(msg))
}
