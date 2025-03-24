//! # Datadog Client
//!
//! This module provides functionality to send GitHub Copilot metrics to Datadog's
//! monitoring service. It handles metric formatting, batching, and transmission
//! using Datadog's HTTP API.
//!
//! The client is responsible for:
//! - Converting GitHub Copilot metrics into Datadog-compatible format
//! - Breaking metrics into appropriate chunks to prevent oversized requests
//! - Handling authentication with the Datadog API
//! - Supporting special cases like different metric namespaces
//! - Managing error scenarios and reporting
//!
//! The primary entry point is the `send_metrics` method, which takes a collection
//! of GitHub Copilot metrics and sends them to Datadog with appropriate formatting.

use super::error::{DatadogError, Result};
use super::models::{standard_tags, MetricPoint, MetricSeries};
use crate::models::github::{
    CopilotDotcomChat, CopilotDotcomPullRequests, CopilotIdeChat, CopilotIdeCodeCompletions,
    CopilotMetrics,
};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// A Datadog client that uses the Datadog HTTP API to send metrics to EU region
///
/// This client handles the whole process of sending metrics to Datadog:
/// - Authentication via API key
/// - Converting metrics to Datadog's format
/// - Batching large requests to avoid hitting API limits
/// - Sending metrics via HTTP POST requests
/// - Logging success/failure for observability
pub struct DatadogClient {
    /// Datadog API key for authentication
    api_key: String,
    /// Datadog API endpoint URL (EU region)
    api_url: String,
}

impl DatadogClient {
    /// Create a new Datadog client for the EU region
    ///
    /// Initializes a client that will communicate with Datadog's EU region API.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Datadog API key for authentication
    ///
    /// # Returns
    ///
    /// A new DatadogClient configured for the EU region API endpoint
    pub fn new(api_key: String) -> Self {
        let api_url = "https://api.datadoghq.eu/api/v2/series".to_string();
        Self { api_key, api_url }
    }

    /// Sends metrics to Datadog
    ///
    /// This is the main entry point for sending GitHub Copilot metrics to Datadog.
    /// It handles the complete process:
    /// 1. Skip sending if in test mode (MOCK_GITHUB_API env var is set)
    /// 2. Get current timestamp for the metrics
    /// 3. Format all metrics for Datadog
    /// 4. Send metrics in appropriate chunks
    /// 5. Log completion status
    ///
    /// # Arguments
    ///
    /// * `metrics` - Array slice of GitHub Copilot metrics to send
    /// * `namespace` - Metric namespace (prefix for all metrics)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success (Ok) or error with details
    ///
    /// # Environment Variables
    ///
    /// * `MOCK_GITHUB_API` - If set, skips actual transmission (for testing)
    pub fn send_metrics(&self, metrics: &[CopilotMetrics], namespace: &str) -> Result<()> {
        info!(
            "Sending {} metrics to Datadog for namespace {}",
            metrics.len(),
            namespace
        );

        // Skip in test mode
        if std::env::var("MOCK_GITHUB_API").is_ok() {
            info!("Test mode: Skipping sending metrics to Datadog");
            return Ok(());
        }

        let timestamp = self.current_timestamp()?;
        let all_series = self.prepare_all_metrics(metrics, namespace, timestamp);
        info!("Prepared {} series for Datadog", all_series.len());

        // Send metrics in chunks to avoid oversized requests
        for (i, chunk) in all_series.chunks(100).enumerate() {
            info!("Sending chunk {} ({} series)", i + 1, chunk.len());
            self.send_metrics_chunk(chunk)?;
        }

        info!("Successfully sent all metrics to Datadog EU API");
        self.log_completion_status(namespace);

        Ok(())
    }

    /// Logs completion status message for observability
    ///
    /// Prints information about the completed metrics transmission to help
    /// with debugging and verification. The message differs based on whether
    /// the metrics are enterprise-wide or team-specific.
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace used for the metrics, which includes
    ///   information about whether this is enterprise or team metrics
    fn log_completion_status(&self, namespace: &str) {
        if !namespace.contains(".team.") {
            println!("ENTERPRISE METRICS CALL: Next should be team metrics. If you don't see team metrics logs, there's an issue");
        } else {
            println!(
                "TEAM METRICS CALL for team: {}",
                namespace.split(".team.").last().unwrap_or("unknown")
            );
        }
    }

