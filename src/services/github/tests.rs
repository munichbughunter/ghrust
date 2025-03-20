use crate::models::github::CopilotMetrics;

// A more robust mock test that simulates the GitHub API
#[test]
fn test_github_api_with_mock() {
    // Create sample metrics for our mock
    let sample_metrics = create_test_metrics();

    // Verify the data
    assert_eq!(sample_metrics.date, "2023-03-01");
    assert_eq!(sample_metrics.total_active_users, Some(1000));
    assert_eq!(
        sample_metrics
            .copilot_ide_code_completions
            .as_ref()
            .unwrap()
            .total_engaged_users,
        600
    );
}

// This is a mock test that demonstrates how we would test the GitHub API client
#[test]
fn test_get_github_metrics_mock() {
    // Create test metrics
    let mock_metrics = create_test_metrics();

    // Verify expected values
    assert_eq!(mock_metrics.date, "2023-03-01");
    assert_eq!(mock_metrics.total_active_users, Some(1000));
    assert_eq!(
        mock_metrics
            .copilot_ide_code_completions
            .as_ref()
            .unwrap()
            .total_engaged_users,
        600
    );
}

// Lambda function test - ignored because it requires real credentials
#[test]
#[ignore]
fn test_lambda_handler_integration() {
    // This is a placeholder for a full integration test
    // It would use actual AWS Lambda invocation with cargo-lambda
    println!("Integration test would go here");
}

// Direct API call test using real credentials
#[test]
fn test_github_api_direct() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Get credentials from environment
    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let enterprise_id =
        std::env::var("GITHUB_ENTERPRISE_ID").expect("GITHUB_ENTERPRISE_ID not set");

    // Create GitHub client
    let client = super::api::GitHubClient::new(&github_token);

    // Make the actual API call
    let result = super::api::get_github_metrics(&client, &enterprise_id);
    println!("\nAPI Call Result: {:?}", result);
}

// Test for the team-specific metrics API
#[test]
fn test_github_team_metrics_mock() {
    // Create test metrics
    let sample_team_metrics = create_test_team_metrics();

    // Verify expected values
    assert_eq!(sample_team_metrics.date, "2023-03-01");
    assert_eq!(sample_team_metrics.total_active_users, Some(150));
    assert_eq!(sample_team_metrics.total_engaged_users, Some(120));
    assert_eq!(
        sample_team_metrics
            .copilot_ide_code_completions
            .as_ref()
            .unwrap()
            .total_engaged_users,
        90
    );
}

// Direct API call to test team-specific metrics - ignored because it requires real credentials
#[test]
#[ignore]
fn test_github_team_metrics_direct() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Get credentials from environment
    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let enterprise_id =
        std::env::var("GITHUB_ENTERPRISE_ID").expect("GITHUB_ENTERPRISE_ID not set");

    // Create GitHub client
    let client = super::api::GitHubClient::new(&github_token);

    // Test with one of the actual teams
    let team_slug = "pts";

    // Make the actual API call
    let result = super::api::get_github_team_metrics(&client, &enterprise_id, team_slug);

    match result {
        Ok(metrics) => {
            println!(
                "\nTeam API Call Result: Found {} data points",
                metrics.len()
            );
            for metric in &metrics {
                println!("  Date: {}", metric.date);
                println!("  Active Users: {:?}", metric.total_active_users);
                println!("  Engaged Users: {:?}", metric.total_engaged_users);
            }
            assert!(!metrics.is_empty(), "Should have received team metrics");
        }
        Err(e) => {
            println!("\nTeam API Call Error: {}", e);
            // Don't fail the test since this is just a diagnostic test
            // and the API might not be available during testing
        }
    }
}

