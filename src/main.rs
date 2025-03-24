// Module declarations for project organization
mod models; // Contains data structures for GitHub and Datadog
mod processors; // Contains business logic for processing metrics
mod services; // Contains API clients for external services
#[cfg(test)] // Test module only included in test builds
mod tests;

// Import necessary dependencies, modules and types
use anyhow::Result;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::env;
use tracing;

// Import processor modules for enterprise and team metrics
use crate::processors::enterprise;
use crate::processors::team;

/// Handler function for AWS Lambda
/// This function is called for each Lambda invocation and orchestrates the metrics collection process
async fn function_handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    println!("Starting lambda function execution...");

    // Get required environment variables for GitHub API authentication
    let github_token = env::var("GITHUB_TOKEN")
        .map_err(|_| Error::from("GITHUB_TOKEN environment variable not set"))?;

    // Get the enterprise ID to identify which GitHub Enterprise instance to query
    let enterprise_id = env::var("GITHUB_ENTERPRISE_ID")
        .map_err(|_| Error::from("GITHUB_ENTERPRISE_ID environment variable not set"))?;

    // Get Datadog API key for sending metrics
    let datadog_api_key = env::var("DATADOG_API_KEY")
        .map_err(|_| Error::from("DATADOG_API_KEY environment variable not set"))?;

    // Get namespace for Datadog metrics or use default if not provided
    // This determines the prefix for all metrics sent to Datadog
    let datadog_namespace = env::var("DATADOG_METRIC_NAMESPACE").unwrap_or_else(|_| {
        println!("DATADOG_METRIC_NAMESPACE not set, using default: github.copilot");
        "github.copilot".to_string()
    });

    // Check if enterprise metrics processing should be skipped
    // This is useful for cases where only team metrics are needed
    let skip_enterprise = env::var("SKIP_ENTERPRISE_METRICS").is_ok();

    // Parse comma-separated team slugs into a vector of strings
    // These identify which teams to collect metrics for
    let team_slugs = env::var("GITHUB_TEAM_SLUGS").ok().map(|slugs| {
        slugs
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    });

    // WORKFLOW STEP 1: Process enterprise-wide metrics if not explicitly skipped
    // These metrics cover all Copilot usage across the entire enterprise
    if !skip_enterprise {
        match enterprise::process_enterprise_metrics(
            &github_token,
            &enterprise_id,
            &datadog_api_key,
            &datadog_namespace,
        ) {
            Ok(_) => {
                println!("Successfully processed enterprise metrics");
            }
            Err(e) => {
                // Log error but continue execution to process team metrics
                // This follows a partial success pattern instead of failing completely
                println!("Error processing enterprise metrics: {}", e);
            }
        }
    } else {
        println!("Skipping enterprise metrics due to SKIP_ENTERPRISE_METRICS flag");
    }

    // WORKFLOW STEP 2: Process team-specific metrics if team slugs are provided
    // These metrics are scoped to individual teams for more granular reporting
    if let Some(slugs) = team_slugs {
        if !slugs.is_empty() {
            match team::process_all_teams(
                &github_token,
                &enterprise_id,
                &slugs,
                &datadog_api_key,
                &datadog_namespace,
            ) {
                Ok(_) => {
                    println!(
                        "Successfully processed team metrics for {} teams",
                        slugs.len()
                    );
                }
                Err(e) => {
                    println!("Error processing team metrics: {}", e);
                }
            }
        } else {
            println!("No team slugs provided, skipping team metrics");
        }
    } else {
        println!("GITHUB_TEAM_SLUGS not set, skipping team metrics");
    }

    // Return success response to Lambda runtime
    // The workflow completes successfully even if some metrics processing failed
    Ok(json!({
        "statusCode": 200,
        "message": "GitHub Copilot metrics processing completed"
    }))
}

/// Main entry point for the Lambda function
/// Sets up tracing and the Lambda runtime service
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing for better observability in AWS Lambda environment
    // This configures the logging format and level
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    // Start the Lambda runtime with our handler function
    // This creates an event loop that processes incoming Lambda events
    lambda_runtime::run(service_fn(function_handler)).await?;

    Ok(())
}