    /// Get current Unix timestamp
    ///
    /// Retrieves the current time as a Unix timestamp (seconds since epoch),
    /// which is required for sending metrics to Datadog.
    ///
    /// # Returns
    ///
    /// * `Result<i64>` - The current timestamp as i64 or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the system time cannot be accessed or is before the Unix epoch
    fn current_timestamp(&self) -> Result<i64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .map_err(|e| DatadogError::TimeError(e.to_string()))
    }

    /// Prepares all metrics to be sent to Datadog
    ///
    /// Converts GitHub Copilot metrics to Datadog's format by:
    /// - Adding core metrics (active and engaged users)
    /// - Processing IDE code completions metrics
    /// - Processing IDE chat metrics
    /// - Processing GitHub.com chat metrics
    /// - Processing GitHub.com pull request metrics
    ///
    /// # Arguments
    ///
    /// * `metrics` - Array slice of GitHub Copilot metrics to process
    /// * `namespace` - Metric namespace (prefix for all metrics)
    /// * `timestamp` - Unix timestamp to use for all metrics
    ///
    /// # Returns
    ///
    /// Vector of JSON Values representing the metrics in Datadog's format
    fn prepare_all_metrics(
        &self,
        metrics: &[CopilotMetrics],
        namespace: &str,
        timestamp: i64,
    ) -> Vec<Value> {
        let mut all_series = MetricSeries::new();

        for metric in metrics {
            let date = &metric.date;
            let base_tags = standard_tags(date);

            // Add core metrics (active and engaged users)
            all_series.add_point(MetricPoint::new(
                format!("{}.total_active_users", namespace),
                metric.total_active_users.unwrap_or(0) as f64,
                timestamp,
                base_tags.clone(),
            ));

            all_series.add_point(MetricPoint::new(
                format!("{}.total_engaged_users", namespace),
                metric.total_engaged_users.unwrap_or(0) as f64,
                timestamp,
                base_tags.clone(),
            ));

            // Add component metrics
            if let Some(ref completions) = metric.copilot_ide_code_completions {
                let mut subseries = self.prepare_ide_code_completions_metrics(
                    completions,
                    namespace,
                    date,
                    timestamp,
                );
                self.merge_series(&mut all_series, &mut subseries);
            }

            if let Some(ref ide_chat) = metric.copilot_ide_chat {
                let mut subseries =
                    self.prepare_ide_chat_metrics(ide_chat, namespace, date, timestamp);
                self.merge_series(&mut all_series, &mut subseries);
            }

            if let Some(ref dotcom_chat) = metric.copilot_dotcom_chat {
                let mut subseries =
                    self.prepare_dotcom_chat_metrics(dotcom_chat, namespace, date, timestamp);
                self.merge_series(&mut all_series, &mut subseries);
            }

            if let Some(ref dotcom_pr) = metric.copilot_dotcom_pull_requests {
                let mut subseries =
                    self.prepare_dotcom_pr_metrics(dotcom_pr, namespace, date, timestamp);
                self.merge_series(&mut all_series, &mut subseries);
            }
        }

        all_series.to_json()
    }

    /// Merge one series into another
    ///
    /// Transfers all points from the source series into the target series.
    /// This uses `std::mem::take` to efficiently move the points vector
    /// without unnecessary cloning.
    ///
    /// # Arguments
    ///
    /// * `target` - The destination MetricSeries that will receive the points
    /// * `source` - The source MetricSeries whose points will be moved to the target
    fn merge_series(&self, target: &mut MetricSeries, source: &mut MetricSeries) {
        for point in std::mem::take(&mut source.points) {
            target.add_point(point);
        }
    }

    /// Sends a chunk of metrics to Datadog
    ///
    /// Transmits a batch of metrics to Datadog's API via HTTP POST.
    /// The metrics are sent as a JSON array in the request body.
    ///
    /// # Arguments
    ///
    /// * `series` - Array slice of JSON Values representing metrics to send
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success (Ok) or error with details
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or Datadog returns an error response
    fn send_metrics_chunk(&self, series: &[Value]) -> Result<()> {
        info!("Sending chunk with {} series", series.len());

        let request_body = serde_json::json!({ "series": series });

        match ureq::post(&self.api_url)
            .set("Content-Type", "application/json")
            .set("DD-API-KEY", &self.api_key)
            .send_json(request_body)
        {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ureq::Error::Status(status, response) => {
                    let body = response
                        .into_string()
                        .unwrap_or_else(|_| "Could not read response body".to_string());
                    Err(DatadogError::HttpError(status, body))
                }
                ureq::Error::Transport(transport) => {
                    Err(DatadogError::Network(transport.to_string()))
                }
            },
        }
    }

    /// Prepare IDE code completions metrics
    ///
    /// Converts IDE code completion metrics from GitHub's format to Datadog's format.
    /// This includes:
    /// - Total engaged users for code completions
    /// - Language-specific metrics (suggestions, acceptances, lines)
    /// - Editor-specific metrics
    ///
    /// # Arguments
    ///
    /// * `completions` - The IDE code completions metrics to convert
    /// * `namespace` - Base namespace for the metrics
    /// * `date` - Date string for tagging
    /// * `timestamp` - Unix timestamp for the metrics
    ///
    /// # Returns
    ///
    /// A MetricSeries containing all the processed IDE code completion metrics
    fn prepare_ide_code_completions_metrics(
        &self,
        completions: &CopilotIdeCodeCompletions,
        namespace: &str,
        date: &str,
        timestamp: i64,
    ) -> MetricSeries {
        let mut series = MetricSeries::new();
        let prefix = format!("{}.ide.code_completions", namespace);
        let base_tags = standard_tags(date);

        // Add total engaged users
        series.add_point(MetricPoint::new(
            format!("{}.total_engaged_users", prefix),
            completions.total_engaged_users as f64,
            timestamp,
            base_tags.clone(),
        ));

        // Process languages
        if let Some(languages) = &completions.languages {
            for language in languages {
                let lang_name = &language.name;
                let mut lang_tags = base_tags.clone();
                lang_tags.push(format!("language:{}", lang_name));

                // Add engaged users
                series.add_point(MetricPoint::new(
                    format!("{}.languages.total_engaged_users", prefix),
                    language.total_engaged_users as f64,
                    timestamp,
                    lang_tags.clone(),
                ));

                // Add optional metrics
                series.add_optional_i64_point(
                    format!("{}.languages.total_code_suggestions", prefix),
                    language.total_code_suggestions,
                    timestamp,
                    &lang_tags,
                );

                series.add_optional_i64_point(
                    format!("{}.languages.total_code_acceptances", prefix),
                    language.total_code_acceptances,
                    timestamp,
                    &lang_tags,
                );

                series.add_optional_i64_point(
                    format!("{}.languages.total_code_lines_suggested", prefix),
                    language.total_code_lines_suggested,
                    timestamp,
                    &lang_tags,
                );

                series.add_optional_i64_point(
                    format!("{}.languages.total_code_lines_accepted", prefix),
                    language.total_code_lines_accepted,
                    timestamp,
                    &lang_tags,
                );
            }
        }

        // Process editors
        if let Some(editors) = &completions.editors {
            for editor in editors {
                let editor_name = &editor.name;
                let mut editor_tags = base_tags.clone();
                editor_tags.push(format!("editor:{}", editor_name));

                series.add_point(MetricPoint::new(
                    format!("{}.editors.total_engaged_users", prefix),
                    editor.total_engaged_users as f64,
                    timestamp,
                    editor_tags.clone(),
                ));
            }
        }

        series
    }

    /// Calculate and prepare IDE chat metrics
    ///
    /// Converts IDE chat metrics from GitHub's format to Datadog's format.
    /// This includes:
    /// - Total engaged users for IDE chat
    /// - Editor-specific metrics
    /// - Model-specific metrics within each editor
    /// - P7S1-specific metrics (if environment variable is set)
    ///
    /// # Arguments
    ///
    /// * `ide_chat` - The IDE chat metrics to convert
    /// * `namespace` - Base namespace for the metrics
    /// * `date` - Date string for tagging
    /// * `timestamp` - Unix timestamp for the metrics
    ///
    /// # Returns
    ///
    /// A MetricSeries containing all the processed IDE chat metrics
    ///
    /// # Environment Variables
    ///
    /// * `DATADOG_NAMESPACE_P7S1` - If set, additional metrics are sent with this namespace
    pub fn prepare_ide_chat_metrics(
        &self,
        ide_chat: &CopilotIdeChat,
        namespace: &str,
        date: &str,
        timestamp: i64,
    ) -> MetricSeries {
        let mut series = MetricSeries::new();
        let prefix = format!("{}.ide.chat", namespace);
        let base_tags = standard_tags(date);

        // Add total engaged users
        series.add_point(MetricPoint::new(
            format!("{}.total_engaged_users", prefix),
            ide_chat.total_engaged_users as f64,
            timestamp,
            base_tags.clone(),
        ));

        // Calculate total metrics across all editors
        let (total_chats, total_copies, total_insertions) =
            self.calculate_ide_chat_totals(ide_chat);

        // Add editors with their models
        if let Some(editors) = &ide_chat.editors {
            for editor in editors {
                let editor_name = &editor.name;
                let mut editor_tags = base_tags.clone();
                editor_tags.push(format!("editor:{}", editor_name));

                series.add_point(MetricPoint::new(
                    format!("{}.editors.total_engaged_users", prefix),
                    editor.total_engaged_users as f64,
                    timestamp,
                    editor_tags.clone(),
                ));

                // Process models if present
                if let Some(models) = &editor.models {
                    for model in models {
                        let model_name = &model.name;
                        let is_custom = if model.is_custom_model {
                            "true"
                        } else {
                            "false"
                        };

                        let mut model_tags = editor_tags.clone();
                        model_tags.push(format!("model:{}", model_name));
                        model_tags.push(format!("is_custom_model:{}", is_custom));

                        series.add_point(MetricPoint::new(
                            format!("{}.editors.models.total_engaged_users", prefix),
                            model.total_engaged_users as f64,
                            timestamp,
                            model_tags.clone(),
                        ));

                        // Add PR summaries if present
                        series.add_optional_i64_point(
                            format!("{}.editors.models.total_pr_summaries_created", prefix),
                            model.total_pr_summaries_created,
                            timestamp,
                            &model_tags,
                        );
                    }
                }
            }
        }

        // Add P7S1 specific metrics if environment variable exists
        if let Ok(p7s1_namespace) = std::env::var("DATADOG_NAMESPACE_P7S1") {
            series.add_point(MetricPoint::new(
                format!("{}.copilot_ide_chat.total_chats", p7s1_namespace),
                total_chats as f64,
                timestamp,
                base_tags.clone(),
            ));

            series.add_point(MetricPoint::new(
                format!("{}.copilot_ide_chat.total_chat_copy_events", p7s1_namespace),
                total_copies as f64,
                timestamp,
                base_tags.clone(),
            ));

            series.add_point(MetricPoint::new(
                format!(
                    "{}.copilot_ide_chat.total_chat_insertion_events",
                    p7s1_namespace
                ),
                total_insertions as f64,
                timestamp,
                base_tags,
            ));
        }

        series
    }

    /// Calculate total metrics for IDE chat
    ///
    /// Calculates aggregate metrics by summing values across all editors and models.
    /// This is used for producing total metrics across all IDE chat usage.
    ///
    /// # Arguments
    ///
    /// * `ide_chat` - The IDE chat metrics to calculate totals for
    ///
    /// # Returns
    ///
    /// A tuple of (total_chats, total_copies, total_insertions) as i64 values
    fn calculate_ide_chat_totals(&self, ide_chat: &CopilotIdeChat) -> (i64, i64, i64) {
        let mut total_chats = 0;
        let mut total_copies = 0;
        let mut total_insertions = 0;

        if let Some(editors) = &ide_chat.editors {
            for editor in editors {
                if let Some(models) = &editor.models {
                    for model in models {
                        if let Some(chats) = model.total_chats {
                            total_chats += chats;
                        }
                        if let Some(copies) = model.total_chat_copy_events {
                            total_copies += copies;
                        }
                        if let Some(insertions) = model.total_chat_insertion_events {
                            total_insertions += insertions;
                        }
                    }
                }
            }
        }

        (total_chats, total_copies, total_insertions)
    }

    /// Prepare metrics for GitHub.com chat
    ///
    /// Converts GitHub.com chat metrics from GitHub's format to Datadog's format.
    /// This includes:
    /// - Total engaged users for GitHub.com chat
    /// - Model-specific metrics (engaged users, total chats)
    ///
    /// # Arguments
    ///
    /// * `chat` - The GitHub.com chat metrics to convert
    /// * `namespace` - Base namespace for the metrics
    /// * `date` - Date string for tagging
    /// * `timestamp` - Unix timestamp for the metrics
    ///
    /// # Returns
    ///
    /// A MetricSeries containing all the processed GitHub.com chat metrics
    fn prepare_dotcom_chat_metrics(
        &self,
        chat: &CopilotDotcomChat,
        namespace: &str,
        date: &str,
        timestamp: i64,
    ) -> MetricSeries {
        let mut series = MetricSeries::new();
        let prefix = format!("{}.dotcom.chat", namespace);
        let base_tags = standard_tags(date);

        // Add total engaged users
        series.add_point(MetricPoint::new(
            format!("{}.total_engaged_users", prefix),
            chat.total_engaged_users as f64,
            timestamp,
            base_tags.clone(),
        ));

        // Add model metrics if models are available
        if let Some(models) = &chat.models {
            for model in models {
                let model_name = &model.name;
                let is_custom = if model.is_custom_model {
                    "true"
                } else {
                    "false"
                };

                let mut model_tags = base_tags.clone();
                model_tags.push(format!("model:{}", model_name));
                model_tags.push(format!("is_custom_model:{}", is_custom));

                series.add_point(MetricPoint::new(
                    format!("{}.models.total_engaged_users", prefix),
                    model.total_engaged_users as f64,
                    timestamp,
                    model_tags.clone(),
                ));

                series.add_optional_i64_point(
                    format!("{}.models.total_chats", prefix),
                    model.total_chats,
                    timestamp,
                    &model_tags,
                );
            }
        }

        series
    }

    /// Prepare metrics for GitHub.com pull requests
    ///
    /// Converts GitHub.com pull request metrics from GitHub's format to Datadog's format.
    /// This includes:
    /// - Total engaged users for GitHub.com pull requests
    /// - Repository-specific metrics
    /// - Model-specific metrics within each repository
    ///
    /// # Arguments
    ///
    /// * `pr` - The GitHub.com pull request metrics to convert
    /// * `namespace` - Base namespace for the metrics
    /// * `date` - Date string for tagging
    /// * `timestamp` - Unix timestamp for the metrics
    ///
    /// # Returns
    ///
    /// A MetricSeries containing all the processed GitHub.com pull request metrics
    fn prepare_dotcom_pr_metrics(
        &self,
        pr: &CopilotDotcomPullRequests,
        namespace: &str,
        date: &str,
        timestamp: i64,
    ) -> MetricSeries {
        let mut series = MetricSeries::new();
        let prefix = format!("{}.dotcom.pull_requests", namespace);
        let base_tags = standard_tags(date);

        // Add total engaged users
        series.add_point(MetricPoint::new(
            format!("{}.total_engaged_users", prefix),
            pr.total_engaged_users as f64,
            timestamp,
            base_tags.clone(),
        ));

        // Add repository metrics if repositories are available
        if let Some(repositories) = &pr.repositories {
            for repo in repositories {
                let repo_name = &repo.name;
                let mut repo_tags = base_tags.clone();
                repo_tags.push(format!("repository:{}", repo_name));

                series.add_point(MetricPoint::new(
                    format!("{}.repositories.total_engaged_users", prefix),
                    repo.total_engaged_users as f64,
                    timestamp,
                    repo_tags.clone(),
                ));

                for model in &repo.models {
                    let model_name = &model.name;
                    let is_custom = if model.is_custom_model {
                        "true"
                    } else {
                        "false"
                    };

                    let mut model_tags = repo_tags.clone();
                    model_tags.push(format!("model:{}", model_name));
                    model_tags.push(format!("is_custom_model:{}", is_custom));

                    series.add_point(MetricPoint::new(
                        format!("{}.repositories.models.total_engaged_users", prefix),
                        model.total_engaged_users as f64,
                        timestamp,
                        model_tags.clone(),
                    ));

                    series.add_optional_i64_point(
                        format!("{}.repositories.models.total_pr_summaries_created", prefix),
                        model.total_pr_summaries_created,
                        timestamp,
                        &model_tags,
                    );
                }
            }
        }

        series
    }
}