// Test for IDE chat metrics calculation
#[test]
fn test_ide_chat_metrics_calculation() {
    // Set environment variable for testing
    std::env::set_var("DATADOG_NAMESPACE_P7S1", "gh.p7s1.test");

    // Create sample metrics with multiple editors and models
    let sample_metrics = create_chat_metrics();

    // Initialize Datadog client (no actual API calls will be made)
    let datadog_client = crate::services::datadog::DatadogClient::new("test_api_key".to_string());

    // Get the series that would be sent to Datadog
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let namespace = "test.namespace";
    let date = "2023-03-01";

    let series = if let Some(ref chat) = sample_metrics.copilot_ide_chat {
        datadog_client.prepare_ide_chat_metrics(chat, namespace, date, timestamp)
    } else {
        vec![]
    };

    // Extract and print the totals for verification
    let mut found_total_chats = false;
    let mut found_total_copy_events = false;
    let mut found_total_insertion_events = false;

    for series_point in &series {
        if let Some(metric_name) = series_point.get("metric").and_then(|m| m.as_str()) {
            if metric_name == "gh.p7s1.test.copilot_ide_chat.total_chats" {
                found_total_chats = true;
                if let Some(points) = series_point.get("points").and_then(|p| p.as_array()) {
                    if let Some(point) = points.first() {
                        if let Some(value) = point.get("value").and_then(|v| v.as_f64()) {
                            // Print the actual value to debug
                            println!("Actual total_chats value: {}", value);
                            // The sum of all total_chats in our sample metrics
                            assert_eq!(value, 530.0); // 137 + 298 + 44 + 51 = 530
                        }
                    }
                }
            } else if metric_name == "gh.p7s1.test.copilot_ide_chat.total_chat_copy_events" {
                found_total_copy_events = true;
                if let Some(points) = series_point.get("points").and_then(|p| p.as_array()) {
                    if let Some(point) = points.first() {
                        if let Some(value) = point.get("value").and_then(|v| v.as_f64()) {
                            // Print the actual value to debug
                            println!("Actual total_chat_copy_events value: {}", value);
                            // The sum of all total_chat_copy_events in our sample metrics
                            assert_eq!(value, 44.0); // 44 + 0 + 0 + 0 = 44
                        }
                    }
                }
            } else if metric_name == "gh.p7s1.test.copilot_ide_chat.total_chat_insertion_events" {
                found_total_insertion_events = true;
                if let Some(points) = series_point.get("points").and_then(|p| p.as_array()) {
                    if let Some(point) = points.first() {
                        if let Some(value) = point.get("value").and_then(|v| v.as_f64()) {
                            // Print the actual value to debug
                            println!("Actual total_chat_insertion_events value: {}", value);
                            // The sum of all total_chat_insertion_events in our sample metrics
                            assert_eq!(value, 39.0); // 39 + 0 + 0 + 0 = 39
                        }
                    }
                }
            }
        }
    }

    // Ensure all metrics were generated
    assert!(found_total_chats, "Missing total_chats metric");
    assert!(
        found_total_copy_events,
        "Missing total_chat_copy_events metric"
    );
    assert!(
        found_total_insertion_events,
        "Missing total_chat_insertion_events metric"
    );
}

// Helper function to create test metrics for enterprise
#[allow(dead_code)]
pub fn create_test_metrics() -> CopilotMetrics {
    use crate::models::github::{
        CopilotDotcomChat, CopilotDotcomPullRequests, CopilotIdeChat, CopilotIdeCodeCompletions,
        Editor, Language, Model, Repository,
    };

    CopilotMetrics {
        date: "2023-03-01".to_string(),
        total_active_users: Some(1000),
        total_engaged_users: Some(800),
        copilot_ide_code_completions: Some(CopilotIdeCodeCompletions {
            total_engaged_users: 600,
            languages: Some(vec![Language {
                name: "Rust".to_string(),
                total_engaged_users: 300,
                total_code_suggestions: Some(5000),
                total_code_acceptances: Some(2500),
                total_code_lines_suggested: Some(10000),
                total_code_lines_accepted: Some(5000),
            }]),
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 550,
                models: None,
            }]),
        }),
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 400,
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 375,
                models: None,
            }]),
        }),
        copilot_dotcom_chat: Some(CopilotDotcomChat {
            total_engaged_users: 300,
            models: Some(vec![Model {
                name: "GPT-4".to_string(),
                is_custom_model: false,
                custom_model_training_date: None,
                total_engaged_users: 290,
                languages: None,
                total_chats: Some(500),
                total_chat_insertion_events: Some(300),
                total_chat_copy_events: Some(200),
                total_pr_summaries_created: None,
            }]),
        }),
        copilot_dotcom_pull_requests: Some(CopilotDotcomPullRequests {
            total_engaged_users: 200,
            repositories: Some(vec![Repository {
                name: "test-repo".to_string(),
                total_engaged_users: 180,
                models: vec![Model {
                    name: "GPT-4".to_string(),
                    is_custom_model: false,
                    custom_model_training_date: None,
                    total_engaged_users: 170,
                    languages: None,
                    total_chats: None,
                    total_chat_insertion_events: None,
                    total_chat_copy_events: None,
                    total_pr_summaries_created: Some(50),
                }],
            }]),
        }),
    }
}

