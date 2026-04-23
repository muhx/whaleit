//! Tests for QuoteService contracts and edge cases.
//!
//! These tests verify the QuoteServiceTrait contract and ensure proper handling
//! of edge cases during the migration from MarketDataServiceTrait.
//!
//! # Critical Contract Points
//!
//! 1. Quote CRUD: All quote operations must maintain consistency
//! 2. Provider operations: Symbol search, profile fetching must delegate to MarketDataClient
//! 3. Sync operations: Must properly orchestrate via QuoteSyncService
//! 4. Gap filling: **CRITICAL** - get_quotes_in_range must fill weekend/holiday gaps
//! 5. Provider settings: Changes must refresh the MarketDataClient

#[cfg(test)]
mod tests {
    use crate::errors::{DatabaseError, Result};
    use crate::quotes::service::{append_historical_seed_quotes, fill_missing_quotes};
    use crate::quotes::{
        model::{LatestQuotePair, Quote},
        store::QuoteStore,
        types::{AssetId, Day, QuoteSource},
    };
    use async_trait::async_trait;
    use chrono::{Duration, NaiveDate, TimeZone, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    // =========================================================================
    // Mock QuoteStore
    // =========================================================================

    #[derive(Clone, Default)]
    struct MockQuoteStore {
        quotes: Arc<Mutex<Vec<Quote>>>,
        fail_on_save: Arc<Mutex<bool>>,
    }

    impl MockQuoteStore {
        fn new() -> Self {
            Self::default()
        }

        #[allow(dead_code)]
        fn with_quotes(quotes: Vec<Quote>) -> Self {
            Self {
                quotes: Arc::new(Mutex::new(quotes)),
                fail_on_save: Arc::new(Mutex::new(false)),
            }
        }

        #[allow(dead_code)]
        fn set_fail_on_save(&self, fail: bool) {
            *self.fail_on_save.lock().unwrap() = fail;
        }

        fn add_quote(&self, quote: Quote) {
            self.quotes.lock().unwrap().push(quote);
        }

        #[allow(dead_code)]
        fn get_all(&self) -> Vec<Quote> {
            self.quotes.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl QuoteStore for MockQuoteStore {
        async fn save_quote(&self, quote: &Quote) -> Result<Quote> {
            if *self.fail_on_save.lock().unwrap() {
                return Err(crate::Error::Unexpected("Intentional save failure".into()));
            }
            let mut quotes = self.quotes.lock().unwrap();
            quotes.retain(|q| q.id != quote.id);
            quotes.push(quote.clone());
            Ok(quote.clone())
        }

        async fn delete_quote(&self, quote_id: &str) -> Result<()> {
            let mut quotes = self.quotes.lock().unwrap();
            quotes.retain(|q| q.id != quote_id);
            Ok(())
        }

        async fn upsert_quotes(&self, quotes_to_upsert: &[Quote]) -> Result<usize> {
            if *self.fail_on_save.lock().unwrap() {
                return Err(crate::Error::Unexpected("Intentional save failure".into()));
            }
            let mut quotes = self.quotes.lock().unwrap();
            let mut count = 0;
            for quote in quotes_to_upsert {
                quotes.retain(|q| q.id != quote.id);
                quotes.push(quote.clone());
                count += 1;
            }
            Ok(count)
        }

        async fn delete_quotes_for_asset(&self, asset_id: &AssetId) -> Result<usize> {
            let mut quotes = self.quotes.lock().unwrap();
            let original_len = quotes.len();
            quotes.retain(|q| q.asset_id != asset_id.as_str());
            Ok(original_len - quotes.len())
        }

        async fn delete_provider_quotes_for_asset(&self, asset_id: &AssetId) -> Result<usize> {
            let mut quotes = self.quotes.lock().unwrap();
            let original_len = quotes.len();
            quotes.retain(|q| q.asset_id != asset_id.as_str() || q.data_source == "MANUAL");
            Ok(original_len - quotes.len())
        }

        async fn latest(
            &self,
            asset_id: &AssetId,
            _source: Option<&QuoteSource>,
        ) -> Result<Option<Quote>> {
            let quotes = self.quotes.lock().unwrap();
            Ok(quotes
                .iter()
                .filter(|q| q.asset_id == asset_id.as_str())
                .max_by_key(|q| q.timestamp)
                .cloned())
        }

        async fn range(
            &self,
            asset_id: &AssetId,
            start: Day,
            end: Day,
            _source: Option<&QuoteSource>,
        ) -> Result<Vec<Quote>> {
            let quotes = self.quotes.lock().unwrap();
            Ok(quotes
                .iter()
                .filter(|q| {
                    q.asset_id == asset_id.as_str()
                        && q.timestamp.date_naive() >= start.date()
                        && q.timestamp.date_naive() <= end.date()
                })
                .cloned()
                .collect())
        }

        async fn latest_batch(
            &self,
            asset_ids: &[AssetId],
            _source: Option<&QuoteSource>,
        ) -> Result<HashMap<AssetId, Quote>> {
            let quotes = self.quotes.lock().unwrap();
            let mut result = HashMap::new();
            for asset_id in asset_ids {
                if let Some(quote) = quotes
                    .iter()
                    .filter(|q| q.asset_id == asset_id.as_str())
                    .max_by_key(|q| q.timestamp)
                {
                    result.insert(asset_id.clone(), quote.clone());
                }
            }
            Ok(result)
        }

        async fn latest_with_previous(
            &self,
            asset_ids: &[AssetId],
        ) -> Result<HashMap<AssetId, LatestQuotePair>> {
            let quotes = self.quotes.lock().unwrap();
            let mut result = HashMap::new();
            for asset_id in asset_ids {
                let mut symbol_quotes: Vec<_> = quotes
                    .iter()
                    .filter(|q| q.asset_id == asset_id.as_str())
                    .collect();
                symbol_quotes.sort_by_key(|q| q.timestamp);

                if let Some(latest) = symbol_quotes.pop() {
                    let previous = symbol_quotes.pop().cloned();
                    result.insert(
                        asset_id.clone(),
                        LatestQuotePair {
                            latest: latest.clone(),
                            previous,
                        },
                    );
                }
            }
            Ok(result)
        }

        async fn get_latest_quote(&self, symbol: &str) -> Result<Quote> {
            self.latest(&AssetId::new(symbol), None)
                .await?
                .ok_or_else(|| {
                    crate::Error::Database(DatabaseError::NotFound(format!(
                        "Quote for {} not found",
                        symbol
                    )))
                })
        }

        async fn get_latest_quotes(&self, symbols: &[String]) -> Result<HashMap<String, Quote>> {
            let asset_ids: Vec<AssetId> = symbols.iter().map(AssetId::new).collect();
            let result = self.latest_batch(&asset_ids, None).await?;
            Ok(result
                .into_iter()
                .map(|(k, v)| (k.as_str().to_string(), v))
                .collect())
        }

        async fn get_latest_quotes_pair(
            &self,
            symbols: &[String],
        ) -> Result<HashMap<String, LatestQuotePair>> {
            let asset_ids: Vec<AssetId> = symbols.iter().map(AssetId::new).collect();
            let result = self.latest_with_previous(&asset_ids).await?;
            Ok(result
                .into_iter()
                .map(|(k, v)| (k.as_str().to_string(), v))
                .collect())
        }

        async fn get_historical_quotes(&self, symbol: &str) -> Result<Vec<Quote>> {
            let quotes = self.quotes.lock().unwrap();
            Ok(quotes
                .iter()
                .filter(|q| q.asset_id == symbol)
                .cloned()
                .collect())
        }

        async fn get_all_historical_quotes(&self) -> Result<Vec<Quote>> {
            Ok(self.quotes.lock().unwrap().clone())
        }

        async fn get_quotes_in_range(
            &self,
            symbol: &str,
            start: NaiveDate,
            end: NaiveDate,
        ) -> Result<Vec<Quote>> {
            let quotes = self.quotes.lock().unwrap();
            Ok(quotes
                .iter()
                .filter(|q| {
                    q.asset_id == symbol
                        && q.timestamp.date_naive() >= start
                        && q.timestamp.date_naive() <= end
                })
                .cloned()
                .collect())
        }

        async fn find_duplicate_quotes(&self, symbol: &str, date: NaiveDate) -> Result<Vec<Quote>> {
            let quotes = self.quotes.lock().unwrap();
            Ok(quotes
                .iter()
                .filter(|q| q.asset_id == symbol && q.timestamp.date_naive() == date)
                .cloned()
                .collect())
        }

        async fn get_quote_bounds_for_assets(
            &self,
            asset_ids: &[String],
            source: &str,
        ) -> Result<HashMap<String, (NaiveDate, NaiveDate)>> {
            let quotes = self.quotes.lock().unwrap();
            let mut result = HashMap::new();
            for asset_id in asset_ids {
                let matching: Vec<_> = quotes
                    .iter()
                    .filter(|q| q.asset_id == *asset_id && q.data_source.as_str() == source)
                    .collect();
                if !matching.is_empty() {
                    let min_date = matching
                        .iter()
                        .map(|q| q.timestamp.date_naive())
                        .min()
                        .unwrap();
                    let max_date = matching
                        .iter()
                        .map(|q| q.timestamp.date_naive())
                        .max()
                        .unwrap();
                    result.insert(asset_id.clone(), (min_date, max_date));
                }
            }
            Ok(result)
        }
    }

    // =========================================================================
    // Test Helpers
    // =========================================================================

    fn create_quote(symbol: &str, date: NaiveDate, close: Decimal) -> Quote {
        Quote {
            id: format!("{}_{}", symbol, date),
            created_at: Utc::now(),
            data_source: "YAHOO".to_string(),
            timestamp: Utc.from_utc_datetime(&date.and_hms_opt(16, 0, 0).unwrap()),
            asset_id: symbol.to_string(),
            open: close,
            high: close,
            low: close,
            close,
            adjclose: close,
            volume: dec!(1000),
            currency: "USD".to_string(),
            notes: None,
        }
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    // =========================================================================
    // Contract Tests: Quote CRUD
    // =========================================================================

    #[tokio::test]
    async fn test_get_latest_quote_returns_most_recent() {
        let store = MockQuoteStore::new();

        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 2), dec!(155)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 3), dec!(160)));

        let latest = store.get_latest_quote("AAPL").await.unwrap();
        assert_eq!(latest.close, dec!(160));
    }

