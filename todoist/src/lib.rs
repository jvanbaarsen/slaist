use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A client for interacting with the Todoist API
pub struct TodoistClient {
    client: reqwest::Client,
    base_url: String,
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
        }
    }

    /// Fetches all active todos from Todoist
    pub async fn get_all_todos(&self) -> Result<Vec<Todo>, TodoistError> {
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

        let todos: Vec<Todo> = response.json().await?;
        Ok(todos)
    }

    /// Fetches todos with optional filters
    pub async fn get_todos_with_filters(
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

    /// Fetches all projects from Todoist
    pub async fn get_all_projects(&self) -> Result<Vec<Project>, TodoistError> {
        let url = format!("{}/projects", self.base_url);

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

        let projects: Vec<Project> = response.json().await?;
        Ok(projects)
    }

    /// Fetches a specific todo by ID
    pub async fn get_todo(&self, id: &str) -> Result<Todo, TodoistError> {
        let url = format!("{}/tasks/{}", self.base_url, id);

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

        let todo: Todo = response.json().await?;
        Ok(todo)
    }

    /// Marks a todo as completed
    pub async fn complete_todo(&self, id: &str) -> Result<(), TodoistError> {
        let url = format!("{}/tasks/{}/close", self.base_url, id);

        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            return Err(TodoistError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            });
        }

        Ok(())
    }

    /// Creates a new todo
    pub async fn create_todo(
        &self,
        content: &str,
        description: Option<&str>,
        project_id: Option<&str>,
        section_id: Option<&str>,
        parent_id: Option<&str>,
        order: Option<u32>,
        labels: Option<Vec<String>>,
        priority: Option<u8>,
        due_string: Option<&str>,
        due_date: Option<&str>,
        due_datetime: Option<&str>,
        due_lang: Option<&str>,
        assignee_id: Option<&str>,
    ) -> Result<Todo, TodoistError> {
        let url = format!("{}/tasks", self.base_url);

        let mut body = HashMap::new();
        body.insert("content", content.to_string());

        if let Some(description) = description {
            body.insert("description", description.to_string());
        }
        if let Some(project_id) = project_id {
            body.insert("project_id", project_id.to_string());
        }
        if let Some(section_id) = section_id {
            body.insert("section_id", section_id.to_string());
        }
        if let Some(parent_id) = parent_id {
            body.insert("parent_id", parent_id.to_string());
        }
        if let Some(order) = order {
            body.insert("order", order.to_string());
        }
        if let Some(labels) = labels {
            body.insert("labels", labels.join(","));
        }
        if let Some(priority) = priority {
            body.insert("priority", priority.to_string());
        }
        if let Some(due_string) = due_string {
            body.insert("due_string", due_string.to_string());
        }
        if let Some(due_date) = due_date {
            body.insert("due_date", due_date.to_string());
        }
        if let Some(due_datetime) = due_datetime {
            body.insert("due_datetime", due_datetime.to_string());
        }
        if let Some(due_lang) = due_lang {
            body.insert("due_lang", due_lang.to_string());
        }
        if let Some(assignee_id) = assignee_id {
            body.insert("assignee_id", assignee_id.to_string());
        }

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(TodoistError::ApiError {
                status: response.status().as_u16(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            });
        }

        let todo: Todo = response.json().await?;
        Ok(todo)
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
        let _client = TodoistClient::new(token);
        // Just verify that the client was created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_get_all_todos_with_invalid_token() {
        let client = TodoistClient::new("invalid_token".to_string());
        let result = client.get_all_todos().await;
        assert!(result.is_err());
    }
}
