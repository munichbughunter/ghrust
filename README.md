# GitHub Copilot Metrics Lambda

This AWS Lambda function fetches GitHub Copilot usage metrics for an enterprise and sends specific metrics to Datadog. It is triggered by an EventBridge event on a daily schedule.

## Prerequisites

- Rust and Cargo installed
- [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda) installed
- AWS CLI configured with appropriate permissions
- GitHub Enterprise account with access to Copilot metrics
- Datadog account with API access

## Environment Variables

The function requires the following environment variables:

- `GITHUB_TOKEN`: Your GitHub API token with access to enterprise metrics
- `GITHUB_ENTERPRISE`: Your GitHub Enterprise slug/ID
- `DATADOG_API_KEY`: Your Datadog API key
- `DATADOG_API_URL`: Your Datadog API URL
- `DATADOG_PREFIX`: Prefix for metric names (defaults to "github_copilot" if not set)

## Local Development and Testing

For local development and testing, you can use a `.env` file to set the required environment variables:

1. Copy the `.env.example` file to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Edit the `.env` file and add your credentials:
   ```
   GITHUB_TOKEN=your_actual_github_token
   GITHUB_ENTERPRISE=your_actual_enterprise_slug
   DATADOG_API_KEY=your_actual_datadog_api_key
   DATADOG_API_URL=your_datadog_url
   DATADOG_PREFIX=your_metric_prefix
   ```

3. Run the tests:
   ```bash
   cargo test
   ```

4. For more verbose logging during testing, use:
   ```bash
   RUST_LOG=debug cargo test
   ```

Note: The `.env` file is only used for local development and testing. In production, environment variables should be set in the Lambda function configuration.

## Building the Lambda Function

```bash
cargo lambda build --release
```

## Deploying the Lambda Function

```bash
cargo lambda deploy --iam-role <your-lambda-execution-role-arn>
```

## Setting up the EventBridge Rule

Create an EventBridge rule to trigger the Lambda function daily:

```bash
aws events put-rule \
  --name "DailyGitHubCopilotMetricsRule" \
  --schedule-expression "cron(0 1 * * ? *)" \
  --state ENABLED

aws events put-targets \
  --rule "DailyGitHubCopilotMetricsRule" \
  --targets "Id"="1","Arn"="<your-lambda-function-arn>"
```

## Metrics Collected

The function collects and sends the following metrics to Datadog:

- `{prefix}.active_users`: Total number of active GitHub Copilot users
- `{prefix}.engaged_users`: Total number of engaged GitHub Copilot users

Where `{prefix}` is the value of the `DATADOG_PREFIX` environment variable.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
