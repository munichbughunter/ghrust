//! # Datadog Monitoring Service
//!
//! This module provides client and utilities for sending metrics to Datadog's
//! monitoring service, with a focus on GitHub Copilot usage metrics.
//!
//! ## Core Components
//!
//! * `client` - The main Datadog API client for sending metrics
//! * `models` - Data structures for representing Datadog metrics
//! * `error` - Structured error types for Datadog operations
//!
//! ## Usage
//!
//! The main entry point is the `DatadogClient` which handles authentication,
//! metric formatting, and transmission to Datadog's API.

pub mod client;
mod error;
mod models;

pub use client::DatadogClient;
// pub use error::{DatadogError, Result as DatadogResult};
