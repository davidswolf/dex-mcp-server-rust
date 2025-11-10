//! Error types for the Dex MCP Server.
//!
//! This module defines custom error types using `thiserror` for precise error handling.

use thiserror::Error;

/// Errors that can occur when interacting with the Dex API.
#[derive(Error, Debug)]
pub enum DexApiError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    /// API returned an error status code
    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    /// Failed to parse JSON response
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Network timeout
    #[error("Request timeout")]
    Timeout,

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Authentication failed
    #[error("Authentication failed")]
    Unauthorized,

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Generic API error with context
    #[error("API error: {0}")]
    Other(String),
}

/// Errors that can occur during configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Required environment variable is missing
    #[error("Missing required environment variable: {0}")]
    MissingVar(String),

    /// Environment variable has invalid value
    #[error("Invalid value for {var}: {reason}")]
    InvalidValue { var: String, reason: String },

    /// Failed to load .env file
    #[error("Failed to load .env file: {0}")]
    DotenvError(String),

    /// Generic configuration error
    #[error("Configuration error: {0}")]
    Other(String),
}

/// Errors that can occur during fuzzy matching operations.
#[derive(Error, Debug)]
pub enum MatchingError {
    /// Invalid search query
    #[error("Invalid search query: {0}")]
    InvalidQuery(String),

    /// No matches found
    #[error("No matches found")]
    NoMatches,

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Generic matching error
    #[error("Matching error: {0}")]
    Other(String),
}

/// Errors that can occur during search operations.
#[derive(Error, Debug)]
pub enum SearchError {
    /// Index not ready
    #[error("Search index not ready")]
    IndexNotReady,

    /// Invalid search parameters
    #[error("Invalid search parameters: {0}")]
    InvalidParameters(String),

    /// Search execution failed
    #[error("Search execution failed: {0}")]
    ExecutionError(String),

    /// Generic search error
    #[error("Search error: {0}")]
    Other(String),
}

/// Convenience type alias for Results with DexApiError
pub type DexApiResult<T> = Result<T, DexApiError>;

/// Convenience type alias for Results with ConfigError
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Convenience type alias for Results with MatchingError
pub type MatchingResult<T> = Result<T, MatchingError>;

/// Convenience type alias for Results with SearchError
pub type SearchResult<T> = Result<T, SearchError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DexApiError::NotFound("contact".to_string());
        assert_eq!(err.to_string(), "Resource not found: contact");

        let err = ConfigError::MissingVar("DEX_API_KEY".to_string());
        assert_eq!(
            err.to_string(),
            "Missing required environment variable: DEX_API_KEY"
        );

        let err = MatchingError::NoMatches;
        assert_eq!(err.to_string(), "No matches found");

        let err = SearchError::IndexNotReady;
        assert_eq!(err.to_string(), "Search index not ready");
    }

    #[test]
    fn test_api_error_variants() {
        let err = DexApiError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        assert!(err.to_string().contains("404"));
        assert!(err.to_string().contains("Not found"));
    }
}
