# Todoist Rust Client

A Rust client library for interacting with the Todoist API. This crate provides a simple and ergonomic way to fetch todos, projects, and perform other operations with your Todoist account.

## Features

- 🚀 Async/await support with tokio
- 📋 Fetch all todos or filter by project, section, label, etc.
- 📁 Retrieve all projects
- ✅ Mark todos as completed
- 📝 Create new todos
- 🎯 Type-safe API with proper error handling
- 🔧 Easy to use and integrate

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
todoist = { path = "../todoist" }
tokio = { version = "1.0", features = ["full"] }
```

## Setup

1. Get your Todoist API token from [Todoist Integrations](https://todoist.com/prefs/integrations)
2. Set it as an environment variable:
   ```bash
   export TODOIST_API_TOKEN="your_api_token_here"
   ```

## Usage

### Basic Example

```rust
use todoist::{TodoistClient, TodoistError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), TodoistError> {
    // Create client with your API token
    let token = env::var("TODOIST_API_TOKEN")?;
    let client = TodoistClient::new(token);

    // Fetch all todos
    let todos = client.get_all_todos().await?;
    
    println!("You have {} todos:", todos.len());
    for todo in todos {
        println!("- {}", todo.content);
        if let Some(due) = todo.due {
            println!("  Due: {}", due.string);
        }
    }

    Ok(())
}
```

### Filtering Todos

```rust
// Get todos from a specific project
let todos = client.get_todos_with_filters(
    Some("project_id_here"),
    None,  // section_id
    None,  // label
    None,  // filter
    None,  // lang
    None,  // ids
).await?;

// Get high priority todos
let high_priority = client.get_todos_with_filters(
    None,
    None,
    None,
    Some("p1 | p2"),  // Priority 1 or 2
    None,
    None,
).await?;

// Get todos with specific label
let labeled_todos = client.get_todos_with_filters(
    None,
    None,
    Some("@work"),
    None,
    None,
    None,
).await?;
```

### Working with Projects

```rust
// Get all projects
let projects = client.get_all_projects().await?;

for project in projects {
    println!("📁 {} ({})", project.name, project.id);
    
    // Get todos for this project
    let project_todos = client.get_todos_with_filters(
        Some(&project.id),
        None, None, None, None, None
    ).await?;
    
    println!("  {} todos", project_todos.len());
}
```

### Creating and Completing Todos

```rust
// Create a new todo
let new_todo = client.create_todo(
    "Buy groceries",                    // content
    Some("Milk, bread, eggs"),         // description
    Some("project_id"),                // project_id
    None,                              // section_id
    None,                              // parent_id
    None,                              // order
    Some(vec!["@errands".to_string()]), // labels
    Some(2),                           // priority
    Some("tomorrow"),                  // due_string
    None, None, None, None,            // other due options
).await?;

println!("Created todo: {}", new_todo.content);

// Mark a todo as completed
client.complete_todo(&new_todo.id).await?;
println!("Todo completed!");
```

## Data Structures

### Todo

The main `Todo` struct contains:

- `id`: Unique identifier
- `content`: The todo text
- `description`: Optional description
- `is_completed`: Completion status
- `priority`: Priority level (1-4)
- `project_id`: Associated project
- `labels`: Array of labels
- `due`: Due date information
- `url`: Todoist URL for the todo
- And more...

### Project

The `Project` struct contains:

- `id`: Unique identifier
- `name`: Project name
- `color`: Project color
- `is_shared`: Whether the project is shared
- `is_favorite`: Whether the project is favorited
- And more...

## Error Handling

The crate provides comprehensive error handling through the `TodoistError` enum:

```rust
match client.get_all_todos().await {
    Ok(todos) => {
        // Handle success
    }
    Err(TodoistError::AuthenticationError) => {
        eprintln!("Invalid API token");
    }
    Err(TodoistError::ApiError { status, message }) => {
        eprintln!("API error {}: {}", status, message);
    }
    Err(TodoistError::RequestFailed(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Running Examples

You can run the included example to see the crate in action:

```bash
# Set your API token
export TODOIST_API_TOKEN="your_token_here"

# Run the example
cargo run --example fetch_todos
```

## API Reference

### TodoistClient Methods

- `new(token: String)` - Create a new client
- `get_all_todos()` - Fetch all active todos
- `get_todos_with_filters(...)` - Fetch todos with filters
- `get_all_projects()` - Fetch all projects
- `get_todo(id: &str)` - Fetch a specific todo
- `complete_todo(id: &str)` - Mark a todo as completed
- `create_todo(...)` - Create a new todo

## Requirements

- Rust 1.70 or later
- Tokio runtime for async operations
- Valid Todoist API token

## License

This project is licensed under the MIT License.