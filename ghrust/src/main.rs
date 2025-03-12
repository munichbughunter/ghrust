// Generated by Github Copilot
mod models;
mod services;

use anyhow::{anyhow, Result};
#[cfg(debug_assertions)]
use dotenv;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::json;
use services::datadog::DatadogClient;
use services::github::get_github_metrics;
use std::env;
use tracing::{error, info};

// Make function_handler public for testing
pub async fn function_handler(
    _event: LambdaEvent<serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    // Get environment variables
    let github_token = env::var("GITHUB_TOKEN")
        .map_err(|_| anyhow!("GITHUB_TOKEN environment variable not set"))?;
    let enterprise_id = env::var("GITHUB_ENTERPRISE_ID")
        .map_err(|_| anyhow!("GITHUB_ENTERPRISE_ID environment variable not set"))?;

    // Get Datadog environment variables with defaults
    let datadog_api_key = env::var("DATADOG_API_KEY")
        .map_err(|_| anyhow!("DATADOG_API_KEY environment variable not set"))?;

    // Get P7S1 namespace if available (no error if not set)
    let _datadog_namespace_p7s1 =
        env::var("DATADOG_NAMESPACE_P7S1").unwrap_or_else(|_| "gh.p7s1".to_string());
    info!("Using Datadog namespace: {}", _datadog_namespace_p7s1);

    // Call the GitHub API to get metrics
    match get_github_metrics(&github_token, &enterprise_id) {
        Ok(metrics) => {
            info!(
                "Successfully fetched GitHub Copilot metrics: {} data points",
                metrics.len()
            );

            // Send metrics to Datadog
            let datadog_client = DatadogClient::new(datadog_api_key);
            match datadog_client.send_metrics(&metrics, &_datadog_namespace_p7s1) {
                Ok(_) => {
                    info!("Successfully sent metrics to Datadog EU");
                    Ok(json!({
                        "status": "success",
                        "message": format!("Processed {} data points", metrics.len()),
                        "data_points": metrics.len()
                    }))
                }
                Err(e) => {
                    error!("Error sending metrics to Datadog: {:?}", e);
                    Err(Error::from(e))
                }
            }
        }
        Err(err) => {
            error!("Error fetching GitHub metrics: {}", err);
            Err(Error::from(err))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load .env file in development
    #[cfg(debug_assertions)]
    {
        println!("Loading .env file for development");
        dotenv::dotenv().ok();
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    // Run the lambda function
    run(service_fn(function_handler)).await
}
// Generated Code by Github Copilot ends here
