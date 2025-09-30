use thiserror::Error;

#[derive(Debug, Error)]
pub enum CupError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
