# Todoist Rust Client

A Rust client library for interacting with the Todoist API v1. This crate provides a simple and ergonomic way to fetch todos, projects, and perform other operations with your Todoist account.

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

    // Fetch all todos (no filter)
    let todos = client.get_all_todos(None).await?;
    
    println!("You have {} todos:", todos.len());
    for todo in todos {
        println!("- {}", todo.content);
    }

    // Fetch only today's todos using optional query parameter
    let today_todos = client.get_all_todos(Some("today")).await?;
    
    println!("You have {} todos for today:", today_todos.len());
    for todo in today_todos {
        println!("- {}", todo.content);
    }

    // Fetch todos completed today
    let completed_today = client.get_todos_completed_today().await?;
    
    println!("You completed {} todos today:", completed_today.len());
    for todo in completed_today {
        println!("✅ {}", todo.content);
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

### Getting Completed Todos

```rust
// Get todos that were completed today
let completed_today = client.get_todos_completed_today().await?;

println!("Completed today:");
for todo in completed_today {
    println!("✅ {} (completed at: {})", 
        todo.content, 
        todo.completed_at.as_ref().unwrap_or(&"unknown".to_string())
    );
}

// Get todos completed on a specific date
let completed_yesterday = client.get_todos_completed_on_date("2023-12-24").await?;

println!("Completed yesterday:");
for todo in completed_yesterday {
    println!("✅ {}", todo.content);
}

// Get todos completed within a date range (using RFC3339 format)
let completed_in_range = client.get_todos_completed_by_date_range(
    "2023-12-20T00:00:00Z", 
    "2023-12-25T23:59:59Z"
).await?;

println!("Completed in date range:");
for todo in completed_in_range {
    println!("✅ {}", todo.content);
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
- `checked`: Completion status
- `priority`: Priority level (1-4)
- `project_id`: Associated project
- `labels`: Array of labels
- `due`: Due date information
- `completed_at`: When the todo was completed (if applicable)
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

You can run the included examples to see the crate in action:

```bash
# Set your API token
export TODOIST_API_TOKEN="your_token_here"

# Run the basic example
cargo run --example fetch_todos

# Run the completed todos example
cargo run --example completed_todos
```

## API Reference

### TodoistClient Methods

- `new(token: String)` - Create a new client
- `get_all_todos(query: Option<&str>)` - Fetch all active todos, optionally filtered by query
- `get_todos_completed_today()` - Fetch all todos completed today
- `get_todos_completed_on_date(date: &str)` - Fetch todos completed on a specific date (YYYY-MM-DD format)
- `get_todos_completed_by_date_range(since: &str, until: &str)` - Fetch todos completed within a date range (RFC3339 format)
- `get_todos_with_filters(project_id, section_id, parent_id, label, ids)` - Fetch todos with filters
- `get_todos_by_filter(query, lang)` - Fetch todos using the new filter endpoint with query syntax
- `get_all_projects()` - Fetch all projects
- `get_todo(id: &str)` - Fetch a specific todo
- `complete_todo(id)` - Mark todo as complete  
- `create_todo(...)` - Create a new todo

### Filter Query Examples

Both `get_all_todos(Some(query))` and `get_todos_by_filter` methods support Todoist's powerful filter syntax:

- `"today"` - Tasks due today
- `"overdue"` - Overdue tasks  
- `"p1"` - Priority 1 tasks
- `"@label_name"` - Tasks with specific label
- `"#project_name"` - Tasks in specific project
- `"today | overdue"` - Today's tasks OR overdue tasks
- `"p1 & today"` - Priority 1 AND due today

### Method Usage Examples

```rust
// Fetch all todos
let all_todos = client.get_all_todos(None).await?;

// Fetch today's todos using optional parameter
let today_todos = client.get_all_todos(Some("today")).await?;

// Fetch high priority todos using optional parameter  
let urgent_todos = client.get_all_todos(Some("p1")).await?;

// Or use the dedicated filter method for more control
let filtered_todos = client.get_todos_by_filter("today & p1", Some("en")).await?;

// Get todos completed today
let completed_today = client.get_todos_completed_today().await?;

// Get todos completed on a specific date
let completed_on_date = client.get_todos_completed_on_date("2023-12-25").await?;

// Get todos completed within a date range
let completed_in_range = client.get_todos_completed_by_date_range(
    "2023-12-20T00:00:00Z", 
    "2023-12-25T23:59:59Z"
).await?;
```

## Requirements

- Rust 1.70 or later
- Tokio runtime for async operations
- Valid Todoist API token

## License

This project is licensed under the MIT License.