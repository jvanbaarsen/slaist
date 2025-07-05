use chrono::Utc;
use std::env;
use todoist::{TodoistClient, TodoistError};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), TodoistError> {
    println!("🚀 Todoist Client - Continuous Refresh");
    println!("=======================================");

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

    println!("📱 Fetching todos every 10 seconds... (Press Ctrl+C to stop)");
    println!();

    let mut iteration = 1;

    loop {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        println!("🔄 Refresh #{} - {}", iteration, now);
        println!("{:-<60}", "");

        // Fetch and display todos
        match client.get_all_todos().await {
            Ok(todos) => {
                println!("📋 Your Todos ({} total):", todos.len());

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

                        let status_icon = if todo.is_completed { "✅" } else { "📝" };

                        println!(
                            "{} {} {} {}",
                            i + 1,
                            status_icon,
                            priority_icon,
                            todo.content
                        );

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
                let due_today_count = todos
                    .iter()
                    .filter(|t| {
                        if let Some(due) = &t.due {
                            let today = Utc::now().format("%Y-%m-%d").to_string();
                            due.date == today
                        } else {
                            false
                        }
                    })
                    .count();

                println!("\n📊 Statistics:");
                println!("   📝 Total todos: {}", todos.len());
                println!("   ✅ Active todos: {}", active_count);
                println!("   ✔️  Completed todos: {}", completed_count);
                println!("   🔥 High priority: {}", high_priority_count);
                println!("   📅 Due today: {}", due_today_count);
            }
            Err(e) => {
                eprintln!("❌ Error fetching todos: {}", e);
                match e {
                    TodoistError::ApiError { status, .. } if status == 401 => {
                        eprintln!("🔑 Check your API token - it might be invalid or expired");
                    }
                    TodoistError::RequestFailed(_) => {
                        eprintln!("🌐 Network error - check your internet connection");
                    }
                    _ => {}
                }
            }
        }

        // Show high priority todos separately
        match client
            .get_todos_with_filters(
                None,            // project_id
                None,            // section_id
                None,            // label
                Some("p1 | p2"), // filter for high priority
                None,            // lang
                None,            // ids
            )
            .await
        {
            Ok(high_priority_todos) if !high_priority_todos.is_empty() => {
                println!("\n🔥 High Priority Todos ({}):", high_priority_todos.len());
                for todo in high_priority_todos.iter().take(5) {
                    let status_icon = if todo.is_completed { "✅" } else { "📝" };
                    println!("   {} {} (P{})", status_icon, todo.content, todo.priority);
                }
            }
            Ok(_) => {
                println!("\n🎉 No high priority todos!");
            }
            Err(_) => {
                // Don't show error for secondary query
            }
        }

        println!("\n{:-<60}", "");
        println!("⏳ Waiting 10 seconds until next refresh...");
        println!();

        // Wait 10 seconds before next iteration
        sleep(Duration::from_secs(10)).await;
        iteration += 1;
    }
}
