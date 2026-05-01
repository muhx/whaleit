//! Import-time duplicate detection (D-06/D-07/D-08/D-09).
//!
//! Three-key gate: same account_id + amount within $0.01 + date within ±3d.
//! Confidence formula: amount_exactness*0.4 + date_closeness*0.3 + payee_similarity*0.3
//! Bucket mapping: ≥95 AlmostCertain, 70-94 Likely, 50-69 Possible, <50 suppressed.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{merchant_normalizer::normalize_merchant, NewTransaction, Transaction};

/// Confidence bucket for a duplicate match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DuplicateBucket {
    AlmostCertain,
    Likely,
    Possible,
}

/// A duplicate match result for a candidate import row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateMatch {
    /// Index of the candidate row in the input slice.
    pub candidate_row_index: usize,
    /// ID of the existing transaction that matches.
    pub existing_transaction_id: String,
    /// Confidence score 0..=100.
    pub confidence: u8,
    /// Confidence bucket.
    pub bucket: DuplicateBucket,
}

/// Detects duplicate candidates against existing transactions in the window.
///
/// Also performs within-batch detection: if two candidates in `candidates`
/// match each other, the later one is flagged.
pub fn detect_duplicates(
    candidates: &[NewTransaction],
    existing_in_window: &[Transaction],
) -> Vec<DuplicateMatch> {
    let mut results = Vec::new();

    for (i, candidate) in candidates.iter().enumerate() {
        let mut best_confidence: f64 = 0.0;
        let mut best_existing_id: Option<String> = None;

        // Check against pre-existing transactions
        for existing in existing_in_window {
            if let Some((id, conf)) = score_pair(
                &candidate.account_id,
                candidate.amount,
                candidate.transaction_date,
                candidate.payee.as_deref(),
                &existing.account_id,
                existing.amount,
                existing.transaction_date,
                existing.payee.as_deref(),
                &existing.id,
            ) {
                if conf > best_confidence {
                    best_confidence = conf;
                    best_existing_id = Some(id);
                }
            }
        }

        // Within-batch: check earlier candidates (project to synthetic IDs)
        for (j, prior) in candidates[..i].iter().enumerate() {
            let synthetic_id = format!("batch:{}", j);
            if let Some((id, conf)) = score_pair(
                &candidate.account_id,
                candidate.amount,
                candidate.transaction_date,
                candidate.payee.as_deref(),
                &prior.account_id,
                prior.amount,
                prior.transaction_date,
                prior.payee.as_deref(),
                &synthetic_id,
            ) {
                if conf > best_confidence {
                    best_confidence = conf;
                    best_existing_id = Some(id);
                }
            }
        }

        if let Some(existing_id) = best_existing_id {
            let confidence_u8 = best_confidence.round() as u8;
            if let Some(bucket) = bucket_for(confidence_u8) {
                results.push(DuplicateMatch {
                    candidate_row_index: i,
                    existing_transaction_id: existing_id,
                    confidence: confidence_u8,
                    bucket,
                });
            }
        }
    }

    results
}

/// Returns `(existing_id, confidence_0_to_100)` if the three-key gate passes.
#[allow(clippy::too_many_arguments)]
fn score_pair(
    cand_account: &str,
    cand_amount: Decimal,
    cand_date: NaiveDate,
    cand_payee: Option<&str>,
    exist_account: &str,
    exist_amount: Decimal,
    exist_date: NaiveDate,
    exist_payee: Option<&str>,
    existing_id: &str,
) -> Option<(String, f64)> {
    // --- Three-key gate ---
    if cand_account != exist_account {
        return None;
    }
    let amount_delta = (cand_amount - exist_amount).abs();
    if amount_delta > Decimal::new(1, 2) {
        return None;
    }
    let day_delta = (cand_date - exist_date).num_days().unsigned_abs();
    if day_delta > 3 {
        return None;
    }

    // --- Confidence formula ---
    let amount_exactness: f64 = if amount_delta < Decimal::new(1, 3) {
        1.0
    } else {
        0.5
    };

    let date_closeness: f64 = 1.0 - (day_delta as f64) / 3.0;

    let payee_similarity: f64 = {
        let norm_cand = normalize_merchant(cand_payee.unwrap_or(""));
        let norm_exist = normalize_merchant(exist_payee.unwrap_or(""));
        strsim::normalized_levenshtein(&norm_cand, &norm_exist)
    };

    let confidence =
        (amount_exactness * 0.4 + date_closeness * 0.3 + payee_similarity * 0.3) * 100.0;

    Some((existing_id.to_string(), confidence))
}

/// Maps a confidence score to a bucket. Returns None if below 50 (suppressed).
fn bucket_for(confidence: u8) -> Option<DuplicateBucket> {
    match confidence {
        95..=100 => Some(DuplicateBucket::AlmostCertain),
        70..=94 => Some(DuplicateBucket::Likely),
        50..=69 => Some(DuplicateBucket::Possible),
        _ => None,
    }
}
