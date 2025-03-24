//! # Team Metrics Test Tool
//!
//! This binary provides a command-line utility for testing the GitHub team metrics
//! functionality. It allows developers to fetch and display metrics for specific
//! teams without sending them to Datadog, which is useful for:
//!
//! - Verifying that the GitHub API integration is working correctly
//! - Debugging team-specific metrics issues
//! - Viewing raw metrics data for specific teams
//!
//! ## Usage
//!
//! 1. Set the required environment variables:
//!    - GITHUB_TOKEN: A valid GitHub personal access token
//!    - GITHUB_ENTERPRISE_ID: ID of the GitHub Enterprise organization
//!    - GITHUB_TEAM_SLUGS: Comma-separated list of team slugs to fetch metrics for
//!
//! 2. Run the binary: `cargo run --bin test_team_metrics`
//!
//! The tool will fetch metrics for each specified team and display a summary
//! of the results, including the date, active users, and engaged users for
//! each metrics data point.

use anyhow::Result;
use std::env;

// Import only what we need
use ghrust::services::github::{get_team_metrics, GitHubClient};

/// Main entry point for the team metrics test tool
///
/// This function:
/// 1. Loads environment variables
/// 2. Initializes a GitHub API client
/// 3. Parses the list of team slugs to process
/// 4. Fetches metrics for each team
/// 5. Displays a summary of the retrieved metrics
///
/// # Returns
///
/// A Result indicating success or containing an error if any step fails
#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let github_token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable not set");
    let enterprise_id = env::var("GITHUB_ENTERPRISE_ID")
        .expect("GITHUB_ENTERPRISE_ID environment variable not set");

    // Create a GitHub client
    let client = GitHubClient::new(&github_token);

    // Process specific team(s)
    let teams_env =
        env::var("GITHUB_TEAM_SLUGS").expect("GITHUB_TEAM_SLUGS environment variable not set");

    println!("Teams string: '{}'", teams_env);

    // Trim whitespace from each team slug
    let teams: Vec<&str> = teams_env
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    println!("Parsed {} teams: {:?}", teams.len(), teams);

    if teams.is_empty() {
        println!("No valid team slugs found");
        return Ok(());
    }

    for team_slug in teams {
        println!("Processing team: {}", team_slug);

        // Fetch team metrics
        match get_team_metrics(&client, &enterprise_id, team_slug) {
            Ok(metrics) => {
                println!("Fetched {} metrics for team {}", metrics.len(), team_slug);

                // Print metrics summary
                for metric in &metrics {
                    println!("Date: {}", metric.date);
                    if let Some(active) = metric.total_active_users {
                        println!("Total Active Users: {}", active);
                    }
                    if let Some(engaged) = metric.total_engaged_users {
                        println!("Total Engaged Users: {}", engaged);
                    }
                    println!("-------------------------");
                }
            }
            Err(e) => println!("Error fetching metrics for team {}: {}", team_slug, e),
        }

        println!("Finished processing team: {}", team_slug);
    }

    Ok(())
}
