//! # GitHub API Services
//!
//! This module provides services for interacting with GitHub's API,
//! specifically for retrieving Copilot metrics.
//!
//! ## Submodules
//! - `api`: Contains the GitHub API client for making HTTP requests
//! - `metrics`: Functions for processing raw API responses into metric objects

pub mod api;
mod metrics;

#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;

// Re-export public items
pub use api::GitHubClient;
pub use metrics::{get_enterprise_metrics, get_team_metrics};
#[cfg(test)]
pub use test_helpers::create_test_metrics_with_params as create_mock_metrics;
