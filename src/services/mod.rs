//! # Services Module
//!
//! The services module provides client interfaces and utilities for interacting with
//! external APIs and services. This is the core integration layer of the application
//! that handles communication with third-party systems.
//!
//! ## Submodules
//!
//! * `datadog` - Client and utilities for sending metrics to Datadog's monitoring service.
//!   This module provides functionality to format, batch, and transmit GitHub Copilot
//!   usage metrics to Datadog for visualization and analysis.
//!
//! * `github` - Client and utilities for fetching data from the GitHub API.
//!   This module handles authentication, request formation, error handling, and response
//!   parsing when communicating with GitHub's Copilot metrics endpoints.
//!
//! ## Architecture
//!
//! The services in this module are designed to be:
//!
//! 1. Modular - Each external service has its own isolated submodule
//! 2. Testable - Services can be mocked for testing without network dependencies
//! 3. Error-handling - All network and API interactions include proper error handling
//!
//! These services are used by the application's processors to fetch data from GitHub
//! and report processed metrics to Datadog.

// Generated by Github Copilot
pub mod datadog;
pub mod github;
// Generated Code by Github Copilot ends here
