//! # GitHub Copilot Metrics Library
//!
//! This library provides functionality for collecting, processing, and reporting
//! GitHub Copilot usage metrics. It's designed to work both as a standalone binary
//! and as a Lambda function.
//!
//! The library exposes modules for working with GitHub and Datadog APIs,
//! as well as processors for different types of metrics.

// Public modules that can be used by external crates
pub mod models;
pub mod processors;
pub mod services;

// Testing modules only included in test builds
#[cfg(test)]
pub mod tests;

pub use models::github::CopilotMetrics;
