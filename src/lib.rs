// Export modules for external use
pub mod models;
pub mod processors;
pub mod services;

#[cfg(test)]
pub mod tests;

pub use models::github::CopilotMetrics;
