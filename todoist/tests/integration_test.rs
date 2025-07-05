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

#[tokio::test]
async fn test_get_all_projects_with_invalid_token() {
    let client = TodoistClient::new("invalid_token".to_string());

    match client.get_all_projects().await {
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

#[tokio::test]
async fn test_get_todo_with_invalid_token() {
    let client = TodoistClient::new("invalid_token".to_string());

    match client.get_todo("123456789").await {
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

    // Test fetching all projects
    match client.get_all_projects().await {
        Ok(projects) => {
            println!("Successfully fetched {} projects", projects.len());

            // Verify the structure of projects
            for project in projects.iter().take(3) {
                assert!(!project.id.is_empty());
                assert!(!project.name.is_empty());
                assert!(!project.url.is_empty());
            }
        }
        Err(e) => {
            panic!("Failed to fetch projects: {}", e);
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

#[test]
fn test_todo_struct_serialization() {
    use serde_json;
    use todoist::Todo;

    let json_str = r#"
    {
        "id": "123456789",
        "content": "Test todo",
        "description": "Test description",
        "is_completed": false,
        "priority": 1,
        "project_id": "987654321",
        "section_id": null,
        "parent_id": null,
        "order": 1,
        "labels": ["@test"],
        "url": "https://todoist.com/app/task/123456789",
        "comment_count": 0,
        "created_at": "2023-01-01T00:00:00Z",
        "creator_id": "user123",
        "assignee_id": null,
        "assigner_id": null,
        "due": null
    }
    "#;

    let todo: Result<Todo, _> = serde_json::from_str(json_str);
    assert!(todo.is_ok());

    let todo = todo.unwrap();
    assert_eq!(todo.id, "123456789");
    assert_eq!(todo.content, "Test todo");
    assert_eq!(todo.description, Some("Test description".to_string()));
    assert!(!todo.is_completed);
    assert_eq!(todo.priority, 1);
    assert_eq!(todo.labels, vec!["@test"]);
}

#[test]
fn test_project_struct_serialization() {
    use serde_json;
    use todoist::Project;

    let json_str = r#"
    {
        "id": "987654321",
        "name": "Test Project",
        "color": "red",
        "parent_id": null,
        "order": 1,
        "comment_count": 0,
        "is_shared": false,
        "is_favorite": false,
        "is_inbox_project": false,
        "is_team_inbox": false,
        "view_style": "list",
        "url": "https://todoist.com/app/project/987654321"
    }
    "#;

    let project: Result<Project, _> = serde_json::from_str(json_str);
    assert!(project.is_ok());

    let project = project.unwrap();
    assert_eq!(project.id, "987654321");
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.color, "red");
    assert!(!project.is_shared);
    assert!(!project.is_favorite);
}
