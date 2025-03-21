// Generated by Github Copilot
mod models;
mod processors;
mod services;
#[cfg(test)]
mod tests;

use anyhow::Result;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::env;
use tracing;

use crate::processors::enterprise;
use crate::processors::team;

/// Handler function for AWS Lambda
async fn function_handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    println!("Starting lambda function execution...");

    // Get environment variables
    let github_token = env::var("GITHUB_TOKEN")
        .map_err(|_| Error::from("GITHUB_TOKEN environment variable not set"))?;

    let enterprise_id = env::var("GITHUB_ENTERPRISE_ID")
        .map_err(|_| Error::from("GITHUB_ENTERPRISE_ID environment variable not set"))?;

    let datadog_api_key = env::var("DATADOG_API_KEY")
        .map_err(|_| Error::from("DATADOG_API_KEY environment variable not set"))?;

    let datadog_namespace = env::var("DATADOG_METRIC_NAMESPACE").unwrap_or_else(|_| {
        println!("DATADOG_METRIC_NAMESPACE not set, using default: github.copilot");
        "github.copilot".to_string()
    });

    // Check if we should skip enterprise metrics
    let skip_enterprise = env::var("SKIP_ENTERPRISE_METRICS").is_ok();

    // Get team slugs if provided
    let team_slugs = env::var("GITHUB_TEAM_SLUGS").ok().map(|slugs| {
        slugs
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    });

    // Process enterprise metrics (if not skipped)
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
                println!("Error processing enterprise metrics: {}", e);
                // Continue with team metrics rather than failing completely
            }
        }
    } else {
        println!("Skipping enterprise metrics due to SKIP_ENTERPRISE_METRICS flag");
    }

    // Process team metrics if team slugs are provided
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

    // Return success response
    Ok(json!({
        "statusCode": 200,
        "message": "GitHub Copilot metrics processing completed"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    // Run the lambda service
    lambda_runtime::run(service_fn(function_handler)).await?;

    Ok(())
}
