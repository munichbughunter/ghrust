# GitHub Copilot Metrics Lambda

A serverless function that retrieves GitHub Copilot metrics and sends them to Datadog for monitoring and analysis.

## Overview

This Lambda function:
1. Fetches Copilot metrics from a GitHub Enterprise instance
2. Optionally fetches team-specific Copilot metrics for defined teams
3. Processes and formats the metrics
4. Sends the metrics to Datadog with appropriate namespace
5. Returns status information about the operation

## Project Structure

```
.
├── src/
│   ├── main.rs                      # Main entry point and Lambda handler
│   ├── processors/                  # Metrics processing logic
│   │   ├── mod.rs                   # Module definition
│   │   ├── enterprise.rs            # Enterprise metrics processing
│   │   └── team.rs                  # Team metrics processing
│   ├── services/                    # External service integrations
│   │   ├── datadog/                 # Datadog API integration
│   │   │   ├── mod.rs               # Module definition
│   │   │   ├── client.rs            # Datadog client implementation
│   │   │   └── models.rs            # Datadog metrics models
│   │   └── github/                  # GitHub API integration
│   │       ├── mod.rs               # Module definition
│   │       ├── api.rs               # GitHub API client
│   │       └── metrics.rs           # Metrics collection functions
│   └── models/                      # Data models
│       └── github.rs                # GitHub metrics models
├── Cargo.toml                       # Project dependencies
├── .env                             # Environment variables for deployment
└── README.md                        # Project documentation
```

## Environment Variables

| Name | Required | Description |
|------|----------|-------------|
| `GITHUB_TOKEN` | Yes | GitHub personal access token with admin:enterprise permissions |
| `GITHUB_ENTERPRISE_ID` | Yes | ID of the GitHub Enterprise organization |
| `GITHUB_TEAM_SLUGS` | No | Comma-separated list of team slugs for team metrics |
| `DATADOG_API_KEY` | Yes | Datadog API key |
| `DATADOG_METRIC_NAMESPACE` | No | Namespace prefix for Datadog metrics (default: github.copilot) |
| `SKIP_ENTERPRISE_METRICS` | No | If set to any value, skips processing enterprise-wide metrics |
| `MOCK_GITHUB_API` | No | If set to any value, uses mock data instead of calling GitHub API |
| `SKIP_DATADOG_TESTS` | No | If set to any value, skips tests that require Datadog API access |

## Testing

The project includes comprehensive test coverage. To run the tests:

```bash
# Run all tests
cargo test

# Skip tests that require Datadog API access
SKIP_DATADOG_TESTS=1 cargo test

# Run ignored tests (tests marked with #[ignore])
cargo test -- --ignored
```

Some tests require API access to GitHub or Datadog and are skipped by default. To run these tests, you need to:
1. Set up the required environment variables
2. Remove the `SKIP_DATADOG_TESTS` environment variable

## Building and Deployment

### Prerequisites

- Rust toolchain
- AWS CLI configured with appropriate permissions
- cargo-lambda (install with `cargo install cargo-lambda`)

### Building

To build the project:

```bash
# Build in debug mode
cargo build

# Build in release mode (optimized for production)
cargo build --release
```

The compiled binary will be available in `target/debug/` or `target/release/` directory, depending on the build mode.

### Deployment

To deploy the application to AWS Lambda:

```bash
# Deploy to AWS Lambda using cargo-lambda
cargo lambda deploy --env-file .env ghrust
```

This command:
- Builds the project in release mode
- Packages it for AWS Lambda (with correct runtime)
- Deploys it to Lambda as a function named 'ghrust'
- Configures environment variables from the .env file

Make sure your .env file contains all the required environment variables listed above.

## Data Flow

1. The Lambda function is triggered (e.g., by a scheduled event)
2. Enterprise-wide metrics are fetched from GitHub API (unless skipped)
3. If team slugs are configured, team-specific metrics are fetched
4. Metrics are processed and formatted
5. Metrics are sent to Datadog with appropriate namespace
6. Function returns a status response

## Metrics Collected

The function collects the following metrics from GitHub and sends them to Datadog:

### Enterprise Metrics
- Total active users
- Total engaged users
- IDE code completions (total and by language)
- IDE chat metrics
- Dotcom chat metrics
- Dotcom pull request metrics

### Team Metrics
- Same metrics as enterprise, but scoped to specific teams
- Team metrics are sent with the namespace: `{base_namespace}.team.{team_slug}`

## Architecture

### Datadog Service
The Datadog service is modularized into:
- `client.rs`: Implements the `DatadogClient` for sending metrics to Datadog
- `models.rs`: Contains data structures for representing metrics
- `mod.rs`: Exports the public interface

This modular design improves code organization and maintainability.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
