use csv::StringRecord;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Parse error{}: {}", .0.line.map(|l| format!(" on line {l}")).unwrap_or_default(), .0.message)]
    Parse(ParseIssue),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error(transparent)]
    Csv(#[from] csv::Error),
}

impl From<ParseIssue> for Error {
    fn from(issue: ParseIssue) -> Self {
        Error::Parse(issue)
    }
}

#[derive(Debug, Clone)]
pub struct ParseIssue {
    message: String,
    line: Option<u64>,
}

impl ParseIssue {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        let line = None;
        Self { message, line }
    }

    pub(crate) fn with_record(self, record: &StringRecord) -> Self {
        let message = self.message;
        let line = record.position().map(|p| p.line());
        Self { message, line }
    }
}
