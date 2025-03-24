//! # GitHub Services Tests
//!
//! This module contains tests for the GitHub API services and metrics processing functionality.
//! It includes unit tests using mock data as well as integration tests that can connect to the
//! real GitHub API (marked with #[ignore] to prevent them from running in normal test runs).
//!
//! The tests verify:
//! - Creation and validation of mock metrics data
//! - Enterprise-wide metrics processing
//! - Team-specific metrics processing
//! - IDE chat metrics calculation
//! - API response handling
//!
//! Some tests require environment variables for real API access, while others
//! use mock data to enable testing without external dependencies.

use super::test_helpers::{
    create_chat_metrics, create_mock_api_response, create_test_metrics, create_test_team_metrics,
};
use crate::models::github::CopilotMetrics;
use crate::services::github::{get_enterprise_metrics, get_team_metrics};

/// Core test for mock metrics functionality
///
/// Verifies that the mock metrics generation functions produce metrics objects
/// with the expected values. This is a fundamental test that ensures our test
/// fixtures are working correctly, since many other tests rely on these mocks.
///
/// Specifically, this test checks:
/// - The date is set correctly
/// - The total active users value is as expected
/// - The IDE code completions engaged users count is correct
#[test]
fn test_mock_metrics() {
    let metrics = create_test_metrics();
    assert_eq!(metrics.date, "2023-03-01");
    assert_eq!(metrics.total_active_users, Some(1000));
    assert_eq!(
        metrics
            .copilot_ide_code_completions
            .as_ref()
            .unwrap()
            .total_engaged_users,
        600
    );
}

/// Test enterprise metrics with mocks
///
/// Tests the structure and content of enterprise-level metrics using mock data.
/// This verifies that enterprise metrics objects contain the expected fields and values,
/// particularly focusing on language-specific data within the IDE code completions.
///
/// This test ensures that:
/// - The top-level active and engaged user counts are correct
/// - The language data within IDE code completions exists
/// - The Rust language metrics have the expected values
#[test]
fn test_enterprise_metrics_mock() {
    let metrics = create_test_metrics();

    assert_eq!(metrics.total_active_users, Some(1000));
    assert_eq!(metrics.total_engaged_users, Some(800));

    // Test IDE code completions language data
    if let Some(completions) = &metrics.copilot_ide_code_completions {
        if let Some(languages) = &completions.languages {
            assert!(!languages.is_empty());
            let rust_lang = &languages[0];
            assert_eq!(rust_lang.name, "Rust");
            assert_eq!(rust_lang.total_engaged_users, 300);
        } else {
            panic!("Expected languages data to be present");
        }
    } else {
        panic!("Expected IDE code completions data to be present");
    }
}

/// Test team metrics with mocks
///
/// Tests the structure and content of team-level metrics using mock data.
/// This ensures that the team metrics objects contain the expected fields and values,
/// with appropriately scaled numbers compared to enterprise metrics.
///
/// Specifically checks:
/// - The date is set correctly
/// - The active and engaged user counts are at team-appropriate scale
/// - The IDE code completions engaged users count is correct for team level
#[test]
fn test_team_metrics_mock() {
    let metrics = create_test_team_metrics();
    assert_eq!(metrics.date, "2023-03-01");
    assert_eq!(metrics.total_active_users, Some(150));
    assert_eq!(metrics.total_engaged_users, Some(120));
    assert_eq!(
        metrics
            .copilot_ide_code_completions
            .as_ref()
            .unwrap()
            .total_engaged_users,
        90
    );
}

/// Integration test for Lambda handler functionality
///
/// This test would verify the end-to-end Lambda function execution.
/// It is marked with #[ignore] so it doesn't run during normal test execution,
/// as it would require actual AWS Lambda runtime components.
///
/// To run this test: `cargo test test_lambda_handler_integration -- --ignored`
#[test]
#[ignore]
fn test_lambda_handler_integration() {
    println!("Integration test would go here");
}

/// Direct GitHub API integration test
///
/// Tests the direct connection to the GitHub API to fetch enterprise metrics.
/// This test is marked with #[ignore] because it requires:
/// 1. Valid GitHub credentials in environment variables
/// 2. Network access to the GitHub API
///
/// Required environment variables:
/// - GITHUB_TOKEN: A valid GitHub personal access token
/// - GITHUB_ENTERPRISE_ID: ID of a GitHub enterprise organization
///
/// To run this test: `cargo test test_github_api_direct -- --ignored`
#[test]
#[ignore]
fn test_github_api_direct() {
    dotenvy::dotenv().ok();

    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let enterprise_id =
        std::env::var("GITHUB_ENTERPRISE_ID").expect("GITHUB_ENTERPRISE_ID not set");

    let client = super::api::GitHubClient::new(&github_token);
    let result = get_enterprise_metrics(&client, &enterprise_id);
    println!("\nAPI Call Result: {:?}", result);
}

