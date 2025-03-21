use super::models::{standard_tags, MetricPoint, MetricSeries};
use crate::models::github::{
    CopilotDotcomChat, CopilotDotcomPullRequests, CopilotIdeChat, CopilotIdeCodeCompletions,
    CopilotMetrics,
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// A Datadog client that uses the Datadog HTTP API to send metrics to EU region
pub struct DatadogClient {
    api_key: String,
    api_url: String,
}

impl DatadogClient {
    /// Create a new Datadog client for the EU region
    pub fn new(api_key: String) -> Self {
        let api_url = "https://api.datadoghq.eu/api/v2/series".to_string();
        Self { api_key, api_url }
    }

    /// Sends metrics to Datadog
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
    fn current_timestamp(&self) -> Result<i64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .map_err(|e| anyhow!("Time error: {}", e))
    }

    /// Prepares all metrics to be sent to Datadog
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
    fn merge_series(&self, target: &mut MetricSeries, source: &mut MetricSeries) {
        for point in std::mem::take(&mut source.points) {
            target.add_point(point);
        }
    }

    /// Sends a chunk of metrics to Datadog
    fn send_metrics_chunk(&self, series: &[Value]) -> Result<()> {
        info!("Sending chunk with {} series", series.len());

        let request_body = serde_json::json!({ "series": series });

        match ureq::post(&self.api_url)
            .set("Content-Type", "application/json")
            .set("DD-API-KEY", &self.api_key)
            .send_json(request_body)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to send metrics to Datadog: {}", e)),
        }
    }

    /// Prepare IDE code completions metrics
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
