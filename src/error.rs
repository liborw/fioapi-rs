use thiserror::Error;

#[derive(Debug, Error)]
pub enum FioError {
    #[error("invalid token length: expected {expected} characters, got {actual}")]
    InvalidTokenLength { expected: usize, actual: usize },

    #[error("invalid date range: start {start} must be before or equal to end {end}")]
    InvalidDateRange {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },

    #[error("invalid parameter: {0}")]
    InvalidParameter(&'static str),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("unexpected http status: {0}")]
    Status(reqwest::StatusCode),

    #[error("invalid or unexpected response format")]
    InvalidResponse,

    #[error("api rejected request: {0}")]
    Api(#[from] ApiError),
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("invalid request (404)")]
    InvalidRequest,

    #[error("time limit exceeded (409)")]
    TimeLimit,

    #[error("too many items (413)")]
    TooManyItems,

    #[error("not authorized (422)")]
    Authorization,

    #[error("invalid token (500)")]
    InvalidToken,

    #[error("unexpected status {0}")]
    UnexpectedStatus(reqwest::StatusCode),
}
