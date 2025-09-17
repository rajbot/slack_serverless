# Slack Serverless

A serverless framework for creating Slack apps in Rust, designed for deployment on AWS Lambda with DynamoDB storage. Inspired by the [Bolt for Python](https://github.com/slackapi/bolt-python) framework.

## Features

- 🦀 **Rust-first**: Built with Rust for performance and safety
- ⚡ **Serverless**: Designed for AWS Lambda deployment
- 🗄️ **DynamoDB Integration**: OAuth tokens stored in DynamoDB
- 🔐 **OAuth Support**: Full OAuth 2.0 flow implementation
- 🔧 **Builder Pattern**: Fluent API for app configuration
- 🎯 **Event Handling**: Support for events, commands, actions, and shortcuts
- 🛡️ **Security**: Request signature verification
- 📝 **Structured Logging**: Integration with `tracing` crate

## Quick Start

### Basic App

```rust
use slack_serverless::{App, Context, Say, Ack, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::builder()
        .token_from_env("SLACK_BOT_TOKEN")?
        .signing_secret_from_env("SLACK_SIGNING_SECRET")?
        .build()?;

    // Event handlers would be registered here
    // app.event("app_mention", handle_mention);
    // app.command("/hello", handle_hello);

    // Deploy to Lambda
    app.lambda_handler().run().await?;
    Ok(())
}
```

### OAuth-Enabled App

```rust
use slack_serverless::{App, Result};
use slack_serverless::oauth::dynamodb_store::{DynamoDbInstallationStore, DynamoDbStateStore};

#[tokio::main]
async fn main() -> Result<()> {
    let aws_config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&aws_config);

    let app = App::builder()
        .client_id_from_env("SLACK_CLIENT_ID")?
        .client_secret_from_env("SLACK_CLIENT_SECRET")?
        .signing_secret_from_env("SLACK_SIGNING_SECRET")?
        .scopes(vec!["chat:write", "app_mentions:read"])
        .oauth_settings(|oauth| {
            oauth
                .installation_store(DynamoDbInstallationStore::new(
                    dynamodb_client.clone(),
                    "slack_installations".to_string()
                ))
                .state_store(DynamoDbStateStore::new(
                    dynamodb_client,
                    "slack_oauth_states".to_string()
                ))
        })
        .build()?;

    app.lambda_handler().run().await?;
    Ok(())
}
```

## Project Structure

```
slack_serverless/
├── src/
│   ├── app/           # App builder and configuration
│   ├── adapter/       # Platform adapters (AWS Lambda)
│   ├── oauth/         # OAuth flow and token storage
│   ├── listener/      # Event listeners and routing
│   ├── middleware/    # Middleware pipeline
│   ├── client/        # Slack Web API client
│   ├── context/       # Request context and utilities
│   ├── request/       # Request types
│   ├── response/      # Response types
│   └── error.rs       # Error types
├── examples/          # Example applications
└── tests/            # Tests
```

## Environment Variables

### Required
- `SLACK_SIGNING_SECRET`: Your app's signing secret from Slack
- `SLACK_BOT_TOKEN`: Bot user OAuth token (for single-workspace apps)

### OAuth Apps
- `SLACK_CLIENT_ID`: Your app's client ID
- `SLACK_CLIENT_SECRET`: Your app's client secret
- `SLACK_REDIRECT_URI`: OAuth redirect URI

### Optional
- `INSTALLATIONS_TABLE`: DynamoDB table for installations (default: "slack_installations")
- `OAUTH_STATES_TABLE`: DynamoDB table for OAuth states (default: "slack_oauth_states")

## Deployment

### AWS Lambda

1. **Build for Lambda**:
   ```bash
   cargo lambda build --release
   ```

2. **Deploy**:
   ```bash
   cargo lambda deploy --iam-role arn:aws:iam::YOUR-ACCOUNT:role/lambda-execution-role
   ```

3. **Set up API Gateway** to route requests to your Lambda function

4. **Configure DynamoDB tables** (if using OAuth):
   ```bash
   aws dynamodb create-table \
     --table-name slack_installations \
     --attribute-definitions AttributeName=team_id,AttributeType=S AttributeName=enterprise_id,AttributeType=S \
     --key-schema AttributeName=team_id,KeyType=HASH AttributeName=enterprise_id,KeyType=RANGE \
     --billing-mode PAY_PER_REQUEST
   ```

### IAM Permissions

Your Lambda execution role needs:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "dynamodb:GetItem",
        "dynamodb:PutItem",
        "dynamodb:DeleteItem",
        "dynamodb:Query"
      ],
      "Resource": [
        "arn:aws:dynamodb:*:*:table/slack_installations",
        "arn:aws:dynamodb:*:*:table/slack_oauth_states"
      ]
    }
  ]
}
```

## Examples

- [`basic_app.rs`](examples/basic_app.rs) - Simple bot with event handling
- [`oauth_app.rs`](examples/oauth_app.rs) - Multi-workspace app with OAuth
- [`lambda_deployment.rs`](examples/lambda_deployment.rs) - Production-ready Lambda deployment

## Architecture

### Request Flow

1. **API Gateway** receives Slack webhook
2. **Lambda Handler** processes the request
3. **Signature Verification** validates the request
4. **Request Parsing** converts to internal types
5. **Event Routing** matches handlers
6. **Handler Execution** processes the event
7. **Response** sent back to Slack

### OAuth Flow

1. User initiates installation
2. App redirects to Slack OAuth URL
3. User authorizes the app
4. Slack redirects to callback URL
5. App exchanges code for tokens
6. Tokens stored in DynamoDB
7. Installation complete

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Bolt for Python](https://github.com/slackapi/bolt-python)
- Built with the amazing Rust ecosystem
- Powered by AWS serverless services