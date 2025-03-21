use serde_json::{json, Value};

/// Represents a metric series point to be sent to Datadog
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    pub timestamp: i64,
    pub tags: Vec<String>,
}

impl MetricPoint {
    /// Create a new metric point
    pub fn new(name: impl Into<String>, value: f64, timestamp: i64, tags: Vec<String>) -> Self {
        Self {
            name: name.into(),
            value,
            timestamp,
            tags,
        }
    }

    /// Convert the metric point to a Datadog API-compatible JSON Value
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
pub struct MetricSeries {
    pub points: Vec<MetricPoint>,
}

impl MetricSeries {
    /// Create a new empty metric series
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add a single metric point to the series
    pub fn add_point(&mut self, point: MetricPoint) {
        self.points.push(point);
    }

    /// Add a metric point for an i64 optional value
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
pub fn standard_tags(date: &str) -> Vec<String> {
    vec![
        format!("date:{}", date),
        "source:github-copilot-metrics".to_string(),
    ]
}