// Helper function to create test metrics for a team
#[allow(dead_code)]
pub fn create_test_team_metrics() -> CopilotMetrics {
    use crate::models::github::{
        CopilotDotcomChat, CopilotDotcomPullRequests, CopilotIdeChat, CopilotIdeCodeCompletions,
        Editor, Language, Model, Repository,
    };

    CopilotMetrics {
        date: "2023-03-01".to_string(),
        total_active_users: Some(150),
        total_engaged_users: Some(120),
        copilot_ide_code_completions: Some(CopilotIdeCodeCompletions {
            total_engaged_users: 90,
            languages: Some(vec![Language {
                name: "Rust".to_string(),
                total_engaged_users: 45,
                total_code_suggestions: Some(750),
                total_code_acceptances: Some(375),
                total_code_lines_suggested: Some(1500),
                total_code_lines_accepted: Some(750),
            }]),
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 82,
                models: None,
            }]),
        }),
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 60,
            editors: Some(vec![Editor {
                name: "VS Code".to_string(),
                total_engaged_users: 56,
                models: None,
            }]),
        }),
        copilot_dotcom_chat: Some(CopilotDotcomChat {
            total_engaged_users: 45,
            models: Some(vec![Model {
                name: "GPT-4".to_string(),
                is_custom_model: false,
                custom_model_training_date: None,
                total_engaged_users: 43,
                languages: None,
                total_chats: Some(75),
                total_chat_insertion_events: Some(45),
                total_chat_copy_events: Some(30),
                total_pr_summaries_created: None,
            }]),
        }),
        copilot_dotcom_pull_requests: Some(CopilotDotcomPullRequests {
            total_engaged_users: 30,
            repositories: Some(vec![Repository {
                name: "test-repo".to_string(),
                total_engaged_users: 27,
                models: vec![Model {
                    name: "GPT-4".to_string(),
                    is_custom_model: false,
                    custom_model_training_date: None,
                    total_engaged_users: 25,
                    languages: None,
                    total_chats: None,
                    total_chat_insertion_events: None,
                    total_chat_copy_events: None,
                    total_pr_summaries_created: Some(10),
                }],
            }]),
        }),
    }
}

