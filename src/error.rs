use thiserror::Error;

#[derive(Debug, Error)]
pub enum WarpError {
    #[error("API key not configured. Run 'warp config set law.key YOUR_KEY' to configure.")]
    NoApiKey,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error ({code}): {message}")]
    ApiError {
        code: String,
        message: String,
        hint: Option<String>,
    },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    #[allow(dead_code)]
    NotFound(String),

    #[error("Timeout: operation timed out after {0} seconds")]
    #[allow(dead_code)]
    Timeout(u64),

    #[error("Rate limit exceeded. Please try again later.")]
    RateLimit,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Authentication failed: {0}")]
    #[allow(dead_code)]
    AuthenticationFailed(String),

    #[error("{0}")]
    Other(String),
}

impl WarpError {
    /// Create an API error with an optional hint
    #[allow(dead_code)]
    pub fn api_error(code: impl Into<String>, message: impl Into<String>, hint: Option<String>) -> Self {
        Self::ApiError {
            code: code.into(),
            message: message.into(),
            hint,
        }
    }

    /// Get user-friendly hint for the error
    #[allow(dead_code)]
    pub fn hint(&self) -> Option<String> {
        match self {
            Self::NoApiKey => Some(
                "Visit https://open.law.go.kr to get your API key. \
                 Then run: warp config set law.key YOUR_KEY".to_string()
            ),
            Self::ApiError { hint, .. } => hint.clone(),
            Self::Network(_) => Some("Check your internet connection and try again.".to_string()),
            Self::RateLimit => Some("You've made too many requests. Please wait a moment.".to_string()),
            Self::AuthenticationFailed(_) => Some("Check your API key configuration.".to_string()),
            _ => None,
        }
    }

    /// Check if the error is retryable
    #[allow(dead_code)]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout(_) | Self::ServerError(_) | Self::RateLimit
        )
    }
}

pub type Result<T> = std::result::Result<T, WarpError>;