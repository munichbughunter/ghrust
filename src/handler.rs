use anyhow::{anyhow, Result};
use lambda_runtime::{Error, LambdaEvent};
use serde_json::json;
use std::env;
use tracing::{error, info};

use crate::services::datadog::DatadogClient;
use crate::services::github::{get_github_metrics, get_github_team_metrics, GitHubClient};

// Make function_handler public for testing
pub async fn function_handler(
    _event: LambdaEvent<serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    // Set up panic handler to log panics rather than just crashing
    std::panic::set_hook(Box::new(|panic_info| {
        println!("PANIC OCCURRED: {:?}", panic_info);
        if let Some(location) = panic_info.location() {
            println!(
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            );
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("Panic message: {}", s);
        }
    }));

    // Always check that required environment variables are set
    // even if using mocks - this ensures test_missing_env_vars test passes
    let github_token = env::var("GITHUB_TOKEN")
        .map_err(|_| anyhow!("GITHUB_TOKEN environment variable not set"))?;
    let enterprise_id = env::var("GITHUB_ENTERPRISE_ID")
        .map_err(|_| anyhow!("GITHUB_ENTERPRISE_ID environment variable not set"))?;
    let datadog_api_key = env::var("DATADOG_API_KEY")
        .map_err(|_| anyhow!("DATADOG_API_KEY environment variable not set"))?;
    let datadog_namespace = env::var("DATADOG_PREFIX")
        .map_err(|_| anyhow!("DATADOG_PREFIX environment variable not set"))?;

    // Create clients
    let datadog_client = DatadogClient::new(datadog_api_key);
    let github_client = GitHubClient::new(&github_token);

    // Check if we should skip enterprise metrics or process only team metrics
    let process_only_teams = std::env::var("PROCESS_ONLY_TEAMS").is_ok();

    if process_only_teams {
        println!("PROCESS_ONLY_TEAMS flag set, skipping enterprise metrics and only processing team metrics");
        process_team_metrics(
            &github_client,
            &datadog_client,
            &enterprise_id,
            &datadog_namespace,
        )
        .await?;
        return Ok(json!({
            "statusCode": 200,
            "message": "GitHub team metrics processed successfully"
        }));
    }

    println!("About to call process_metrics");
    let result = process_metrics(
        &github_client,
        &datadog_client,
        &enterprise_id,
        &datadog_namespace,
    )
    .await;

    println!(
        "Returned from process_metrics, result is: {:?}",
        result.is_ok()
    );
    result
}

// Separate function dedicated to processing team metrics
async fn process_team_metrics(
    github_client: &GitHubClient,
    datadog_client: &DatadogClient,
    enterprise_id: &str,
    datadog_namespace: &str,
) -> Result<(), Error> {
    println!("=== PROCESSING TEAM METRICS ONLY ===");
    let is_test_mode = std::env::var("MOCK_GITHUB_API").is_ok();

    // Fetch team metrics for specific teams
    let teams_env = env::var("GITHUB_TEAM_SLUGS");
    println!("GITHUB_TEAM_SLUGS raw env var result: {:?}", teams_env);

    if let Ok(teams_str) = teams_env {
        println!("Teams string: '{}'", teams_str);
        // Trim whitespace from each team slug
        let teams: Vec<&str> = teams_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        println!("Parsed {} teams: {:?}", teams.len(), teams);

        if teams.is_empty() {
            println!("WARNING: No valid team slugs found after parsing!");
            return Ok(());
        }

        for team_slug in teams {
            println!("Processing team slug: '{}'", team_slug);

            let team_metrics = if is_test_mode {
                // Use mock data for testing
                let now = chrono::Utc::now();
                let date = now.format("%Y-%m-%d").to_string();

                let mock_metric = crate::models::github::CopilotMetrics {
                    date,
                    total_active_users: Some(50),
                    total_engaged_users: Some(40),
                    copilot_ide_code_completions: None,
                    copilot_ide_chat: None,
                    copilot_dotcom_chat: None,
                    copilot_dotcom_pull_requests: None,
                };

                vec![mock_metric]
            } else {
                println!("Fetching GitHub metrics for team {}", team_slug);
                // Use the real API
                match get_github_team_metrics(github_client, enterprise_id, team_slug) {
                    Ok(team_metrics) => team_metrics,
                    Err(e) => {
                        println!(
                            "Error fetching GitHub metrics for team {}: {}",
                            team_slug, e
                        );
                        // Continue with other teams rather than failing completely
                        continue;
                    }
                }
            };

            if team_metrics.is_empty() {
                println!("No metrics data available for team {}", team_slug);
                continue;
            }

            // For each date in the team metrics, send to Datadog
            for metric in &team_metrics {
                println!(
                    "Processing metrics for team {} on date: {}",
                    team_slug, metric.date
                );

                // Create team-specific namespace
                let team_namespace = format!("{}.team.{}", datadog_namespace, team_slug);

                // In test mode, skip sending to Datadog
                if !is_test_mode {
                    // Send team metrics to Datadog
                    match datadog_client.send_metrics(&[metric.clone()], &team_namespace) {
                        Ok(_) => println!(
                            "Successfully sent team metrics for {} to Datadog",
                            team_slug
                        ),
                        Err(e) => {
                            println!("Error sending team metrics to Datadog: {}", e);
                            // Continue with other teams rather than failing completely
                            continue;
                        }
                    }
                }
            }
        }
    } else {
        println!("GITHUB_TEAM_SLUGS environment variable not found");
    }

    println!("=== TEAM METRICS PROCESSING COMPLETED ===");
    Ok(())
}

