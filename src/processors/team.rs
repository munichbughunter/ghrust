//! # Team Metrics Processing
//!
//! This module handles the processing of GitHub Copilot metrics for specific teams
//! within a GitHub Enterprise instance. It provides functionality to:
//!
//! - Fetch team-specific metrics from the GitHub API
//! - Process the metrics data
//! - Send the metrics to Datadog with team-specific namespace
//!
//! The module is designed to work with both individual teams and multiple teams
//! in batch processing scenarios.

use anyhow::{anyhow, Result};
use tracing::{debug, info};

use crate::services::{
    datadog::DatadogClient,
    github::{get_team_metrics, GitHubClient},
};

/// Process team-specific metrics and send to Datadog
///
/// This function fetches GitHub Copilot metrics for a specific team within an enterprise,
/// processes the data, and sends the metrics to Datadog with a team-specific namespace.
///
/// # Arguments
///
/// * `github_token` - GitHub personal access token with appropriate permissions
/// * `enterprise_id` - ID of the GitHub Enterprise organization
/// * `team_slug` - Slug identifier for the team (used in API paths and metrics namespacing)
/// * `datadog_api_key` - Datadog API key for authentication
/// * `datadog_namespace` - Base namespace prefix for Datadog metrics
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if successful, or an error if any step fails
///
/// # Errors
///
/// Returns an error if:
/// * Unable to fetch team metrics from GitHub
/// * Sending metrics to Datadog fails
pub fn process_team_metrics(
    github_token: &str,
    enterprise_id: &str,
    team_slug: &str,
    datadog_api_key: &str,
    datadog_namespace: &str,
) -> Result<()> {
    info!(
        "Starting team metrics processing for {}/{}",
        enterprise_id, team_slug
    );

    // Initialize clients
    let github_client = GitHubClient::new(github_token);
    let datadog_client = DatadogClient::new(datadog_api_key.to_string());

    // Fetch team metrics from GitHub
    let metrics = match get_team_metrics(&github_client, enterprise_id, team_slug) {
        Ok(metrics) => {
            if metrics.is_empty() {
                debug!(
                    "No team metrics returned for {}/{}",
                    enterprise_id, team_slug
                );
                return Ok(());
            }
            metrics
        }
        Err(e) => {
            return Err(anyhow!("Failed to fetch team metrics: {}", e));
        }
    };

    info!(
        "Retrieved {} team metrics data points for {}/{}",
        metrics.len(),
        enterprise_id,
        team_slug
    );

    // Create team-specific namespace
    let team_namespace = format!("{}.team.{}", datadog_namespace, team_slug);

    // Send metrics to Datadog with team-specific namespace
    datadog_client.send_metrics(&metrics, &team_namespace)?;

    info!(
        "Team metrics processing completed for {}/{}",
        enterprise_id, team_slug
    );
    Ok(())
}

/// Process metrics for multiple teams
///
/// This function iterates through a list of team slugs and processes metrics for each team.
/// It tracks the success and failure count, and returns an error if any team processing fails.
///
/// # Arguments
///
/// * `github_token` - GitHub personal access token with appropriate permissions
/// * `enterprise_id` - ID of the GitHub Enterprise organization
/// * `team_slugs` - Array of team slug identifiers to process
/// * `datadog_api_key` - Datadog API key for authentication
/// * `datadog_namespace` - Base namespace prefix for Datadog metrics
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if all teams processed successfully, or an error if any team fails
///
/// # Errors
///
/// Returns an error if one or more teams could not be processed successfully,
/// including the count of failed teams in the error message.
pub fn process_all_teams(
    github_token: &str,
    enterprise_id: &str,
    team_slugs: &[String],
    datadog_api_key: &str,
    datadog_namespace: &str,
) -> Result<()> {
    info!("Processing metrics for {} teams", team_slugs.len());

    let mut success_count = 0;
    let mut error_count = 0;

    for team_slug in team_slugs {
        match process_team_metrics(
            github_token,
            enterprise_id,
            team_slug,
            datadog_api_key,
            datadog_namespace,
        ) {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                error_count += 1;
                debug!("Error processing team {}: {}", team_slug, e);
            }
        }
    }

    info!(
        "Team metrics processing completed. Successful: {}, Failed: {}",
        success_count, error_count
    );

    if error_count > 0 {
        Err(anyhow!("Failed to process {} teams", error_count))
    } else {
        Ok(())
    }
}
