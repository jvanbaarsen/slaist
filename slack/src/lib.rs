use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct SlackClient {
    client: Client,
    token: String,
}

#[derive(Debug, Serialize)]
struct SlackMessage {
    text: String,
    channel: String,
}

#[derive(Debug, Deserialize)]
struct SlackApiResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug)]
pub enum SlackError {
    HttpError(reqwest::Error),
    ApiError(String),
    ConfigError(String),
    SerializationError(serde_json::Error),
}

impl std::fmt::Display for SlackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlackError::HttpError(e) => write!(f, "HTTP error: {}", e),
            SlackError::ApiError(e) => write!(f, "Slack API error: {}", e),
            SlackError::ConfigError(e) => write!(f, "Configuration error: {}", e),
            SlackError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for SlackError {}

impl From<reqwest::Error> for SlackError {
    fn from(err: reqwest::Error) -> Self {
        SlackError::HttpError(err)
    }
}

impl From<serde_json::Error> for SlackError {
    fn from(err: serde_json::Error) -> Self {
        SlackError::SerializationError(err)
    }
}

impl SlackClient {
    /// Create a new Slack client
    ///
    /// Will automatically look for SLACK_BOT_TOKEN environment variable
    pub fn new() -> Result<Self, SlackError> {
        let token = env::var("SLACK_BOT_TOKEN").map_err(|_| {
            SlackError::ConfigError("SLACK_BOT_TOKEN environment variable must be set".to_string())
        })?;

        if token.is_empty() {
            return Err(SlackError::ConfigError(
                "SLACK_BOT_TOKEN cannot be empty".to_string(),
            ));
        }

        Ok(SlackClient {
            client: Client::new(),
            token,
        })
    }

    /// Create a new Slack client with explicit bot token
    pub fn with_bot_token(token: String) -> Result<Self, SlackError> {
        if token.is_empty() {
            return Err(SlackError::ConfigError(
                "Bot token cannot be empty".to_string(),
            ));
        }

        Ok(SlackClient {
            client: Client::new(),
            token,
        })
    }

    /// Post a message to Slack
    ///
    /// # Arguments
    /// * `message` - The message text to send
    /// * `channel` - Channel to send to (e.g., "#general", "@username", or channel ID)
    /// * `username` - Optional username to display as
    /// * `icon_emoji` - Optional emoji icon (e.g., ":robot_face:")
    pub async fn post_message(&self, message: &str, channel: &str) -> Result<(), SlackError> {
        let payload = SlackMessage {
            text: message.to_string(),
            channel: channel.to_string(),
        };

        let response = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        let api_response: SlackApiResponse = response.json().await?;

        if !api_response.ok {
            let error_msg = api_response
                .error
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SlackError::ApiError(error_msg));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_client_creation_fails_without_token() {
        // Clear environment variable for this test
        unsafe {
            env::remove_var("SLACK_BOT_TOKEN");
        }

        let result = SlackClient::new();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::ConfigError(_)));
    }

    #[test]
    fn test_slack_client_with_bot_token() {
        let client = SlackClient::with_bot_token("xoxb-test-token".to_string());
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.token, "xoxb-test-token");
    }

    #[test]
    fn test_slack_client_with_empty_token() {
        let result = SlackClient::with_bot_token("".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::ConfigError(_)));
    }
}
