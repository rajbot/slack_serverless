use slack_serverless::{App, Context, Say, Ack, Result};
#[cfg(feature = "oauth")]
use slack_serverless::oauth::dynamodb_store::{DynamoDbInstallationStore, DynamoDbStateStore};
use aws_config;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::init();

    #[cfg(feature = "oauth")]
    {
        // Initialize AWS configuration
        let aws_config = aws_config::load_from_env().await;
        let dynamodb_client = DynamoDbClient::new(&aws_config);

        // Create DynamoDB stores
        let installation_store = DynamoDbInstallationStore::new(
            dynamodb_client.clone(),
            env::var("INSTALLATIONS_TABLE").unwrap_or_else(|_| "slack_installations".to_string()),
        );
        
        let state_store = DynamoDbStateStore::new(
            dynamodb_client,
            env::var("OAUTH_STATES_TABLE").unwrap_or_else(|_| "slack_oauth_states".to_string()),
        );

        // Create app with OAuth configuration
        let app = App::builder()
            .client_id_from_env("SLACK_CLIENT_ID")?
            .client_secret_from_env("SLACK_CLIENT_SECRET")?
            .signing_secret_from_env("SLACK_SIGNING_SECRET")?
            .scopes(vec!["chat:write", "app_mentions:read", "commands"])
            .redirect_uri(env::var("SLACK_REDIRECT_URI").unwrap_or_else(|_| 
                "https://your-lambda-url.amazonaws.com/slack/oauth_redirect".to_string()
            ))
            .oauth_settings(|oauth| {
                oauth
                    .installation_store(installation_store)
                    .state_store(state_store)
            })
            .build()?;

        // Handle app mentions
        // app.event("app_mention", handle_app_mention);

        // Handle slash commands
        // app.command("/deploy", handle_deploy_command);

        info!("Starting OAuth-enabled Slack app...");

        // For Lambda deployment
        #[cfg(feature = "lambda")]
        {
            app.lambda_handler().run().await?;
        }
    }

    #[cfg(not(feature = "oauth"))]
    {
        println!("OAuth feature not enabled. Enable with --features oauth");
    }

    Ok(())
}

// Example handlers
async fn handle_app_mention(context: Context) -> Result<()> {
    context.say.text("Hello! I'm an OAuth-enabled bot!").await?;
    Ok(())
}

async fn handle_deploy_command(context: Context, ack: Ack) -> Result<()> {
    ack.text("🚀 Deployment started! This is a demo command.").await?;
    
    // You could add actual deployment logic here
    // For example, trigger AWS CodePipeline, call GitHub Actions API, etc.
    
    Ok(())
}