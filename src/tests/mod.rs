#[cfg(test)]
mod tests {
    use crate::handler::function_handler;
    use lambda_runtime::{Context, LambdaEvent};
    use serde_json::json;

    // Test helper to mock the GitHub API
    fn setup_mocks() {
        // Replace the actual implementation with one that returns a mock
        #[cfg(test)]
        {
            // Mock the GitHub client to return test data
            let _ = std::env::set_var("MOCK_GITHUB_API", "true");
        }
    }

    // Test helper to cleanup environment variables after tests
    fn cleanup_env_vars() {
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GITHUB_ENTERPRISE_ID");
        std::env::remove_var("DATADOG_API_KEY");
        std::env::remove_var("DATADOG_PREFIX");
        std::env::remove_var("GITHUB_TEAMS");
        std::env::remove_var("MOCK_GITHUB_API");
    }

    // Setup standard test environment variables
    fn setup_test_env_vars() {
        std::env::set_var("GITHUB_TOKEN", "fake_token");
        std::env::set_var("GITHUB_ENTERPRISE_ID", "fake_enterprise");
        std::env::set_var("DATADOG_API_KEY", "fake_api_key");
        std::env::set_var("DATADOG_PREFIX", "test.prefix");
    }

    // Test the main handler function
    #[tokio::test]
    async fn test_function_handler_mock() {
        cleanup_env_vars();
        setup_test_env_vars();
        std::env::set_var("GITHUB_TEAMS", "team1,team2");
        setup_mocks();

        // Create a fake event
        let event = LambdaEvent {
            payload: json!({}),
            context: Context::default(),
        };

        // Call the handler function
        let result = function_handler(event).await;

        // Print error details if it failed
        if let Err(ref e) = result {
            println!("Test failed with error: {}", e);
        }

        // Verify the result
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response["statusCode"], 200);
            assert!(response["message"]
                .as_str()
                .unwrap()
                .contains("GitHub metrics processed successfully"));
        }

        cleanup_env_vars();
    }

    // Test for missing environment variables
    #[tokio::test]
    async fn test_missing_env_vars() {
        cleanup_env_vars();

        // Make sure no environment variables are set
        // The MOCK_GITHUB_API is not set to ensure we're not using mocks

        // Create a fake event
        let event = LambdaEvent {
            payload: json!({}),
            context: Context::default(),
        };

        // Call the handler function
        let result = function_handler(event).await;

        // Verify that the function returns an error
        assert!(result.is_err());

        // Check the specific error message for GITHUB_TOKEN
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("GITHUB_TOKEN"),
            "Error should mention GITHUB_TOKEN but got: {}",
            err
        );

        cleanup_env_vars();
    }

    // Test with real API call - ignored because it needs real credentials
    #[tokio::test]
    #[ignore]
    async fn test_real_api_call() {
        cleanup_env_vars();
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        // Create a fake event
        let event = LambdaEvent {
            payload: json!({}),
            context: Context::default(),
        };

        // Call the handler function
        let result = function_handler(event).await;

        // Verify the result
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response["statusCode"], 200);
        }

        cleanup_env_vars();
    }

    // Test with no teams specified
    #[tokio::test]
    async fn test_no_teams_specified() {
        cleanup_env_vars();
        setup_test_env_vars();
        // Explicitly ensure GITHUB_TEAMS is not set
        std::env::remove_var("GITHUB_TEAMS");
        setup_mocks();

        // Create a fake event
        let event = LambdaEvent {
            payload: json!({}),
            context: Context::default(),
        };

        // Call the handler function
        let result = function_handler(event).await;

        // Verify the result - should still succeed
        assert!(result.is_ok());

        cleanup_env_vars();
    }
}
