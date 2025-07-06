use std::env;
use todoist::{TodoistClient, TodoistError};

#[tokio::main]
async fn main() -> Result<(), TodoistError> {
    // Create client with your API token
    let token =
        env::var("TODOIST_API_TOKEN").expect("Please set TODOIST_API_TOKEN environment variable");
    let client = TodoistClient::new(token);

    println!("🔄 Fetching all todos...");

    // Fetch all todos (no filter)
    let todos = client.get_all_todos(None).await?;

    println!("📋 You have {} todos:", todos.len());

    if todos.is_empty() {
        println!("   No todos found! 🎉");
        return Ok(());
    }

    // Display todos with details
    for (i, todo) in todos.iter().enumerate() {
        let status = if todo.checked { "✅" } else { "⭕" };
        let priority_text = match todo.priority {
            4 => " 🔴 P1",
            3 => " 🟡 P2",
            2 => " 🔵 P3",
            _ => "",
        };

        println!("{}. {} {}{}", i + 1, status, todo.content, priority_text);

        if !todo.labels.is_empty() {
            println!("     Labels: {}", todo.labels.join(", "));
        }

        if let Some(due) = &todo.due {
            println!("     Due: {}", due.string);
        }

        if let Some(description) = &todo.description {
            if !description.is_empty() {
                println!("     Description: {}", description);
            }
        }

        println!();
    }

    // Example: Fetch todos with query using get_all_todos with optional parameter
    println!("🔍 Fetching today's todos using get_all_todos...");

    let today_todos = client.get_all_todos(Some("today")).await?;
    println!("📅 Found {} todos for today:", today_todos.len());
    for todo in today_todos.iter().take(3) {
        let status = if todo.checked { "✅" } else { "⭕" };
        println!("   {} {}", status, todo.content);
    }
    println!();

    // Example: Fetch todos with filters
    println!("🔍 Fetching todos from a specific project...");

    // You can get your project IDs by first fetching projects
    let projects = client.get_all_projects().await?;

    if let Some(project) = projects.first() {
        println!("   Using project: {}", project.name);

        let filtered_todos = client
            .get_todos_with_filters(
                Some(&project.id), // project_id
                None,              // section_id
                None,              // parent_id
                None,              // label
                None,              // ids
            )
            .await?;

        println!("   Found {} todos in this project", filtered_todos.len());
    }

    // Example: Use the new filter endpoint
    println!("🔍 Using filter query...");

    let filtered_todos = client
        .get_todos_by_filter(
            "today | overdue", // Query for today's and overdue tasks
            Some("en"),        // Language
        )
        .await?;

    println!(
        "📅 Found {} todos for today or overdue:",
        filtered_todos.len()
    );
    for todo in filtered_todos {
        let status = if todo.checked { "✅" } else { "⭕" };
        println!("   {} {}", status, todo.content);
    }

    Ok(())
}
