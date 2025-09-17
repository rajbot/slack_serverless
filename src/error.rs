use thiserror::Error;

pub type Result<T> = std::result::Result<T, SlackError>;

#[derive(Error, Debug)]
pub enum SlackError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("AWS DynamoDB error: {0}")]
    DynamoDb(String),

    #[error("Lambda runtime error: {0}")]
    Lambda(#[from] lambda_runtime::Error),

    #[error("Invalid request signature")]
    InvalidSignature,

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Slack API error: {code} - {message}")]
    SlackApi { code: String, message: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Internal error: {0}")]
    Internal(String),
}