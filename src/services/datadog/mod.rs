//! # Datadog API Services
//!
//! This module provides services for sending metrics to Datadog.
//! It includes a client for making API requests and models for representing
//! metrics in Datadog's format.
//!
//! ## Submodules
//! - `client`: Contains the Datadog API client
//! - `models`: Data structures for Datadog metrics

mod client;
mod models;

// Re-export the client for convenience
pub use client::DatadogClient;
