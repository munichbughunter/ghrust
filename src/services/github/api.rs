use crate::models::github::{
    CopilotDotcomChat, CopilotIdeChat, CopilotIdeCodeCompletions, CopilotMetrics, Language,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use tracing::error;

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
}

/// Fetches GitHub Copilot metrics for the entire enterprise
///
/// This function queries the GitHub API endpoint for enterprise-wide Copilot metrics
/// and returns the results as a vector of CopilotMetrics.
pub fn get_github_metrics(
    client: &GitHubClient,
    enterprise_id: &str,
) -> Result<Vec<CopilotMetrics>> {
    // Calculate yesterday's date in ISO 8601 format
    let now: DateTime<Utc> = Utc::now();
    let yesterday = now - Duration::days(1);
    let since_date = yesterday.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let url = format!(
        "https://api.github.com/enterprises/{}/copilot/metrics",
        enterprise_id
    );

    let response = ureq::get(&url)
        .query("since", &since_date)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {}", client.token))
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call();

    match response {
        Ok(response) => {
            let response_text = response.into_string()?;
            println!("\nResponse Body:");
            println!("{}", response_text);
            println!("=======================\n");

            // Parse the text into JSON
            let metrics: Vec<CopilotMetrics> = match serde_json::from_str(&response_text) {
                Ok(metrics) => {
                    println!("\n=== Metrics for Datadog ===");
                    for metric in &metrics {
                        let metric: &CopilotMetrics = metric;
                        println!("Date: {}", metric.date);
                        if let Some(active) = metric.total_active_users {
                            println!("Total Active Users: {}", active);
                        }
                        if let Some(engaged) = metric.total_engaged_users {
                            println!("Total Engaged Users: {}", engaged);
                        }
                        if let Some(ref completions) = metric.copilot_ide_code_completions {
                            let completions: &CopilotIdeCodeCompletions = completions;
                            println!(
                                "IDE Code Completions Engaged Users: {}",
                                completions.total_engaged_users
                            );

                            if let Some(ref languages) = completions.languages {
                                println!("\nLanguage Breakdown:");
                                for lang in languages {
                                    let lang: &Language = lang;
                                    println!(
                                        "  {} - {} users",
                                        lang.name, lang.total_engaged_users
                                    );
                                }
                            }
                        }
                        if let Some(ref chat) = metric.copilot_ide_chat {
                            let chat: &CopilotIdeChat = chat;
                            println!("IDE Chat Engaged Users: {}", chat.total_engaged_users);
                        }
                        if let Some(ref chat) = metric.copilot_dotcom_chat {
                            let chat: &CopilotDotcomChat = chat;
                            println!("Dotcom Chat Engaged Users: {}", chat.total_engaged_users);
                        }
                        println!("------------------------");
                    }
                    println!("=======================\n");
                    metrics
                }
                Err(e) => {
                    error!("Failed to parse JSON response: {}", e);
                    return Err(anyhow!("Failed to read JSON: {}", e));
                }
            };
            Ok(metrics)
        }
        Err(ureq::Error::Status(403, _)) => {
            Err(anyhow!("Forbidden: Not authorized to access this resource"))
        }
        Err(ureq::Error::Status(404, _)) => {
            Err(anyhow!("Not Found: The requested resource does not exist"))
        }
        Err(ureq::Error::Status(422, _)) => Err(anyhow!(
            "Unprocessable Entity: Copilot Usage Metrics API setting is disabled"
        )),
        Err(ureq::Error::Status(status, response)) => {
            if let Ok(response_text) = response.into_string() {
                error!("Error response from GitHub API: {}", response_text);
            }
            Err(anyhow!("Error: Received status code: {}", status))
        }
        Err(e) => Err(anyhow!("Error fetching GitHub metrics: {}", e)),
    }
}

/// Fetches GitHub Copilot metrics for a specific team
///
/// This function queries the GitHub API endpoint for team-specific Copilot metrics
/// and returns the results as a vector of CopilotMetrics.
pub fn get_github_team_metrics(
    client: &GitHubClient,
    enterprise_id: &str,
    team_slug: &str,
) -> Result<Vec<CopilotMetrics>> {
    // Calculate yesterday's date in ISO 8601 format
    let now: DateTime<Utc> = Utc::now();
    let yesterday = now - Duration::days(1);
    let since_date = yesterday.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let url = format!(
        "https://api.github.com/enterprises/{}/team/{}/copilot/metrics",
        enterprise_id, team_slug
    );

    let response = ureq::get(&url)
        .query("since", &since_date)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {}", client.token))
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call();

    match response {
        Ok(response) => {
            let response_text = response.into_string()?;
            println!("\nTeam {} Response Body:", team_slug);
            println!("{}", response_text);
            println!("=======================\n");

            // Parse the text into JSON
            let metrics: Vec<CopilotMetrics> = match serde_json::from_str(&response_text) {
                Ok(metrics) => {
                    println!("\n=== Team {} Metrics for Datadog ===", team_slug);
                    for metric in &metrics {
                        let metric: &CopilotMetrics = metric;
                        println!("Date: {}", metric.date);
                        if let Some(active) = metric.total_active_users {
                            println!("Total Active Users: {}", active);
                        }
                        if let Some(engaged) = metric.total_engaged_users {
                            println!("Total Engaged Users: {}", engaged);
                        }
                        println!("------------------------");
                    }
                    println!("=======================\n");
                    metrics
                }
                Err(e) => {
                    error!(
                        "Failed to parse JSON response for team {}: {}",
                        team_slug, e
                    );
                    return Err(anyhow!("Failed to read JSON for team {}: {}", team_slug, e));
                }
            };
            Ok(metrics)
        }
        Err(ureq::Error::Status(403, _)) => Err(anyhow!(
            "Forbidden: Not authorized to access team {} resource",
            team_slug
        )),
        Err(ureq::Error::Status(404, _)) => Err(anyhow!(
            "Not Found: The requested team {} resource does not exist",
            team_slug
        )),
        Err(ureq::Error::Status(422, _)) => Err(anyhow!(
            "Unprocessable Entity: Copilot Usage Metrics API setting is disabled for team {}",
            team_slug
        )),
        Err(ureq::Error::Status(status, response)) => {
            if let Ok(response_text) = response.into_string() {
                error!(
                    "Error response from GitHub API for team {}: {}",
                    team_slug, response_text
                );
            }
            Err(anyhow!(
                "Error: Received status code {} for team {}",
                status,
                team_slug
            ))
        }
        Err(e) => Err(anyhow!(
            "Error fetching GitHub metrics for team {}: {}",
            team_slug,
            e
        )),
    }
}
