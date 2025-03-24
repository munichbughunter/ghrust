//! # Test Utilities and Tests
//!
//! This module contains test cases for the application, focusing on
//! verifying that core functionality works as expected. The tests
//! in this module verify the behavior of utility functions used
//! throughout the application.
//!
//! Most tests use mock data rather than making real API calls, allowing
//! for fast and reliable test execution without external dependencies.

#[cfg(test)]
mod tests {
    use crate::services::github::create_mock_metrics;

    /// Test the create_mock_metrics function
    ///
    /// This test verifies that the `create_mock_metrics` function correctly
    /// creates a metrics object with the specified active and engaged user counts.
    ///
    /// Specifically, it checks:
    /// - The total_active_users field matches the provided value
    /// - The total_engaged_users field matches the provided value
    ///
    /// This test ensures that mock data generation for testing works reliably,
    /// which is important since many other tests depend on this functionality.
    #[test]
    fn test_create_mock_metrics() {
        let metrics = create_mock_metrics(100, 80);
        assert_eq!(metrics.total_active_users, Some(100));
        assert_eq!(metrics.total_engaged_users, Some(80));
    }
}
