//! # Metric Processors
//!
//! This module contains processors that handle the collection, transformation,
//! and reporting of GitHub Copilot metrics. Each processor is responsible for
//! a specific scope of metrics processing.
//!
//! The processors serve as the main business logic layer of the application,
//! coordinating between the GitHub API services (to fetch metrics) and the
//! Datadog services (to report metrics).
//!
//! ## Submodules
//!
//! * `enterprise` - Processes enterprise-wide GitHub Copilot metrics.
//!   Handles fetching aggregated metrics for an entire GitHub Enterprise
//!   organization and sending them to Datadog with the appropriate namespace.
//!
//! * `team` - Processes team-specific GitHub Copilot metrics.
//!   Handles fetching metrics for individual teams within a GitHub Enterprise
//!   organization and sending them to Datadog with team-specific namespaces.
//!
//! ## Architecture
//!
//! The processors follow these general steps:
//! 1. Initialize the necessary service clients (GitHub, Datadog)
//! 2. Fetch metrics from GitHub using the appropriate scope (enterprise or team)
//! 3. Apply any necessary transformations or calculations to the metrics
//! 4. Send the processed metrics to Datadog with the appropriate namespace
//! 5. Return success or error information

// This module contains processors for different metrics
pub mod enterprise;
pub mod team;