// Internal function that processes metrics using the provided clients
async fn process_metrics(
    github_client: &GitHubClient,
    datadog_client: &DatadogClient,
    enterprise_id: &str,
    datadog_namespace: &str,
) -> Result<serde_json::Value, Error> {
    let start_time = std::time::Instant::now();
    println!("Starting process_metrics function");

    // Check if we're in test mode
    let is_test_mode = std::env::var("MOCK_GITHUB_API").is_ok();
    // Check if we should skip enterprise metrics
    let skip_enterprise = std::env::var("SKIP_ENTERPRISE_METRICS").is_ok();

    println!("Test mode enabled: {}", is_test_mode);
    println!("Skip enterprise metrics: {}", skip_enterprise);
    println!("Enterprise ID: {}", enterprise_id);
    println!("Datadog namespace: {}", datadog_namespace);

    // Fetch enterprise-wide metrics
    if !skip_enterprise {
        info!("Fetching GitHub metrics for enterprise {}", enterprise_id);
        let metrics = if is_test_mode {
            // Use mock data for testing
            let now = chrono::Utc::now();
            let date = now.format("%Y-%m-%d").to_string();

            let mock_metric = crate::models::github::CopilotMetrics {
                date,
                total_active_users: Some(100),
                total_engaged_users: Some(80),
                copilot_ide_code_completions: None,
                copilot_ide_chat: None,
                copilot_dotcom_chat: None,
                copilot_dotcom_pull_requests: None,
            };

            vec![mock_metric]
        } else {
            println!("Fetching GitHub metrics for enterprise {}", enterprise_id);
            println!("=======================\n");
            // Use the real API
            match get_github_metrics(github_client, enterprise_id) {
                Ok(metrics) => metrics,
                Err(e) => {
                    error!("Error fetching GitHub metrics: {}", e);
                    return Err(Error::from(e));
                }
            }
        };

        if metrics.is_empty() {
            info!("No metrics data available for the enterprise");
        } else {
            // For each date in the metrics, send to Datadog
            for metric in &metrics {
                info!("Processing metrics for date: {}", metric.date);

                // In test mode, skip sending to Datadog
                if !is_test_mode {
                    // Send enterprise-wide metrics to Datadog
                    if let Err(e) =
                        datadog_client.send_metrics(&[metric.clone()], datadog_namespace)
                    {
                        error!("Error sending metrics to Datadog: {}", e);
                        return Err(Error::from(e));
                    }
                    println!("Successfully sent enterprise metrics to Datadog");
                    println!("Function will now attempt to move to team metrics section");
                }
            }
        }
    } else {
        println!("Skipping enterprise metrics due to SKIP_ENTERPRISE_METRICS flag");
    }

    println!("TRANSITION POINT: Finished enterprise metrics, about to start team metrics");
    println!("Elapsed time so far: {:?}", start_time.elapsed());
    println!("=======================================================");

    // Process team metrics directly using our new dedicated function instead of the catch_unwind approach
    if let Err(e) = process_team_metrics(
        github_client,
        datadog_client,
        enterprise_id,
        datadog_namespace,
    )
    .await
    {
        println!("Error processing team metrics: {:?}", e);
        error!("Error processing team metrics: {:?}", e);
        // Continue execution even if team metrics failed
    }

    // Return success response
    println!(
        "Finishing process_metrics function. Total elapsed time: {:?}",
        start_time.elapsed()
    );
    println!("About to return JSON response and exit function");
    Ok(json!({
        "statusCode": 200,
        "message": "GitHub metrics processed successfully"
    }))
}