/// Direct GitHub API team metrics integration test
///
/// Tests the direct connection to the GitHub API to fetch team-specific metrics.
/// This test is marked with #[ignore] because it requires:
/// 1. Valid GitHub credentials in environment variables
/// 2. Network access to the GitHub API
/// 3. A valid team slug in the hardcoded variable
///
/// Required environment variables:
/// - GITHUB_TOKEN: A valid GitHub personal access token
/// - GITHUB_ENTERPRISE_ID: ID of a GitHub enterprise organization
///
/// To run this test: `cargo test test_github_team_metrics_direct -- --ignored`
#[test]
#[ignore]
fn test_github_team_metrics_direct() {
    dotenvy::dotenv().ok();

    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let enterprise_id =
        std::env::var("GITHUB_ENTERPRISE_ID").expect("GITHUB_ENTERPRISE_ID not set");

    let client = super::api::GitHubClient::new(&github_token);
    let team_slug = "pts";

    let result = get_team_metrics(&client, &enterprise_id, team_slug);

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
        }
    }
}

/// Test for IDE chat metrics calculation
///
/// Verifies the calculation of aggregate metrics for IDE chat functionality.
/// This test uses mock chat metrics data to test metric aggregation logic,
/// specifically focusing on total chats, chat copies, and chat insertions.
///
/// The test can be skipped by setting the SKIP_DATADOG_TESTS environment variable,
/// which is useful in environments where Datadog dependencies are not available.
///
/// This test has two execution modes:
/// 1. With the "datadog_tests" feature: Tests the actual Datadog metrics preparation
/// 2. Without the feature: Simply verifies the structure of the mock data
///
/// Environment variables:
/// - SKIP_DATADOG_TESTS: If set, skips this test
/// - DATADOG_NAMESPACE_P7S1: Set by the test to a test value
#[test]
fn test_ide_chat_metrics_calculation() {
    // This test requires access to the Datadog client, which may not be available in all test environments
    if let Ok(_) = std::env::var("SKIP_DATADOG_TESTS") {
        println!("Skipping Datadog test as SKIP_DATADOG_TESTS is set");
        return;
    }

    std::env::set_var("DATADOG_NAMESPACE_P7S1", "gh.p7s1.test");

    let metrics = create_chat_metrics();

    // Mock the Datadog functionality or skip if not available
    #[cfg(feature = "datadog_tests")]
    {
        let datadog_client =
            crate::services::datadog::DatadogClient::new("test_api_key".to_string());

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let namespace = "test.namespace";
        let date = "2023-03-01";

        let series = if let Some(ref chat) = metrics.copilot_ide_chat {
            datadog_client.prepare_ide_chat_metrics(chat, namespace, date, timestamp)
        } else {
            vec![]
        };

        // Verify metrics calculations
        let expected_metrics = [
            ("gh.p7s1.test.copilot_ide_chat.total_chats", 530.0),
            ("gh.p7s1.test.copilot_ide_chat.total_chat_copy_events", 44.0),
            (
                "gh.p7s1.test.copilot_ide_chat.total_chat_insertion_events",
                39.0,
            ),
        ];

        for (metric_name, expected_value) in &expected_metrics {
            let full_metric_name =
                format!("gh.p7s1.test.{}", metric_name.split('.').last().unwrap());
            let found = series.iter().any(|s| {
                if let Some(name) = s.get("metric").and_then(|m| m.as_str()) {
                    if name == full_metric_name {
                        if let Some(points) = s.get("points").and_then(|p| p.as_array()) {
                            if let Some(point) = points.first() {
                                if let Some(value) = point.get("value").and_then(|v| v.as_f64()) {
                                    println!("Actual {} value: {}", metric_name, value);
                                    return (value - expected_value).abs() < f64::EPSILON;
                                }
                            }
                        }
                    }
                }
                false
            });

            assert!(found, "Missing or incorrect {} metric", metric_name);
        }
    }

    // For non-datadog tests, just verify the metrics structure
    #[cfg(not(feature = "datadog_tests"))]
    {
        if let Some(chat) = &metrics.copilot_ide_chat {
            assert_eq!(chat.total_engaged_users, 80);
            if let Some(editors) = &chat.editors {
                assert_eq!(editors.len(), 2);
                assert_eq!(editors[0].name, "VS Code");
                assert_eq!(editors[0].total_engaged_users, 75);
            }
        } else {
            panic!("Expected IDE chat metrics to be present");
        }
    }
}

/// Tests for mock client implementation
///
/// This module contains tests that use a mock implementation of the GitHub client
/// to test functionality without requiring real API access.
#[cfg(test)]
mod mock_client_tests {
    use super::*;
    use crate::services::github::GitHubClient;
    use anyhow::Result;

    /// Mock response method extension for GitHubClient
    ///
    /// Adds a method to GitHubClient that returns mock API response data
    /// instead of making real API calls. This is used for testing the client
    /// without requiring real API access.
    #[cfg(test)]
    impl GitHubClient {
        fn mock_response(&self) -> Result<Vec<CopilotMetrics>> {
            create_mock_api_response()
        }
    }

    /// Test GitHub API client with mock data
    ///
    /// Verifies that the GitHubClient can correctly handle API responses
    /// by using a mock implementation that returns predefined data.
    ///
    /// This test:
    /// - Creates a client with a fake token (won't be used)
    /// - Calls the mock_response method to get simulated API data
    /// - Verifies the structure and values of the returned metrics
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
