//! # GitHub Test Helpers
//!
//! This module provides helper functions for creating realistic test data for GitHub Copilot metrics.
//! These functions are designed to simplify unit testing by generating pre-populated metrics objects
//! with realistic values that mimic actual API responses.
//!
//! The module includes:
//! - Functions for creating enterprise-level test metrics
//! - Functions for creating team-level test metrics
//! - Specialized metrics for testing chat-related functionality
//! - Functions for simulating GitHub API responses
//!
//! These test helpers are particularly useful for:
//! - Unit testing processors and services without calling the real GitHub API
//! - Testing edge cases and various data scenarios
//! - Ensuring consistent test data across test suites

// Helper functions for tests that simplify test data creation
use crate::models::github::{
    CopilotDotcomChat, CopilotDotcomPullRequests, CopilotIdeChat, CopilotIdeCodeCompletions,
    CopilotMetrics, Editor, Language, Model, Repository,
};
use anyhow::Result;
use chrono::Utc;

/// Create test metrics suitable for enterprise testing
///
/// Generates a fully populated `CopilotMetrics` object with realistic values
/// that represent enterprise-wide Copilot usage. This includes data for
/// IDE code completions, IDE chat, GitHub.com chat, and pull request metrics.
///
/// # Returns
///
/// A `CopilotMetrics` instance with enterprise-scale values (e.g., 1000 active users)
///
/// # Example
///
/// ```
/// let metrics = create_test_metrics();
/// assert_eq!(metrics.total_active_users, Some(1000));
/// ```
pub fn create_test_metrics() -> CopilotMetrics {
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

/// Create test metrics suitable for team testing (smaller numbers)
///
/// Generates a fully populated `CopilotMetrics` object with realistic values
/// that represent team-level Copilot usage. This includes data for all the same
/// categories as enterprise metrics, but with lower numeric values to represent
/// a team rather than an entire enterprise.
///
/// # Returns
///
/// A `CopilotMetrics` instance with team-scale values (e.g., 150 active users)
///
/// # Use Cases
///
/// - Testing team-specific metrics processors
/// - Testing functions that need to handle smaller metric values
/// - Comparing team vs enterprise metrics in the same test
pub fn create_test_team_metrics() -> CopilotMetrics {
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

/// Create test metrics focused on IDE chat features for metrics calculation
///
/// Generates a specialized `CopilotMetrics` object that has detailed IDE chat metrics
/// but minimal or no data for other metrics categories. This is particularly useful
/// for testing chat-specific functionality without the noise of other metrics.
///
/// The metrics contain data for multiple editors (VS Code and IntelliJ) and
/// multiple models (GPT-4 and GPT-3.5) to test complex aggregation logic.
///
/// # Returns
///
/// A `CopilotMetrics` instance with detailed IDE chat metrics but minimal other data
///
/// # Special Features
///
/// - Contains data from multiple editors (VS Code, IntelliJ)
/// - Contains data from multiple models (GPT-3.5, GPT-4)
/// - Has specific numeric values for testing chat metrics calculations
pub fn create_chat_metrics() -> CopilotMetrics {
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

/// Create a realistic API response with the current date
///
/// Generates a vector of metrics that mimics an actual GitHub API response,
/// using the current date for the metrics. This is useful for testing code
/// that expects to process fresh metrics data as it would come from the API.
///
/// # Returns
///
/// A `Result<Vec<CopilotMetrics>>` containing one metrics entry with today's date
///
/// # Features
///
/// - Uses the current date (from `Utc::now()`)
/// - Simulates a successful API response
/// - Contains a subset of metrics that might be returned by the API
///
/// # Use Cases
///
/// - Testing API response handlers
/// - Testing date-sensitive functionality
/// - Simulating successful API calls without mocking
pub fn create_mock_api_response() -> Result<Vec<CopilotMetrics>> {
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();

    Ok(vec![CopilotMetrics {
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
    }])
}

/// Create test metrics with specified active and engaged users
///
/// Generates a `CopilotMetrics` object with customized values for active and engaged users
/// while keeping other metrics the same as the standard test metrics. This allows testing
/// scenarios with specific user counts without manually creating entire metrics objects.
///
/// # Arguments
///
/// * `active_users` - Number of active users to set in the metrics
/// * `engaged_users` - Number of engaged users to set in the metrics
///
/// # Returns
///
/// A `CopilotMetrics` instance with the specified user counts
///
/// # Example
///
/// ```
/// Test a scenario with 500 active users but only 100 engaged users
/// let metrics = create_test_metrics_with_params(500, 100);
/// assert_eq!(metrics.total_active_users, Some(500));
/// assert_eq!(metrics.total_engaged_users, Some(100));
/// ```
pub fn create_test_metrics_with_params(active_users: i64, engaged_users: i64) -> CopilotMetrics {
    let mut metrics = create_test_metrics();
    metrics.total_active_users = Some(active_users);
    metrics.total_engaged_users = Some(engaged_users);
    metrics
}
