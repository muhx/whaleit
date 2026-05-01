use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Splits sum {got} does not match transaction amount {want}")]
    SplitsSumMismatch { got: String, want: String },
    #[error("Transfer source and destination accounts must differ")]
    TransferAccountsIdentical,
    #[error("OFX parse failed: {0}")]
    OfxParse(String),
    #[error("CSV compile failed: {0}")]
    CsvCompile(String),
    #[error("Idempotency key collision (file already imported?): {0}")]
    IdempotencyCollision(String),
}
