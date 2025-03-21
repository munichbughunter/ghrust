// GitHub metrics processing functions
use anyhow::Result;
use chrono::{Duration, Utc};
use tracing::info;

use super::api::GitHubClient;
use crate::models::github::CopilotMetrics;

/// Fetches enterprise-wide Copilot metrics from GitHub
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
fn calculate_default_since_date() -> String {
    let thirty_days_ago = Utc::now() - Duration::days(30);
    thirty_days_ago.format("%Y-%m-%d").to_string()
}
