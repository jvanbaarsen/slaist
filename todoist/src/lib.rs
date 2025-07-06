use chrono::Utc;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

/// A client for interacting with the Todoist API
pub struct TodoistClient {
    client: reqwest::Client,
    base_url: String,
    query: Option<String>,
}

/// Represents a Todoist task/todo item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: String,
    pub user_id: String,
    pub project_id: String,
    pub section_id: Option<String>,
    pub parent_id: Option<String>,
    pub content: String,
    pub description: Option<String>,
    pub priority: u8,
    pub labels: Vec<String>,
    pub due: Option<TodoDue>,
    pub deadline: Option<serde_json::Value>,
    pub duration: Option<serde_json::Value>,
    pub checked: bool,
    pub is_deleted: bool,
    pub added_at: String,
    pub completed_at: Option<String>,
    pub updated_at: String,
    pub child_order: u32,
    pub day_order: Option<i32>,
    pub is_collapsed: Option<bool>,
    pub added_by_uid: Option<String>,
    pub assigned_by_uid: Option<String>,
    pub responsible_uid: Option<String>,
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

/// Represents a Todoist project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: String,
    pub parent_id: Option<String>,
    pub order: u32,
    pub comment_count: u32,
    pub is_shared: bool,
    pub is_favorite: bool,
    pub is_inbox_project: bool,
    pub is_team_inbox: bool,
    pub view_style: String,
    pub url: String,
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
    pub fn new(token: String, query: Option<String>) -> Self {
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
            base_url: "https://api.todoist.com/api/v1".to_string(),
            query: Some(query.unwrap_or_else(|| "(overdue | today) & #Work".to_string())),
        }
    }

    /// Fetches all active todos from Todoist, optionally filtered by query
    pub async fn get_all_todos(&self) -> Result<Vec<Todo>, TodoistError> {
        // If query is provided, use the filter endpoint
        if let Some(q) = &self.query {
            return self.get_todos_by_filter(&q, None).await;
        }

        // Otherwise use the standard tasks endpoint
        let url = format!("{}/tasks", self.base_url);

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

        #[derive(serde::Deserialize)]
        struct TodosResponse {
            results: Vec<Todo>,
        }

        let response_data: TodosResponse = response.json().await?;
        Ok(response_data.results)
    }

    /// Fetches todos using the new filter endpoint
    async fn get_todos_by_filter(
        &self,
        query: &str,
        lang: Option<&str>,
    ) -> Result<Vec<Todo>, TodoistError> {
        let mut url = format!("{}/tasks/filter", self.base_url);
        let mut params = vec![format!("query={}", urlencoding::encode(query))];

        if let Some(lang) = lang {
            params.push(format!("lang={}", lang));
        }

        url.push('?');
        url.push_str(&params.join("&"));

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

        #[derive(serde::Deserialize)]
        struct TodosResponse {
            results: Vec<Todo>,
        }

        let response_data: TodosResponse = response.json().await?;
        Ok(response_data.results)
    }

    /// Fetches all todos that were completed today
    pub async fn get_todos_completed_today(&self) -> Result<Vec<Todo>, TodoistError> {
        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

        let since = today_start.and_utc().to_rfc3339();
        let until = today_end.and_utc().to_rfc3339();

        self.get_todos_completed_by_date_range(&since, &until).await
    }

    /// Fetches all todos that were completed on a specific date
    ///
    /// # Arguments
    /// * `date` - The date in YYYY-MM-DD format (e.g., "2023-12-25")
    pub async fn get_todos_completed_on_date(&self, date: &str) -> Result<Vec<Todo>, TodoistError> {
        // Parse the date and create start/end of day timestamps
        let date_start = format!("{}T00:00:00Z", date);
        let date_end = format!("{}T23:59:59Z", date);

        self.get_todos_completed_by_date_range(&date_start, &date_end)
            .await
    }

    /// Fetches todos completed within a specific date range
    ///
    /// # Arguments
    /// * `since` - Start of date range in RFC3339 format
    /// * `until` - End of date range in RFC3339 format
    pub async fn get_todos_completed_by_date_range(
        &self,
        since: &str,
        until: &str,
    ) -> Result<Vec<Todo>, TodoistError> {
        // Use the correct endpoint for completed tasks by completion date
        let url = format!("{}/tasks/completed/by_completion_date", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("since", since),
                ("until", until),
                ("filter_query", self.query.as_ref().unwrap().as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TodoistError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            });
        }

        #[derive(serde::Deserialize)]
        struct CompletedTasksResponse {
            items: Vec<Todo>,
        }

        let response_data: CompletedTasksResponse = response.json().await?;
        Ok(response_data.items)
    }
}

// Helper function for creating a client - useful for testing
pub fn create_client(token: String) -> TodoistClient {
    TodoistClient::new(token, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let token = "test_token".to_string();
        let _client = TodoistClient::new(token, None);
        // Just verify that the client was created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_get_all_todos_with_invalid_token() {
        let client = TodoistClient::new("invalid_token".to_string(), None);
        let result = client.get_all_todos(None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_todos_completed_today_with_invalid_token() {
        let client = TodoistClient::new("invalid_token".to_string(), None);
        let result = client.get_todos_completed_today().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_todos_completed_on_date_with_invalid_token() {
        let client = TodoistClient::new("invalid_token".to_string(), None);
        let result = client.get_todos_completed_on_date("2023-12-25").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_todos_completed_by_date_range_with_invalid_token() {
        let client = TodoistClient::new("invalid_token".to_string(), None);
        let result = client
            .get_todos_completed_by_date_range("2023-12-25T00:00:00Z", "2023-12-25T23:59:59Z")
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_date_range_formatting() {
        // Test that we can format date ranges correctly for the API
        let now = chrono::Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

        let since = today_start.and_utc().to_rfc3339();
        let until = today_end.and_utc().to_rfc3339();

        // Should be in RFC3339 format
        assert!(since.contains("T00:00:00"));
        assert!(until.contains("T23:59:59"));
        assert!(since.contains("Z") || since.contains("+"));
        assert!(until.contains("Z") || until.contains("+"));
    }
}
