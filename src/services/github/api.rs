use crate::models::github::CopilotMetrics;
use anyhow::{anyhow, Result};
use tracing::{debug, error, info};

/// A client for interacting with the GitHub API
#[derive(Clone)]
pub struct GitHubClient {
    token: String,
}

impl GitHubClient {
    /// Creates a new GitHub client with the given token
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    /// Fetches enterprise-wide Copilot metrics from the GitHub API
    pub fn fetch_enterprise_metrics(
        &self,
        enterprise_id: &str,
        since_date: &str,
    ) -> Result<Vec<CopilotMetrics>> {
        let url = format!(
            "https://api.github.com/enterprises/{}/copilot/metrics",
            enterprise_id
        );

        info!("Fetching enterprise metrics for {}", enterprise_id);
        self.fetch_metrics(&url, since_date, "enterprise")
    }

    /// Fetches team-specific Copilot metrics from the GitHub API
    pub fn fetch_team_metrics(
        &self,
        enterprise_id: &str,
        team_slug: &str,
        since_date: &str,
    ) -> Result<Vec<CopilotMetrics>> {
        let url = format!(
            "https://api.github.com/enterprises/{}/team/{}/copilot/metrics",
            enterprise_id, team_slug
        );

        info!("Fetching team metrics for {}/{}", enterprise_id, team_slug);
        self.fetch_metrics(&url, since_date, "team")
    }

    // Core fetch metrics function used by both enterprise and team fetching
    fn fetch_metrics(
        &self,
        url: &str,
        since_date: &str,
        context: &str,
    ) -> Result<Vec<CopilotMetrics>> {
        debug!("Requesting {} metrics from URL: {}", context, url);

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(5))
            .timeout_read(std::time::Duration::from_secs(30))
            .build();

        let response = match agent
            .get(url)
            .query("since", since_date)
            .set("Accept", "application/vnd.github+json")
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("X-GitHub-Api-Version", "2022-11-28")
            .call()
        {
            Ok(resp) => resp.into_string()?,
            Err(e) => return self.handle_api_error(e),
        };

        debug!("Received API response ({} bytes)", response.len());

        match serde_json::from_str::<Vec<CopilotMetrics>>(&response) {
            Ok(metrics) => {
                if metrics.is_empty() {
                    info!("No metrics data available");
                } else {
                    info!("Received {} data points", metrics.len());
                    self.log_metrics_summary(&metrics);
                }
                Ok(metrics)
            }
            Err(e) => Err(anyhow!("Error parsing GitHub {} metrics: {}", context, e)),
        }
    }

    // Helper function to handle API errors
    fn handle_api_error(&self, e: ureq::Error) -> Result<Vec<CopilotMetrics>> {
        match e {
            ureq::Error::Status(status, response) => {
                let body = response
                    .into_string()
                    .unwrap_or_else(|_| "Could not read response body".to_string());
                error!("HTTP error {}: {}", status, body);

                let err_msg = match status {
                    401 => "Authentication error: Invalid GitHub token",
                    403 => "Authorization error: Insufficient permissions",
                    404 => "Not found: Resource does not exist",
                    422 => "Validation error: Unprocessable entity",
                    429 => "Rate limit exceeded: Try again later",
                    _ => "GitHub API error",
                };
                Err(anyhow!("{}: {}", err_msg, body))
            }
            ureq::Error::Transport(transport) => {
                error!("Transport error: {}", transport);
                Err(anyhow!("Network error: {}", transport))
            }
        }
    }

    // Helper function to log metrics summary
    fn log_metrics_summary(&self, metrics: &[CopilotMetrics]) {
        for metric in metrics {
            debug!(
                "Date: {}, Active: {:?}, Engaged: {:?}",
                metric.date, metric.total_active_users, metric.total_engaged_users
            );

            // Log feature-specific metrics using iter + for_each for conciseness
            [
                (
                    "IDE Code",
                    metric
                        .copilot_ide_code_completions
                        .as_ref()
                        .map(|c| c.total_engaged_users),
                ),
                (
                    "IDE Chat",
                    metric
                        .copilot_ide_chat
                        .as_ref()
                        .map(|c| c.total_engaged_users),
                ),
                (
                    "Dotcom Chat",
                    metric
                        .copilot_dotcom_chat
                        .as_ref()
                        .map(|c| c.total_engaged_users),
                ),
                (
                    "Dotcom PR",
                    metric
                        .copilot_dotcom_pull_requests
                        .as_ref()
                        .map(|c| c.total_engaged_users),
                ),
            ]
            .iter()
            .filter_map(|(name, users)| users.map(|u| (name, u)))
            .for_each(|(name, users)| debug!("{} Engaged Users: {}", name, users));
        }
    }
}
