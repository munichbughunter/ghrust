//! # GitHub Metrics Processing
//!
//! This module provides functionality for retrieving and processing GitHub Copilot metrics data.
//! It serves as a higher-level interface to the GitHub API client, handling date calculations
//! and providing simplified methods for both enterprise-wide and team-specific metrics.
//!
//! The module offers:
//! - Enterprise-wide metrics collection
//! - Team-specific metrics collection
//! - Default date range calculation (last 30 days)
//!
//! This module abstracts away some of the complexity of the raw API client,
//! making it easier to fetch metrics in common scenarios.

// GitHub metrics processing functions
use anyhow::Result;
use chrono::{Duration, Utc};
use tracing::info;

use super::api::GitHubClient;
use crate::models::github::CopilotMetrics;

/// Fetches enterprise-wide Copilot metrics from GitHub
///
/// This function retrieves Copilot usage metrics for an entire GitHub Enterprise organization.
/// It automatically calculates a sensible default date range (30 days prior to today)
/// and uses the GitHub API client to fetch the metrics.
///
/// # Arguments
///
/// * `client` - A reference to an authenticated GitHubClient instance
/// * `enterprise_id` - ID of the GitHub Enterprise organization
///
/// # Returns
///
/// * `Result<Vec<CopilotMetrics>>` - A collection of metrics on success, or an error
///   if the API request fails
///
/// # Example
///
/// ```
/// use ghrust::services::github::{GitHubClient, get_enterprise_metrics};
/// # use anyhow::Result;
/// # fn example() -> Result<()> {
/// let client = GitHubClient::new("your-github-token");
/// let metrics = get_enterprise_metrics(&client, "12345")?;
/// # Ok(())
/// # }
/// ```
pub fn get_enterprise_metrics(
    client: &GitHubClient,
    enterprise_id: &str,
) -> Result<Vec<CopilotMetrics>> {
    // Calculate a reasonable date range (usually 30 days back)
    let since_date = calculate_default_since_date();

    // Fetch the metrics
    let metrics = client.fetch_enterprise_metrics(enterprise_id, &since_date)?;

    // Log summary information
    info!("Retrieved {} enterprise metric entries", metrics.len());

    Ok(metrics)
}

/// Fetches team-specific Copilot metrics from GitHub
///
/// This function retrieves Copilot usage metrics for a specific team within a
/// GitHub Enterprise organization. It automatically calculates a sensible default
/// date range (30 days prior to today) and uses the GitHub API client to fetch
/// the metrics for the specified team.
///
/// # Arguments
///
/// * `client` - A reference to an authenticated GitHubClient instance
/// * `enterprise_id` - ID of the GitHub Enterprise organization
/// * `team_slug` - Slug identifier for the team to fetch metrics for
///
/// # Returns
///
/// * `Result<Vec<CopilotMetrics>>` - A collection of team-specific metrics on success,
///   or an error if the API request fails
///
/// # Notes
///
/// Team metrics contain the same types of data as enterprise metrics, but are filtered
/// to only include data from members of the specified team.
pub fn get_team_metrics(
    client: &GitHubClient,
    enterprise_id: &str,
    team_slug: &str,
) -> Result<Vec<CopilotMetrics>> {
    // Calculate a reasonable date range (usually 30 days back)
    let since_date = calculate_default_since_date();

    // Fetch the metrics
    let metrics = client.fetch_team_metrics(enterprise_id, team_slug, &since_date)?;

    // Log summary information
    info!(
        "Retrieved {} team metric entries for team {}",
        metrics.len(),
        team_slug
    );

    Ok(metrics)
}

/// Calculate a default "since" date (30 days back from today)
///
/// This helper function generates an ISO 8601 date string (YYYY-MM-DD format)
/// representing 30 days before the current date. This provides a reasonable
/// default time period for fetching metrics.
///
/// # Returns
///
/// * `String` - ISO 8601 formatted date string (e.g., "2023-01-01")
///
/// # Notes
///
/// The 30-day window is a balance between getting enough historical data
/// and keeping API response sizes manageable.
fn calculate_default_since_date() -> String {
    let thirty_days_ago = Utc::now() - Duration::days(30);
    thirty_days_ago.format("%Y-%m-%d").to_string()
}
