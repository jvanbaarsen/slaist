use std::env;
use todoist::{TodoistClient, TodoistError};

#[tokio::test]
async fn test_client_creation() {
    let token = "test_token".to_string();
    let _client = TodoistClient::new(token);

    // Test that the client is created successfully
    // We can't test actual API calls without a valid token
    // Just verify that the client was created without panicking
    assert!(true); // Client creation succeeded
}

#[tokio::test]
async fn test_get_all_todos_with_invalid_token() {
    let client = TodoistClient::new("invalid_token".to_string());

    match client.get_all_todos().await {
        Err(TodoistError::ApiError { status, .. }) => {
            // Should get a 401 Unauthorized or similar error
            assert!(status == 401 || status == 403);
        }
        Err(_) => {
            // Any error is acceptable for invalid token
        }
        Ok(_) => {
            panic!("Expected error for invalid token, but got success");
        }
    }
}

// Integration test that requires a valid API token
// This test will only run if TODOIST_API_TOKEN is set
#[tokio::test]
async fn test_real_api_calls() {
    // Skip this test if no API token is provided
    let token = match env::var("TODOIST_API_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            println!("Skipping real API test - TODOIST_API_TOKEN not set");
            return;
        }
    };

    if token.is_empty() || token == "your_api_token_here" {
        println!("Skipping real API test - invalid token");
        return;
    }

    let client = TodoistClient::new(token);

    // Test fetching all todos
    match client.get_all_todos().await {
        Ok(todos) => {
            println!("Successfully fetched {} todos", todos.len());

            // Verify the structure of todos
            for todo in todos.iter().take(5) {
                assert!(!todo.id.is_empty());
                assert!(!todo.content.is_empty());
                assert!(!todo.project_id.is_empty());
                assert!(!todo.url.is_empty());
            }
        }
        Err(e) => {
            panic!("Failed to fetch todos: {}", e);
        }
    }

    // Test fetching todos with filters
    match client
        .get_todos_with_filters(None, None, None, None, None, None)
        .await
    {
        Ok(todos) => {
            println!("Successfully fetched {} todos with filters", todos.len());
        }
        Err(e) => {
            panic!("Failed to fetch todos with filters: {}", e);
        }
    }
}
