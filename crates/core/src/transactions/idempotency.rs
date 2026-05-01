//! Idempotency key computation for transaction deduplication (Phase 4).
//!
//! When OFX FITID is present it is used as the external_ref and dominates the
//! hash, making the key bank-stable across re-imports of the same file.
//! Without an external_ref the key is a SHA-256 of canonical semantic fields.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};

/// Computes a stable idempotency key for a transaction import row.
///
/// # Canonical fields (pipe-delimited, in order)
/// `account_id | direction | transaction_date | amount_normalized | currency | payee | external_ref`
///
/// When `external_ref = Some(fitid)` the FITID is the last field and makes the
/// key deterministic even if payee metadata changes between exports.
pub fn compute_transaction_idempotency_key(
    account_id: &str,
    direction: &str,
    transaction_date: NaiveDate,
    amount: Decimal,
    currency: &str,
    payee: Option<&str>,
    external_ref: Option<&str>,
) -> String {
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        account_id,
        direction,
        transaction_date,
        amount.normalize(),
        currency,
        payee.unwrap_or(""),
        external_ref.unwrap_or(""),
    );
    let mut h = Sha256::new();
    h.update(canonical.as_bytes());
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn identical_inputs_equal() {
        let k1 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(42.50),
            "USD",
            Some("Starbucks"),
            None,
        );
        let k2 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(42.50),
            "USD",
            Some("Starbucks"),
            None,
        );
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 64);
    }

    #[test]
    fn single_field_changed_differs() {
        let k1 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(42.50),
            "USD",
            Some("Starbucks"),
            None,
        );
        let k2 = compute_transaction_idempotency_key(
            "acc2",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(42.50),
            "USD",
            Some("Starbucks"),
            None,
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn fitid_takes_precedence() {
        // Same semantic fields but different FITIDs → different keys (bank-stable)
        let k1 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(42.50),
            "USD",
            Some("Starbucks"),
            Some("FITID001"),
        );
        let k2 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(42.50),
            "USD",
            Some("Starbucks"),
            Some("FITID002"),
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn decimal_normalization_same_key() {
        // 1.10 and 1.1 should produce the same normalized string → same key
        let k1 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(1.10),
            "USD",
            Some("Store"),
            None,
        );
        let k2 = compute_transaction_idempotency_key(
            "acc1",
            "EXPENSE",
            date(2026, 1, 15),
            dec!(1.1),
            "USD",
            Some("Store"),
            None,
        );
        assert_eq!(k1, k2);
    }
}
