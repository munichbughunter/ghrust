//! # Datadog Metrics Models
//!
//! This module defines data structures for representing metrics in Datadog's API format.
//! These models simplify the process of creating, collecting, and serializing metrics
//! before sending them to Datadog's API.
//!
//! The module provides:
//! - `MetricPoint`: Represents a single metric data point with timestamp, value, and tags
//! - `MetricSeries`: Collects multiple metric points for batch submission
//! - Helper functions for creating standardized tags
//!
//! These models support the Datadog client by handling serialization to the specific
//! JSON format expected by the Datadog API.

use serde_json::{json, Value};

/// Represents a metric series point to be sent to Datadog
///
/// A MetricPoint contains all the information needed to record a single metric
/// observation in Datadog, including:
/// - A name that identifies the metric (e.g., "github.copilot.total_active_users")
/// - A numeric value representing the metric measurement
/// - A timestamp (Unix time in seconds) indicating when the measurement was taken
/// - A collection of tags for filtering and grouping metrics in Datadog dashboards
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    pub timestamp: i64,
    pub tags: Vec<String>,
}

impl MetricPoint {
    /// Create a new metric point
    ///
    /// Constructs a new MetricPoint with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the metric (e.g., "github.copilot.active_users")
    /// * `value` - Numeric value of the metric
    /// * `timestamp` - Unix timestamp in seconds
    /// * `tags` - Vector of tag strings (e.g., ["date:2023-03-01", "source:github"])
    ///
    /// # Returns
    ///
    /// A new `MetricPoint` instance with the provided values
    pub fn new(name: impl Into<String>, value: f64, timestamp: i64, tags: Vec<String>) -> Self {
        Self {
            name: name.into(),
            value,
            timestamp,
            tags,
        }
    }

    /// Convert the metric point to a Datadog API-compatible JSON Value
    ///
    /// Serializes the metric point to the specific JSON structure expected by
    /// Datadog's metrics API. The structure includes:
    /// - The metric name
    /// - The metric type (always "GAUGE" for these metrics)
    /// - An array of points with timestamp and value
    /// - An array of tags for filtering
    ///
    /// # Returns
    ///
    /// A serde_json::Value representing the metric in Datadog's API format
    pub fn to_json(&self) -> Value {
        json!({
            "metric": self.name,
            "type": "GAUGE",
            "points": [
                {
                    "timestamp": self.timestamp,
                    "value": self.value
                }
            ],
            "tags": self.tags
        })
    }
}

/// A collection of metric points to be sent to Datadog
///
/// MetricSeries provides a container for collecting multiple related metrics
/// before converting them to JSON and sending them to Datadog. This allows for
/// batch submission and simplifies the process of working with groups of metrics.
pub struct MetricSeries {
    pub points: Vec<MetricPoint>,
}

impl MetricSeries {
    /// Create a new empty metric series
    ///
    /// Initializes a new MetricSeries with no points.
    ///
    /// # Returns
    ///
    /// An empty `MetricSeries` instance
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add a single metric point to the series
    ///
    /// Appends a MetricPoint to the collection of points in this series.
    ///
    /// # Arguments
    ///
    /// * `point` - The MetricPoint to add to the series
    pub fn add_point(&mut self, point: MetricPoint) {
        self.points.push(point);
    }

    /// Add a metric point for an i64 optional value
    ///
    /// Convenience method that creates and adds a MetricPoint for an optional
    /// i64 value. If the value is None, no point is added.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the metric
    /// * `value` - Optional i64 value for the metric
    /// * `timestamp` - Unix timestamp in seconds
    /// * `tags` - Slice of tag strings to apply to the metric
    ///
    /// # Note
    ///
    /// This method handles i64 to f64 conversion automatically and only adds
    /// a point if the value is present (Some).
    pub fn add_optional_i64_point(
        &mut self,
        name: impl Into<String>,
        value: Option<i64>,
        timestamp: i64,
        tags: &[String],
    ) {
        if let Some(val) = value {
            self.add_point(MetricPoint::new(name, val as f64, timestamp, tags.to_vec()));
        }
    }

    /// Convert the metric series to a vector of JSON Values
    ///
    /// Transforms all points in the series to their JSON representation,
    /// suitable for submission to the Datadog API.
    ///
    /// # Returns
    ///
    /// A vector of serde_json::Value objects, one for each point in the series
    pub fn to_json(&self) -> Vec<Value> {
        self.points.iter().map(|p| p.to_json()).collect()
    }
}

impl Default for MetricSeries {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create standard tags
///
/// Creates a vector of standard tags that should be included with all metrics.
/// This ensures consistency in tagging across different metric types.
///
/// # Arguments
///
/// * `date` - The date string (YYYY-MM-DD) to include as a date tag
///
/// # Returns
///
/// A vector containing standard tags:
/// - date:{date} - Identifies when the metrics were collected
/// - source:github-copilot-metrics - Identifies the source of the metrics
pub fn standard_tags(date: &str) -> Vec<String> {
    vec![
        format!("date:{}", date),
        "source:github-copilot-metrics".to_string(),
    ]
}
