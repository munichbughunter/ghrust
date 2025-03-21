use anyhow::Result;
use std::env;

// Import only what we need
use ghrust::services::github::{get_team_metrics, GitHubClient};

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
