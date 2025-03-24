//! # Enterprise Metrics Processor
//!
//! This module handles the processing of enterprise-wide GitHub Copilot metrics.
//! It coordinates the complete workflow of fetching metrics from GitHub's API,
//! processing them, and sending them to Datadog for monitoring and visualization.
//!
//! The enterprise processor is responsible for metrics that cover an entire
//! GitHub Enterprise organization, providing a broad view of Copilot usage
//! across all teams and users within the enterprise.
//!
//! This module serves as a key integration point between the GitHub API client
//! and the Datadog client, managing the end-to-end flow of metrics data.

use anyhow::{anyhow, Result};
use tracing::{debug, info};

use crate::services::{
    datadog::DatadogClient,
    github::{get_enterprise_metrics, GitHubClient},
};

/// Process and send enterprise-wide metrics to Datadog
///
/// This function orchestrates the end-to-end process for enterprise metrics:
/// 1. Initializes the GitHub and Datadog API clients
/// 2. Fetches enterprise-wide Copilot metrics from GitHub
/// 3. Processes and transforms the metrics as needed
/// 4. Sends the processed metrics to Datadog for monitoring
///
/// If the GitHub API returns no metrics, the function will log this and return
/// successfully without attempting to send data to Datadog.
///
/// # Arguments
///
/// * `github_token` - Personal access token for GitHub API authentication
/// * `enterprise_id` - ID of the GitHub Enterprise organization to fetch metrics for
/// * `datadog_api_key` - API key for Datadog authentication
/// * `datadog_namespace` - Namespace prefix for metrics in Datadog (e.g., "github.copilot")
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if processing was successful, or an error with details
///
/// # Errors
///
/// This function may return errors in the following cases:
/// * Unable to fetch metrics from GitHub API
/// * Unable to send metrics to Datadog API
///
/// # Logging
///
/// The function logs its progress at different stages using the tracing crate:
/// * Starting the enterprise metrics processing
/// * Number of metrics data points retrieved
/// * Completion of the metrics processing
pub fn process_enterprise_metrics(
    github_token: &str,
    enterprise_id: &str,
    datadog_api_key: &str,
    datadog_namespace: &str,
) -> Result<()> {
    info!(
        "Starting enterprise metrics processing for {}",
        enterprise_id
    );

    // Initialize clients
    let github_client = GitHubClient::new(github_token);
    let datadog_client = DatadogClient::new(datadog_api_key.to_string());

    // Fetch metrics from GitHub
    let metrics = match get_enterprise_metrics(&github_client, enterprise_id) {
        Ok(metrics) => {
            if metrics.is_empty() {
                debug!("No enterprise metrics returned for {}", enterprise_id);
                return Ok(());
            }
            metrics
        }
        Err(e) => {
            return Err(anyhow!("Failed to fetch enterprise metrics: {}", e));
        }
    };

    info!(
        "Retrieved {} metrics data points for enterprise {}",
        metrics.len(),
        enterprise_id
    );

    // Send metrics to Datadog
    datadog_client.send_metrics(&metrics, datadog_namespace)?;

    info!(
        "Enterprise metrics processing completed for {}",
        enterprise_id
    );
    Ok(())
}
