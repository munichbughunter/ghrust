//! # Datadog Client Error Types
//!
//! This module defines structured error types for the Datadog client using the `thiserror` crate.
//! It provides specific error variants for different failure modes when sending metrics to
//! Datadog's API, enabling better error handling and more detailed error messages.
//!
//! The primary types defined are:
//! - `DatadogError`: An enum of all possible Datadog client errors
//! - `Result<T>`: A type alias for `std::result::Result<T, DatadogError>`

use thiserror::Error;

/// Datadog client errors that can occur when sending metrics
///
/// This enum captures the various error conditions that can occur when
/// sending metrics to Datadog's API, including network issues, data formatting
/// problems, and API response errors. Each variant includes relevant context.
#[derive(Error, Debug)]
pub enum DatadogError {
    /// Error with timestamp generation or handling
    #[error("Time error: {0}")]
    TimeError(String),

    /// Network or transport error
    #[error("Network error: {0}")]
    Network(String),

    /// HTTP request error with status code
    #[error("HTTP error {0}: {1}")]
    HttpError(u16, String),
}

/// A specialized Result type for Datadog operations
///
/// This type alias is used throughout the Datadog client for consistent
/// error handling and to avoid repeating the error type.
pub type Result<T> = std::result::Result<T, DatadogError>;
