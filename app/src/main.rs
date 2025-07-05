use std::env;
use todoist::{TodoistClient, TodoistError};

#[tokio::main]
async fn main() -> Result<(), TodoistError> {
    println!("🚀 Todoist Client Demo");
    println!("=====================");

    // Try to get API token from environment
    let token = match env::var("TODOIST_API_TOKEN") {
        Ok(token) if !token.is_empty() && token != "your_api_token_here" => token,
        _ => {
            println!("⚠️  TODOIST_API_TOKEN environment variable not set or invalid");
            println!("   Please set your Todoist API token:");
            println!("   export TODOIST_API_TOKEN=\"your_actual_token_here\"");
            println!("   Get your token from: https://todoist.com/prefs/integrations");
            return Ok(());
        }
    };

    // Create Todoist client
    let client = TodoistClient::new(token);

    let filter = "(today | overdue) & #Work";

    // Fetch and display todos
    match client
        .get_all(None, None, None, Some(filter), None, None)
        .await
    {
        Ok(todos) => {
            println!("\n📋 Your Todos ({} total):", todos.len());
            println!("{:-<60}", "");

            if todos.is_empty() {
                println!("   No todos found! 🎉");
            } else {
                // Show first 10 todos
                for (i, todo) in todos.iter().take(10).enumerate() {
                    let priority_icon = match todo.priority {
                        4 => "🔴",
                        3 => "🟠",
                        2 => "🟡",
                        _ => "⚪",
                    };

                    println!("{} {} {}", i + 1, priority_icon, todo.content);

                    if let Some(due) = &todo.due {
                        println!("     📅 Due: {}", due.string);
                    }

                    if !todo.labels.is_empty() {
                        println!("     🏷️  Labels: {}", todo.labels.join(", "));
                    }
                }

                if todos.len() > 10 {
                    println!("   ... and {} more todos", todos.len() - 10);
                }
            }

            // Show some statistics
            let completed_count = todos.iter().filter(|t| t.is_completed).count();
            let active_count = todos.len() - completed_count;
            let high_priority_count = todos.iter().filter(|t| t.priority >= 3).count();

            println!("\n📊 Statistics:");
            println!("   📝 Total todos: {}", todos.len());
            println!("   ✅ Active todos: {}", active_count);
            println!("   ✔️  Completed todos: {}", completed_count);
            println!("   🔥 High priority: {}", high_priority_count);
        }
        Err(e) => {
            eprintln!("❌ Error fetching todos: {}", e);
            return Err(e);
        }
    }

    println!("\n✨ Demo completed successfully!");
    println!("   Run `cargo run --example fetch_todos` for a more detailed example");

    Ok(())
}