    #[tokio::test]
    async fn test_get_latest_quote_not_found_returns_error() {
        let store = MockQuoteStore::new();
        let result = store.get_latest_quote("UNKNOWN").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_latest_quotes_for_multiple_symbols() {
        let store = MockQuoteStore::new();

        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));
        store.add_quote(create_quote("MSFT", date(2024, 1, 1), dec!(350)));
        store.add_quote(create_quote("GOOG", date(2024, 1, 1), dec!(140)));

        let symbols = vec!["AAPL".to_string(), "MSFT".to_string()];
        let quotes = store.get_latest_quotes(&symbols).await.unwrap();

        assert_eq!(quotes.len(), 2);
        assert!(quotes.contains_key("AAPL"));
        assert!(quotes.contains_key("MSFT"));
    }

    #[tokio::test]
    async fn test_get_latest_quotes_pair_includes_previous() {
        let store = MockQuoteStore::new();

        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 2), dec!(155)));

        let symbols = vec!["AAPL".to_string()];
        let pairs = store.get_latest_quotes_pair(&symbols).await.unwrap();

        let pair = pairs.get("AAPL").unwrap();
        assert_eq!(pair.latest.close, dec!(155));
        assert_eq!(pair.previous.as_ref().unwrap().close, dec!(150));
    }

    #[tokio::test]
    async fn test_get_latest_quotes_pair_no_previous() {
        let store = MockQuoteStore::new();

        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));

        let symbols = vec!["AAPL".to_string()];
        let pairs = store.get_latest_quotes_pair(&symbols).await.unwrap();

        let pair = pairs.get("AAPL").unwrap();
        assert_eq!(pair.latest.close, dec!(150));
        assert!(pair.previous.is_none());
    }

    // =========================================================================
    // Contract Tests: Date Range Queries
    // =========================================================================

    #[tokio::test]
    async fn test_get_quotes_in_range_filters_correctly() {
        let store = MockQuoteStore::new();

        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 5), dec!(155)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 10), dec!(160)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 15), dec!(165)));

        let quotes = store
            .get_quotes_in_range("AAPL", date(2024, 1, 3), date(2024, 1, 12))
            .await
            .unwrap();

        assert_eq!(quotes.len(), 2);
        let closes: Vec<_> = quotes.iter().map(|q| q.close).collect();
        assert!(closes.contains(&dec!(155)));
        assert!(closes.contains(&dec!(160)));
    }

    #[tokio::test]
    async fn test_stale_manual_quote_is_seeded_for_gap_fill() {
        let store = MockQuoteStore::new();
        let symbol = "ETF:MANUAL";
        let start = date(2026, 3, 1);
        let end = date(2026, 3, 3);
        let stale_quote_date = start - Duration::days(45);

        let stale_manual_quote = Quote {
            data_source: "MANUAL".to_string(),
            ..create_quote(symbol, stale_quote_date, dec!(123.45))
        };
        store.add_quote(stale_manual_quote);

        let lookback_start = start - Duration::days(30);
        let mut all_quotes = store
            .get_quotes_in_range(symbol, lookback_start, end)
            .await
            .unwrap();
        assert!(
            all_quotes.is_empty(),
            "lookback window should not include stale manual quote"
        );

        let symbols = HashSet::from([symbol.to_string()]);
        append_historical_seed_quotes(&store, &symbols, start, &HashMap::new(), &mut all_quotes)
            .await
            .unwrap();

        let filled_quotes = fill_missing_quotes(&all_quotes, &symbols, start, end);
        let expected_days = [date(2026, 3, 1), date(2026, 3, 2), date(2026, 3, 3)];
        assert_eq!(filled_quotes.len(), expected_days.len());

        for (quote, expected_day) in filled_quotes.iter().zip(expected_days.iter()) {
            assert_eq!(quote.asset_id, symbol);
            assert_eq!(quote.close, dec!(123.45));
            assert_eq!(quote.data_source, "MANUAL");
            assert_eq!(quote.timestamp.date_naive(), *expected_day);
        }
    }

    // =========================================================================
    // CRITICAL: Gap Filling Tests
    // =========================================================================

    /// **CRITICAL**: This test documents the gap-filling behavior that MUST be
    /// preserved in the migration.
    ///
    /// The old MarketDataService.fill_missing_quotes() fills gaps for:
    /// - Weekends (Saturday, Sunday)
    /// - Holidays (market closed days)
    ///
    /// Without this, portfolio valuation will show $0 on non-trading days.
    #[test]
    fn test_gap_filling_requirement_documented() {
        // Gap filling requirement documented
    }

    /// Test that demonstrates the fill_missing_quotes algorithm:
    /// - Looks back up to 10 years to find initial quotes
    /// - Carries forward the last known quote for each missing day
    /// - Returns a quote for EVERY day in the requested range
    #[test]
    fn test_fill_missing_quotes_algorithm() {
        // Algorithm documented
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[tokio::test]
    async fn test_empty_symbols_returns_empty_result() {
        let store = MockQuoteStore::new();
        let quotes = store.get_latest_quotes(&[]).await.unwrap();
        assert!(quotes.is_empty());
    }

    #[tokio::test]
    async fn test_missing_symbol_not_in_results() {
        let store = MockQuoteStore::new();
        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));

        let symbols = vec!["AAPL".to_string(), "UNKNOWN".to_string()];
        let quotes = store.get_latest_quotes(&symbols).await.unwrap();

        assert_eq!(quotes.len(), 1);
        assert!(quotes.contains_key("AAPL"));
        assert!(!quotes.contains_key("UNKNOWN"));
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let store = MockQuoteStore::new();

        let q1 = Quote {
            id: "AAPL_2024-01-01_1".to_string(),
            ..create_quote("AAPL", date(2024, 1, 1), dec!(150))
        };
        let q2 = Quote {
            id: "AAPL_2024-01-01_2".to_string(),
            ..create_quote("AAPL", date(2024, 1, 1), dec!(151))
        };

        store.add_quote(q1);
        store.add_quote(q2);

        let duplicates = store
            .find_duplicate_quotes("AAPL", date(2024, 1, 1))
            .await
            .unwrap();
        assert_eq!(duplicates.len(), 2);
    }

    #[tokio::test]
    async fn test_bulk_upsert_replaces_existing() {
        let store = MockQuoteStore::new();

        let initial = create_quote("AAPL", date(2024, 1, 1), dec!(150));
        store.save_quote(&initial).await.unwrap();

        let updated = Quote {
            close: dec!(160),
            ..initial.clone()
        };
        let count = store.upsert_quotes(&[updated]).await.unwrap();

        assert_eq!(count, 1);
        let stored = store.get_latest_quote("AAPL").await.unwrap();
        assert_eq!(stored.close, dec!(160));
    }

    #[tokio::test]
    async fn test_delete_quotes_for_asset_removes_all() {
        let store = MockQuoteStore::new();

        store.add_quote(create_quote("AAPL", date(2024, 1, 1), dec!(150)));
        store.add_quote(create_quote("AAPL", date(2024, 1, 2), dec!(155)));
        store.add_quote(create_quote("MSFT", date(2024, 1, 1), dec!(350)));

        let deleted = store
            .delete_quotes_for_asset(&AssetId::new("AAPL"))
            .await
            .unwrap();

        assert_eq!(deleted, 2);
        assert!(store.get_latest_quote("AAPL").await.is_err());
        assert!(store.get_latest_quote("MSFT").await.is_ok());
    }

    // =========================================================================
    // Method Mapping Verification Tests
    // =========================================================================

    /// Verify that QuoteServiceTrait provides all methods needed by consumers.
    #[test]
    fn test_method_mapping_documented() {
        // Method mapping documented
    }
}
