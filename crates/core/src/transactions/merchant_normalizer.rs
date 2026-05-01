//! Merchant name normalization for payee→category memory keys (D-13).
//!
//! Algorithm: lowercase + trim + collapse runs of digits to "#" + collapse
//! runs of whitespace to a single space.
//!
//! Examples:
//!   "WHOLEFDS GRP #10403"     → "wholefds grp #"
//!   "STARBUCKS  STORE 12345"  → "starbucks store #"

use regex::Regex;
use std::sync::OnceLock;

static DIGIT_RUN: OnceLock<Regex> = OnceLock::new();
static SPACE_RUN: OnceLock<Regex> = OnceLock::new();

/// Normalizes a merchant/payee string for use as a memory lookup key.
pub fn normalize_merchant(input: &str) -> String {
    let digits = DIGIT_RUN.get_or_init(|| Regex::new(r"\d+").unwrap());
    let spaces = SPACE_RUN.get_or_init(|| Regex::new(r"\s+").unwrap());
    let lower = input.trim().to_lowercase();
    let no_digits = digits.replace_all(&lower, "#");
    spaces.replace_all(&no_digits, " ").into_owned()
}
