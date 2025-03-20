pub mod api;
pub mod tests;

// Re-export the public functions and types
pub use api::{get_github_metrics, get_github_team_metrics, GitHubClient};
