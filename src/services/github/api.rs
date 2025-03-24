//! # GitHub API Client
//!
//! This module implements the HTTP client for interacting with GitHub's API
//! specifically for retrieving Copilot metrics data. It provides functionality for:
//!
//! - Authenticating with the GitHub API using personal access tokens
//! - Fetching enterprise-wide Copilot usage metrics
//! - Fetching team-specific Copilot usage metrics
//! - Handling API errors and rate limiting
//! - Logging metric summaries for observability
//!
//! The client uses the `ureq` library for making HTTP requests and handles JSON
//! serialization/deserialization of the GitHub API responses.

use super::error::{GitHubError, Result};
use crate::models::github::CopilotMetrics;
use tracing::{debug, error, info};

/// Client for interacting with the GitHub API
///
/// This client handles authentication, request formation, and response parsing
/// for GitHub's Copilot metrics API endpoints. It supports both enterprise-wide
/// and team-specific metrics retrieval.
///
/// The client is designed to be lightweight, with a focus on the specific
/// endpoints needed for Copilot metrics collection rather than being a
/// general-purpose GitHub API client.
#[derive(Clone)]
pub struct GitHubClient {
    /// GitHub personal access token for authentication
    ///
    /// This token must have the appropriate scopes to access Copilot metrics:
    /// - For enterprise metrics: `admin:enterprise` scope
    /// - For team metrics: `admin:enterprise` and `read:org` scopes
    token: String,
}

impl GitHubClient {
    /// Creates a new GitHub API client with the given access token
    ///
    /// # Arguments
    ///
    /// * `token` - GitHub personal access token with appropriate permissions
    ///   (typically requires `admin:enterprise` scope for Copilot metrics)
    ///
    /// # Returns
    ///
    /// A new `GitHubClient` instance configured with the provided token
    ///
    /// # Example
    ///
    /// ```
    /// use ghrust::services::github::GitHubClient;
    /// let client = GitHubClient::new("ghp_your_personal_access_token");
    /// ```
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    /// Fetches enterprise-wide Copilot metrics
    ///
    /// Retrieves Copilot usage metrics for an entire GitHub Enterprise organization.
    /// The metrics include data about code completions, chat, and pull request activity.
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - ID of the GitHub Enterprise organization (e.g., "123456")
    /// * `since_date` - ISO 8601 date string for filtering metrics (e.g., "2023-01-01")
    ///   Only metrics from this date onward will be returned
    ///
    /// # Returns
    ///
    /// * `Result<Vec<CopilotMetrics>>` - Collection of metrics data points on success,
    ///   or an error if the API request fails or returns invalid data
    ///
    /// # API Endpoint
    ///
    /// `GET /enterprises/{enterprise_id}/copilot/metrics`
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

    /// Fetches team-specific Copilot metrics
    ///
    /// Retrieves Copilot usage metrics for a specific team within a GitHub Enterprise organization.
    /// The metrics include data about code completions, chat, and pull request activity,
    /// filtered to only include team members.
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - ID of the GitHub Enterprise organization (e.g., "123456")
    /// * `team_slug` - Slug of the team to fetch metrics for (e.g., "engineering")
    /// * `since_date` - ISO 8601 date string for filtering metrics (e.g., "2023-01-01")
    ///   Only metrics from this date onward will be returned
    ///
    /// # Returns
    ///
    /// * `Result<Vec<CopilotMetrics>>` - Collection of metrics data points on success,
    ///   or an error if the API request fails or returns invalid data
    ///
    /// # API Endpoint
    ///
    /// `GET /enterprises/{enterprise_id}/team/{team_slug}/copilot/metrics`
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

    /// Core fetch metrics function used by both enterprise and team fetching
    ///
    /// This internal method handles the common logic for fetching metrics from
    /// different endpoints. It configures the HTTP request, handles authorization,
    /// processes the response, and parses the JSON data into CopilotMetrics objects.
    ///
    /// # Arguments
    ///
    /// * `url` - The complete GitHub API URL to fetch metrics from
    /// * `since_date` - ISO 8601 date string for filtering metrics
    /// * `context` - String describing the context ("enterprise" or "team") for logging
    ///
    /// # Returns
    ///
    /// * `Result<Vec<CopilotMetrics>>` - Collection of metrics or an error
    ///
    /// # Errors
    ///
    /// This function may return errors in the following cases:
    /// - Network or transport errors
    /// - HTTP errors (e.g., authentication, authorization, rate limits)
    /// - JSON parsing errors
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
            Ok(resp) => match resp.into_string() {
                Ok(body) => body,
                Err(e) => {
                    return Err(GitHubError::Network(format!(
                        "Failed to read response: {}",
                        e
                    )))
                }
            },
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
            Err(e) => Err(GitHubError::ParseError(context.to_string(), e.to_string())),
        }
    }

    /// Helper function to handle API errors
    ///
    /// Processes HTTP errors from the GitHub API and translates them into
    /// more specific error messages. This provides better diagnostics
    /// for common issues like authentication problems or rate limiting.
    ///
    /// # Arguments
    ///
    /// * `e` - The ureq Error that occurred during the API call
    ///
    /// # Returns
    ///
    /// * `Result<Vec<CopilotMetrics>>` - Always returns an Err with a contextualized message
    ///
    /// # Error Handling
    ///
    /// Different HTTP status codes are translated into specific error types:
    /// - 401: Authentication errors (invalid token)
    /// - 403: Authorization errors (insufficient permissions)
    /// - 404: Resource not found
    /// - 422: Validation errors
    /// - 429: Rate limit exceeded
    fn handle_api_error(&self, e: ureq::Error) -> Result<Vec<CopilotMetrics>> {
        match e {
            ureq::Error::Status(status, response) => {
                let body = response
                    .into_string()
                    .unwrap_or_else(|_| "Could not read response body".to_string());
                error!("HTTP error {}: {}", status, body);

                match status {
                    401 => Err(GitHubError::Authentication(body)),
                    403 => Err(GitHubError::Authorization(body)),
                    404 => Err(GitHubError::NotFound(body)),
                    422 => Err(GitHubError::Validation(body)),
                    429 => Err(GitHubError::RateLimit(body)),
                    _ => Err(GitHubError::HttpError(status, body)),
                }
            }
            ureq::Error::Transport(transport) => {
                error!("Transport error: {}", transport);
                Err(GitHubError::Network(transport.to_string()))
            }
        }
    }

    /// Helper function to log metrics summary
    ///
    /// Provides a concise log of the metrics retrieved, including counts of
    /// active and engaged users for each feature area (IDE code completions,
    /// IDE chat, GitHub.com chat, and GitHub.com pull requests).
    ///
    /// This is useful for quick verification that the metrics are being
    /// retrieved correctly and contain the expected data.
    ///
    /// # Arguments
    ///
    /// * `metrics` - Collection of CopilotMetrics objects to summarize
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
