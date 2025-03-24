//! # GitHub API Error Types
//!
//! This module defines structured error types for the GitHub API client using the `thiserror` crate.
//! It provides specific error variants for different failure modes when interacting with
//! GitHub's API, allowing for better error handling and more informative error messages.
//!
//! The primary types defined are:
//! - `GitHubError`: An enum of all possible GitHub API errors
//! - `Result<T>`: A type alias for `std::result::Result<T, GitHubError>`

use thiserror::Error;

/// GitHub API errors that can occur when fetching metrics data
///
/// This enum captures the various error conditions that can occur when
/// interacting with GitHub's API, including HTTP status errors, network issues,
/// and parsing problems. Each variant includes context-specific information.
#[derive(Error, Debug)]
pub enum GitHubError {
    /// Authentication failed due to invalid token
    #[error("Authentication error: Invalid GitHub token - {0}")]
    Authentication(String),

    /// Authorization failed due to insufficient permissions
    #[error("Authorization error: Insufficient permissions - {0}")]
    Authorization(String),

    /// Requested resource was not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Request validation failed
    #[error("Validation error: {0}")]
    Validation(String),

    /// API rate limit was exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Network or transport error occurred
    #[error("Network error: {0}")]
    Network(String),

    /// Error parsing API response
    #[error("Error parsing GitHub {0} metrics: {1}")]
    ParseError(String, String),

    /// Error from HTTP response that couldn't be further classified
    #[error("HTTP error {0}: {1}")]
    HttpError(u16, String),
}

/// A specialized Result type for GitHub API operations
///
/// This type alias is used throughout the GitHub client for consistent
/// error handling and to avoid repeating the error type.
pub type Result<T> = std::result::Result<T, GitHubError>;
