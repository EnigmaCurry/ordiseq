use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrdiseqError {
    #[error("Invalid time signature: {0}")]
    InvalidTimeSignature(String),
}
