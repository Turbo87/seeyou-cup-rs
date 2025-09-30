use csv::StringRecord;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CupError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Parse error{}: {error}", line.map(|l| format!(" on line {l}")).unwrap_or_default())]
    Parse { error: String, line: Option<u64> },

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl CupError {
    pub(crate) fn parse_with_line(error: impl Into<String>, line: Option<u64>) -> Self {
        let error = error.into();
        Self::Parse { error, line }
    }

    pub(crate) fn parse(error: impl Into<String>) -> Self {
        Self::parse_with_line(error, None)
    }

    pub(crate) fn parse2(error: impl Into<String>, record: &StringRecord) -> Self {
        let line = record.position().map(|p| p.line());
        Self::parse_with_line(error, line)
    }
}
