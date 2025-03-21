#[cfg(test)]
mod tests {
    use crate::services::github::create_mock_metrics;

    #[test]
    fn test_create_mock_metrics() {
        let metrics = create_mock_metrics(100, 80);
        assert_eq!(metrics.total_active_users, Some(100));
        assert_eq!(metrics.total_engaged_users, Some(80));
    }
}
