//! # GitHub API Service
//!
//! This module provides client and utilities for interacting with GitHub's API,
//! especially for fetching Copilot metrics data.
//!
//! ## Core Components
//!
//! * `api` - The main GitHub API client for fetching metrics
//! * `error` - Structured error types for GitHub API operations
//!
//! ## Usage
//!
//! The main entry point is the `GitHubClient` which handles authentication and
//! request formation when interacting with GitHub's API.

pub mod api;
mod error;
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
