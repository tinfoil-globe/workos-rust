use std::error::Error as StdError;
use std::fmt;

use reqwest::Error as ReqwestError;
use thiserror::Error;

/// Additional context for HTTP failures.
#[derive(Debug)]
pub struct RequestError {
    message: String,
    source: Option<ReqwestError>,
}

impl RequestError {
    /// Creates a new `RequestError` with the provided message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a new `RequestError` including the originating [`ReqwestError`].
    pub fn with_source(message: impl Into<String>, source: ReqwestError) -> Self {
        Self {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Returns the human-readable message associated with this error.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for RequestError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &(dyn StdError + 'static))
    }
}

impl From<ReqwestError> for RequestError {
    fn from(error: ReqwestError) -> Self {
        let message = match error.url() {
            Some(url) => format!("request to {} failed: {}", url, error),
            None => format!("request failed: {}", error),
        };

        RequestError::with_source(message, error)
    }
}

/// A WorkOS SDK error.
#[derive(Debug, Error)]
pub enum WorkOsError<E> {
    /// An error occurred with the current operation.
    #[error("operational error")]
    Operation(E),

    /// An unauthorized response was received from the WorkOS API.
    #[error("unauthorized")]
    Unauthorized,

    /// The request was rate limited by the WorkOS API.
    #[error("rate limited")]
    RateLimited {
        /// Seconds until the request may be retried, if provided by the API.
        retry_after: Option<f32>,
    },

    /// An error occurred while parsing a URL.
    #[error("URL parse error")]
    UrlParseError(#[from] url::ParseError),

    /// An error occurred while parsing an IP address.
    #[error("IP addres parse error")]
    IpAddrParseError(#[from] std::net::AddrParseError),

    /// An unhandled error occurred with the API request.
    #[error("{0}")]
    RequestError(#[from] RequestError),
}

/// A WorkOS SDK result.
pub type WorkOsResult<T, E> = Result<T, WorkOsError<E>>;

impl<E> From<ReqwestError> for WorkOsError<E> {
    fn from(error: ReqwestError) -> Self {
        WorkOsError::RequestError(RequestError::from(error))
    }
}
