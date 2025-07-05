use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

/// A client for interacting with the Todoist API
pub struct TodoistClient {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

/// Represents a Todoist task/todo item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: String,
    pub content: String,
    pub description: Option<String>,
    pub is_completed: bool,
    pub priority: u8,
    pub project_id: String,
    pub section_id: Option<String>,
    pub parent_id: Option<String>,
    pub order: u32,
    pub labels: Vec<String>,
    pub url: String,
    pub comment_count: u32,
    pub created_at: String,
    pub creator_id: String,
    pub assignee_id: Option<String>,
    pub assigner_id: Option<String>,
    pub due: Option<TodoDue>,
}

/// Represents due date information for a todo
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoDue {
    pub date: String,
    pub is_recurring: bool,
    pub datetime: Option<String>,
    pub string: String,
    pub timezone: Option<String>,
}

/// Error types for Todoist operations
#[derive(Debug, thiserror::Error)]
pub enum TodoistError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("Authentication failed")]
    AuthenticationError,
}

impl TodoistClient {
    /// Creates a new Todoist client with the provided API token
    pub fn new(token: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://api.todoist.com/rest/v2".to_string(),
            token,
        }
    }

    pub async fn get_all(
        &self,
        project_id: Option<&str>,
        section_id: Option<&str>,
        label: Option<&str>,
        filter: Option<&str>,
        lang: Option<&str>,
        ids: Option<Vec<String>>,
    ) -> Result<Vec<Todo>, TodoistError> {
        let mut url = format!("{}/tasks", self.base_url);
        let mut params = Vec::new();

        println!(
            "Debug: TodoistClient created with base_url: {}, token: {}",
            self.base_url,
            &self.token[..std::cmp::min(8, self.token.len())]
        );

        if let Some(project_id) = project_id {
            params.push(format!("project_id={}", project_id));
        }
        if let Some(section_id) = section_id {
            params.push(format!("section_id={}", section_id));
        }
        if let Some(label) = label {
            params.push(format!("label={}", urlencoding::encode(label)));
        }
        if let Some(filter) = filter {
            params.push(format!("filter={}", urlencoding::encode(filter)));
        }
        if let Some(lang) = lang {
            params.push(format!("lang={}", lang));
        }
        if let Some(ids) = ids {
            params.push(format!("ids={}", ids.join(",")));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(TodoistError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            });
        }

        let todos: Vec<Todo> = response.json().await?;
        Ok(todos)
    }
}

// Helper function for creating a client - useful for testing
pub fn create_client(token: String) -> TodoistClient {
    TodoistClient::new(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let token = "test_token".to_string();
        let client = TodoistClient::new(token.clone());
        assert_eq!(client.token, token);
        assert_eq!(client.base_url, "https://api.todoist.com/rest/v2");
    }

    #[tokio::test]
    async fn test_get_all_todos_with_invalid_token() {
        let client = TodoistClient::new("invalid_token".to_string());
        let result = client.get_all(None, None, None, None, None, None).await;
        assert!(result.is_err());
    }
}
