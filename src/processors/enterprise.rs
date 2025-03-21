use anyhow::{anyhow, Result};
use tracing::{debug, info};

use crate::services::{
    datadog::DatadogClient,
    github::{get_enterprise_metrics, GitHubClient},
};

/// Process and send enterprise-wide metrics to Datadog
///
/// This function fetches GitHub Copilot metrics for an enterprise,
/// processes them, and sends the processed metrics to Datadog.
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