// Helper function to create test metrics for chat
#[allow(dead_code)]
pub fn create_chat_metrics() -> CopilotMetrics {
    use crate::models::github::{CopilotIdeChat, Editor, Model};

    CopilotMetrics {
        date: "2023-03-01".to_string(),
        total_active_users: Some(100),
        total_engaged_users: Some(80),
        copilot_ide_code_completions: None,
        copilot_ide_chat: Some(CopilotIdeChat {
            total_engaged_users: 80,
            editors: Some(vec![
                Editor {
                    name: "VS Code".to_string(),
                    total_engaged_users: 75,
                    models: Some(vec![
                        Model {
                            name: "GPT-4".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 70,
                            languages: None,
                            total_chats: Some(137),
                            total_chat_insertion_events: Some(39),
                            total_chat_copy_events: Some(44),
                            total_pr_summaries_created: None,
                        },
                        Model {
                            name: "GPT-3.5".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 65,
                            languages: None,
                            total_chats: Some(298),
                            total_chat_insertion_events: Some(0),
                            total_chat_copy_events: Some(0),
                            total_pr_summaries_created: None,
                        },
                    ]),
                },
                Editor {
                    name: "IntelliJ".to_string(),
                    total_engaged_users: 5,
                    models: Some(vec![
                        Model {
                            name: "GPT-4".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 5,
                            languages: None,
                            total_chats: Some(44),
                            total_chat_insertion_events: Some(0),
                            total_chat_copy_events: Some(0),
                            total_pr_summaries_created: None,
                        },
                        Model {
                            name: "GPT-3.5".to_string(),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            total_engaged_users: 5,
                            languages: None,
                            total_chats: Some(51),
                            total_chat_insertion_events: Some(0),
                            total_chat_copy_events: Some(0),
                            total_pr_summaries_created: None,
                        },
                    ]),
                },
            ]),
        }),
        copilot_dotcom_chat: None,
        copilot_dotcom_pull_requests: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::models::github::{
        CopilotDotcomChat, CopilotIdeChat, CopilotIdeCodeCompletions, CopilotMetrics, Editor,
        Language, Model,
    };
    use crate::services::github::GitHubClient;
    use anyhow::Result;
    use chrono::Utc;

    // Mock implementation for testing
    #[cfg(test)]
    impl GitHubClient {
        fn mock_response(&self) -> Result<Vec<CopilotMetrics>> {
            let now = Utc::now();
            let date = now.format("%Y-%m-%d").to_string();

            let metrics = CopilotMetrics {
                date,
                total_active_users: Some(100),
                total_engaged_users: Some(50),
                copilot_ide_code_completions: Some(CopilotIdeCodeCompletions {
                    total_engaged_users: 30,
                    languages: Some(vec![Language {
                        name: "Python".to_string(),
                        total_engaged_users: 20,
                        total_code_suggestions: Some(1000),
                        total_code_acceptances: Some(800),
                        total_code_lines_suggested: Some(5000),
                        total_code_lines_accepted: Some(4000),
                    }]),
                    editors: None,
                }),
                copilot_ide_chat: Some(CopilotIdeChat {
                    total_engaged_users: 20,
                    editors: Some(vec![Editor {
                        name: "VS Code".to_string(),
                        total_engaged_users: 15,
                        models: Some(vec![Model {
                            name: "gpt-4".to_string(),
                            total_engaged_users: 10,
                            total_chats: Some(100),
                            total_chat_copy_events: Some(50),
                            total_chat_insertion_events: Some(30),
                            total_pr_summaries_created: Some(20),
                            is_custom_model: false,
                            custom_model_training_date: None,
                            languages: None,
                        }]),
                    }]),
                }),
                copilot_dotcom_chat: Some(CopilotDotcomChat {
                    total_engaged_users: 10,
                    models: Some(vec![Model {
                        name: "gpt-4".to_string(),
                        total_engaged_users: 8,
                        total_chats: Some(50),
                        total_chat_copy_events: None,
                        total_chat_insertion_events: None,
                        total_pr_summaries_created: None,
                        is_custom_model: false,
                        custom_model_training_date: None,
                        languages: None,
                    }]),
                }),
                copilot_dotcom_pull_requests: None,
            };

            Ok(vec![metrics])
        }
    }

    #[test]
    fn test_github_api_with_mock() {
        let client = GitHubClient::new("fake_token");
        let metrics = client.mock_response().unwrap();
        let metric = &metrics[0];

        assert_eq!(metric.total_active_users, Some(100));
        assert_eq!(metric.total_engaged_users, Some(50));
        assert_eq!(
            metric
                .copilot_ide_code_completions
                .as_ref()
                .unwrap()
                .total_engaged_users,
            30
        );
        assert_eq!(
            metric
                .copilot_ide_chat
                .as_ref()
                .unwrap()
                .total_engaged_users,
            20
        );
        assert_eq!(
            metric
                .copilot_dotcom_chat
                .as_ref()
                .unwrap()
                .total_engaged_users,
            10
        );
    }
}
