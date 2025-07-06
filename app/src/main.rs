use chrono::Utc;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
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
    let client = TodoistClient::new(token, Some("(overdue | today) & #Work".to_string()));

    println!("📱 Fetching todos every 10 seconds... (Press Ctrl+C to stop)");
    println!();

    let mut iteration = 1;

    // Create the ~/slaist directory if it doesn't exist
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let slaist_dir = Path::new(&home_dir).join("slaist");
    if let Err(e) = fs::create_dir_all(&slaist_dir) {
        eprintln!(
            "⚠️  Warning: Could not create directory {}: {}",
            slaist_dir.display(),
            e
        );
    }

    loop {
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC");
        let date_str = now.format("%Y-%m-%d");
        let filename = format!("{}.md", date_str);
        let file_path = slaist_dir.join(&filename);

        println!("🔄 Refresh #{} - {}", iteration, timestamp);
        println!("{:-<60}", "");

        let mut markdown_content = String::new();

        // Fetch and display active todos
        println!("📋 Active Todos:");
        markdown_content.push_str("## Active Todos\n\n");

        match client.get_all_todos().await {
            Ok(todos) => {
                if todos.is_empty() {
                    println!("   No active todos found! 🎉");
                    markdown_content.push_str("*No active todos found! 🎉*\n\n");
                } else {
                    // Show active todos
                    for (i, todo) in todos.iter().enumerate() {
                        let status_icon = if todo.checked { "[x]" } else { "[ ]" };

                        println!("{} {} {}", i + 1, status_icon, todo.content);

                        // Add to markdown with checkbox format
                        let checkbox = if todo.checked { "- [x]" } else { "- [ ]" };
                        markdown_content.push_str(&format!("{} {}\n", checkbox, todo.content));
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error fetching active todos: {}", e);
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

        // Fetch and display completed todos from today
        println!("\n✅ Completed Today:");
        markdown_content.push_str("\n## Completed Today\n\n");

        match client.get_todos_completed_today().await {
            Ok(completed_todos) => {
                if completed_todos.is_empty() {
                    println!("   No todos completed today yet.");
                    markdown_content.push_str("*No todos completed today yet.*\n\n");
                } else {
                    for (i, todo) in completed_todos.iter().enumerate() {
                        println!("{} [x] {}", i + 1, todo.content);
                        markdown_content.push_str(&format!("- [x] {}\n", todo.content));
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error fetching completed todos: {}", e);
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

        match fs::File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(markdown_content.as_bytes()) {
                    eprintln!(
                        "⚠️  Warning: Could not write to file {}: {}",
                        file_path.display(),
                        e
                    );
                } else {
                    println!("💾 Saved to: {}", file_path.display());
                }
            }
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Could not create file {}: {}",
                    file_path.display(),
                    e
                );
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
