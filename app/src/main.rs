use chrono::Utc;
use serde::{Deserialize, Serialize};
use slack::SlackClient;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use todoist::{Todo, TodoistClient, TodoistError};
use tokio::time::{Duration, sleep};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    todoist_api_token: String,
    slack_bot_token: String,
    slack_channel: Option<String>,
    filter: Option<String>,
    todos_directory: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_todos_dir = Path::new(&home_dir)
            .join("slaist")
            .to_string_lossy()
            .to_string();

        Self {
            todoist_api_token: String::new(),
            slack_bot_token: String::new(),
            slack_channel: Some("#general".to_string()),
            filter: Some("(overdue | today) & #Work".to_string()),
            todos_directory: Some(default_todos_dir),
        }
    }
}

impl Config {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = Path::new(&home_dir).join("slaist").join("config.toml");

        if !config_path.exists() {
            // Create default config file
            let default_config = Self::default();
            let slaist_dir = Path::new(&home_dir).join("slaist");
            fs::create_dir_all(&slaist_dir)?;

            let toml_content = toml::to_string_pretty(&default_config)?;
            fs::write(&config_path, toml_content)?;

            println!(
                "📝 Created default config file at: {}",
                config_path.display()
            );
            println!("⚠️  Please edit the config file and add your API tokens:");
            println!("   - todoist_api_token: Get from https://todoist.com/prefs/integrations");
            println!("   - slack_bot_token: Get from your Slack app settings");

            return Err(
                "Config file created with default values. Please update with your tokens.".into(),
            );
        }

        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_content)?;

        // Validate required fields
        if config.todoist_api_token.is_empty() {
            return Err("todoist_api_token is required in config.toml".into());
        }
        if config.slack_bot_token.is_empty() {
            return Err("slack_bot_token is required in config.toml".into());
        }

        Ok(config)
    }
}

fn expand_tilde_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        if path == "~" {
            PathBuf::from(home_dir)
        } else if path.starts_with("~/") {
            Path::new(&home_dir).join(&path[2..])
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    }
}

fn get_todos_directory(config: &Config) -> PathBuf {
    match &config.todos_directory {
        Some(dir) => expand_tilde_path(dir),
        None => {
            let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            Path::new(&home_dir).join("slaist")
        }
    }
}

/// Parse existing markdown file to extract todo items
fn parse_existing_markdown(content: &str) -> (Vec<(String, bool)>, Option<String>) {
    let mut todos = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut notes_section: Option<String> = None;
    let mut in_notes = false;
    let mut notes_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Check for notes delimiter
        if trimmed == "---" {
            in_notes = true;
            continue;
        }

        // If we're in the notes section, collect all lines
        if in_notes {
            notes_lines.push(line.to_string());
            continue;
        }

        // Handle standard markdown checkbox format
        if trimmed.starts_with("- [ ]") {
            let todo_content = trimmed[5..].trim().to_string();
            todos.push((todo_content, false));
        } else if trimmed.starts_with("- [x]") {
            let mut todo_content = trimmed[5..].trim().to_string();
            // Remove the "*(marked as finished)*" suffix if present
            if todo_content.ends_with("*(marked as finished)*") {
                todo_content = todo_content
                    .trim_end_matches("*(marked as finished)*")
                    .trim()
                    .to_string();
            }
            todos.push((todo_content, true));
        }
        // Handle legacy emoji format for backward compatibility
        else if trimmed.starts_with(":todo:") {
            let todo_content = trimmed[6..].trim().to_string();
            todos.push((todo_content, false));
        } else if trimmed.starts_with(":todo_done:") {
            let mut todo_content = trimmed[11..].trim().to_string();
            // Remove the "*(marked as finished)*" suffix if present
            if todo_content.ends_with("*(marked as finished)*") {
                todo_content = todo_content
                    .trim_end_matches("*(marked as finished)*")
                    .trim()
                    .to_string();
            }
            todos.push((todo_content, true));
        }
    }

    // If we collected notes, join them back together
    if !notes_lines.is_empty() {
        notes_section = Some(notes_lines.join("\n"));
    }

    (todos, notes_section)
}

/// Generate markdown content with comparison logic
fn generate_markdown_content(
    current_todos: &[Todo],
    existing_todos: &[(String, bool)],
    existing_message_id: Option<&str>,
    preserved_notes: Option<&str>,
) -> String {
    let mut content = String::new();

    // Preserve existing message ID if present
    if let Some(message_id) = existing_message_id {
        content.push_str(&format!("<!-- slack_message_id: {} -->\n", message_id));
    }

    // Create a set of current todo contents for fast lookup
    let current_todo_contents: HashSet<String> = current_todos
        .iter()
        .map(|todo| todo.content.clone())
        .collect();

    // Active todos section
    content.push_str("## Active Todos\n\n");

    let active_todos: Vec<_> = current_todos.iter().filter(|todo| !todo.checked).collect();
    if active_todos.is_empty() {
        content.push_str("_No active todos found! 🎉_\n\n");
    } else {
        for todo in active_todos {
            content.push_str(&format!("- [ ] {}\n", todo.content));
        }
    }

    // Completed todos section
    content.push_str("\n## Completed Todos\n\n");

    let completed_todos: Vec<_> = current_todos.iter().filter(|todo| todo.checked).collect();
    let mut has_completed = false;

    // Create a set to track completed todos we've already added
    let mut added_completed: HashSet<String> = HashSet::new();

    // Add currently completed todos
    for todo in completed_todos {
        content.push_str(&format!("- [x] {}\n", todo.content));
        added_completed.insert(todo.content.clone());
        has_completed = true;
    }

    // Add todos that were in markdown but no longer in current todos (mark as finished)
    for (existing_content, was_completed) in existing_todos {
        let in_current = current_todo_contents.contains(existing_content);
        let already_added = added_completed.contains(existing_content);

        if *was_completed && !already_added {
            // Preserve previously completed todos (including those marked as finished)
            content.push_str(&format!("- [x] {}\n", existing_content));
            added_completed.insert(existing_content.clone());
            has_completed = true;
        } else if !*was_completed && !in_current && !already_added {
            // Mark new missing todos as finished
            content.push_str(&format!(
                "- [x] {} *(marked as finished)*\n",
                existing_content
            ));
            has_completed = true;
        }
    }

    if !has_completed {
        content.push_str("_No completed todos yet._\n\n");
    }

    // Add preserved notes section if it exists
    if let Some(notes) = preserved_notes {
        content.push_str("---\n");
        content.push_str(notes);
        // Ensure we end with a newline
        if !notes.ends_with('\n') {
            content.push('\n');
        }
    }

    content
}

/// Extract Slack message ID from markdown content
fn extract_slack_message_id(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("<!-- slack_message_id: ") && line.ends_with(" -->") {
            let id = line
                .trim_start_matches("<!-- slack_message_id: ")
                .trim_end_matches(" -->")
                .to_string();
            return Some(id);
        }
    }
    None
}

/// Add or update Slack message ID in markdown content
fn add_slack_message_id(content: &str, message_id: &str) -> String {
    let metadata_line = format!("<!-- slack_message_id: {} -->", message_id);

    // Check if there's already a message ID
    let mut lines: Vec<&str> = content.lines().collect();

    // Find and replace existing message ID line
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("<!-- slack_message_id: ") && line.ends_with(" -->") {
            lines[i] = &metadata_line;
            return lines.join("\n");
        }
    }

    // If no existing message ID found, add it at the beginning
    format!("{}\n{}", metadata_line, content)
}

/// Filter out Slack message ID metadata from markdown content
fn filter_slack_metadata(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.starts_with("<!-- slack_message_id: ") || !line.ends_with(" -->"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validate that a message ID looks like a valid Slack timestamp
fn validate_message_id(message_id: &str) -> bool {
    // Slack message timestamps are in the format "1234567890.123456"
    // They should contain exactly one dot and be numeric
    let parts: Vec<&str> = message_id.split('.').collect();
    if parts.len() != 2 {
        return false;
    }

    // Check that both parts are numeric
    parts[0].parse::<u64>().is_ok() && parts[1].parse::<u64>().is_ok()
}

#[tokio::main]
async fn main() -> Result<(), TodoistError> {
    println!("🚀 Todoist Client - Continuous Refresh");
    println!("=======================================");

    // Load configuration from TOML file
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
            return Ok(());
        }
    };

    // Create Todoist client
    let client = TodoistClient::new(config.todoist_api_token.clone(), config.filter.clone());

    println!("📱 Fetching todos every 10 seconds... (Press Ctrl+C to stop)");
    println!();

    let mut iteration = 1;

    // Create the todos directory if it doesn't exist
    let slaist_dir = get_todos_directory(&config);
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

        // Read existing markdown file if it exists
        let (existing_todos, existing_message_id, preserved_notes) = if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    let (todos, notes) = parse_existing_markdown(&content);
                    let message_id = extract_slack_message_id(&content);
                    (todos, message_id, notes)
                }
                Err(_) => (Vec::new(), None, None),
            }
        } else {
            (Vec::new(), None, None)
        };

        // Fetch all current todos (active and completed from recent days)
        let all_current_todos = match client.get_all_todos().await {
            Ok(todos) => {
                println!("📋 Fetched {} todos total", todos.len());
                todos
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
                Vec::new()
            }
        };

        println!("existing_todos: {:?}", existing_todos);

        // Check which todos are missing for summary
        let missing_count = existing_todos
            .iter()
            .filter(|(content, was_completed)| {
                !was_completed && !all_current_todos.iter().any(|t| t.content == *content)
            })
            .count();

        // Count previously completed todos that are being preserved
        let preserved_count = existing_todos
            .iter()
            .filter(|(content, was_completed)| {
                *was_completed && !all_current_todos.iter().any(|t| t.content == *content)
            })
            .count();

        // Generate markdown content with comparison logic
        let markdown_content = generate_markdown_content(
            &all_current_todos,
            &existing_todos,
            existing_message_id.as_deref(),
            preserved_notes.as_deref(),
        );

        // Display summary
        let active_count = all_current_todos.iter().filter(|t| !t.checked).count();
        let completed_count = all_current_todos.iter().filter(|t| t.checked).count();

        println!("📊 Summary:");
        println!("   Active: {}", active_count);
        println!("   Completed: {}", completed_count);
        if missing_count > 0 {
            println!("   Marked as finished: {}", missing_count);
        }
        if preserved_count > 0 {
            println!("   Preserved from previous: {}", preserved_count);
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

        let _ = post_slack(&config).await;

        println!("\n{:-<60}", "");
        println!("⏳ Waiting 10 seconds until next refresh...");
        println!();

        // Wait 10 seconds before next iteration
        sleep(Duration::from_secs(10)).await;
        iteration += 1;
    }
}

async fn post_slack(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("📤 Slack Post - Sending Today's Todos");
    println!("=====================================");

    // Get today's date
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d");

    // Construct the path to today's markdown file
    let slaist_dir = get_todos_directory(config);
    let filename = format!("{}.md", date_str);
    let file_path = slaist_dir.join(&filename);
    println!("📁 Looking for file: {}", file_path.display());

    // Check if the file exists
    if !file_path.exists() {
        eprintln!("❌ Error: Todo file for today ({}) not found!", date_str);
        eprintln!("   Expected location: {}", file_path.display());
        eprintln!("   Run the main slaist application first to generate the file.");
        return Ok(());
    }

    // Read the markdown content
    let markdown_content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", file_path.display(), e);
            return Ok(());
        }
    };

    if markdown_content.trim().is_empty() {
        eprintln!("⚠️  Warning: Todo file is empty!");
        return Ok(());
    }

    println!(
        "📋 Found todo content ({} characters)",
        markdown_content.len()
    );

    // Extract existing Slack message ID if present
    let existing_message_id = extract_slack_message_id(&markdown_content);

    // Validate message ID if present
    if let Some(ref msg_id) = existing_message_id {
        if !validate_message_id(msg_id) {
            eprintln!("⚠️  Warning: Invalid message ID format found: {}", msg_id);
            eprintln!("   Expected format: 1234567890.123456");
            eprintln!("   Will attempt to post as new message instead");
        } else {
            println!("📋 Found existing message ID: {}", msg_id);
        }
    }

    // Create Slack client
    let slack_client = match SlackClient::with_bot_token(config.slack_bot_token.clone()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("❌ Error creating Slack client: {}", e);
            eprintln!("   Please check your config file at ~/slaist/config.toml");
            eprintln!("   Make sure slack_bot_token is set correctly.");
            eprintln!("   ");
            eprintln!("   To get a bot token:");
            eprintln!("   1. Go to https://api.slack.com/apps");
            eprintln!("   2. Create a new app or select an existing one");
            eprintln!("   3. Go to 'OAuth & Permissions' and add 'chat:write' scope");
            eprintln!("   4. Install the app to your workspace");
            eprintln!("   5. Copy the 'Bot User OAuth Token' to your config file");
            eprintln!("   ");
            eprintln!("   Example config.toml:");
            eprintln!("   slack_bot_token = \"xoxb-your-bot-token-here\"");
            eprintln!(
                "   slack_channel = \"#your-channel-name\"  # optional, defaults to #general"
            );
            return Ok(());
        }
    };

    // Filter out metadata from markdown content before sending to Slack
    let filtered_markdown = filter_slack_metadata(&markdown_content);

    // Prepare the message
    let message = format!("📅 *Daily Todos - {}*\n\n{}", date_str, filtered_markdown);

    // Get channel from config or use default
    let channel = config.slack_channel.as_deref().unwrap_or("#general");

    println!("📢 Posting to channel: {}", channel);

    // Post or update message to Slack
    match existing_message_id {
        Some(message_id) if validate_message_id(&message_id) => {
            // Update existing message
            println!("🔄 Updating existing message...");
            match slack_client
                .update_message(&message, &channel, &message_id)
                .await
            {
                Ok(_) => {
                    println!("✅ Successfully updated today's todos on Slack!");
                    println!("   Date: {}", date_str);
                    println!("   Channel: {}", channel);
                    println!("   Message ID: {}", message_id);
                    println!("   Content length: {} characters", filtered_markdown.len());
                }
                Err(e) => {
                    eprintln!("❌ Error updating Slack message: {}", e);
                    eprintln!("   Message ID: {}", message_id);
                    eprintln!("   Channel: {}", channel);

                    // Provide specific error guidance
                    match e {
                        slack::SlackError::ApiError(ref msg) => {
                            if msg.contains("cant_update_message") {
                                eprintln!(
                                    "   → This usually means the bot doesn't have permission to update this message"
                                );
                                eprintln!("   → Or the message was posted by a different bot/user");
                            } else if msg.contains("message_not_found") {
                                eprintln!("   → The message with this ID no longer exists");
                            }
                        }
                        _ => {}
                    }

                    // Try to post as new message if update fails
                    println!("🔄 Attempting to post as new message...");
                    match slack_client.post_message(&message, &channel).await {
                        Ok(new_message_id) => {
                            println!("✅ Successfully posted new message to Slack!");
                            println!("   New Message ID: {}", new_message_id);
                            // Update the markdown file with new message ID
                            let updated_content =
                                add_slack_message_id(&markdown_content, &new_message_id);
                            if let Err(e) = fs::write(&file_path, updated_content) {
                                eprintln!(
                                    "⚠️  Warning: Could not update file with new message ID: {}",
                                    e
                                );
                            } else {
                                println!("📝 Updated markdown file with new message ID");
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Error posting new message to Slack: {}", e);
                        }
                    }
                }
            }
        }
        Some(invalid_id) => {
            eprintln!(
                "⚠️  Skipping update due to invalid message ID: {}",
                invalid_id
            );
            println!("🚀 Sending new message to Slack...");
            match slack_client.post_message(&message, &channel).await {
                Ok(message_id) => {
                    println!("✅ Successfully posted today's todos to Slack!");
                    println!("   Date: {}", date_str);
                    println!("   Channel: {}", channel);
                    println!("   Message ID: {}", message_id);
                    println!("   Content length: {} characters", filtered_markdown.len());

                    // Update the markdown file with the new valid message ID
                    let updated_content = add_slack_message_id(&markdown_content, &message_id);
                    if let Err(e) = fs::write(&file_path, updated_content) {
                        eprintln!("⚠️  Warning: Could not update file with message ID: {}", e);
                    } else {
                        println!("📝 Updated markdown file with valid message ID");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error posting to Slack: {}", e);
                }
            }
        }
        None => {
            // Post new message
            println!("🚀 Sending new message to Slack...");
            match slack_client.post_message(&message, &channel).await {
                Ok(message_id) => {
                    println!("✅ Successfully posted today's todos to Slack!");
                    println!("   Date: {}", date_str);
                    println!("   Channel: {}", channel);
                    println!("   Message ID: {}", message_id);
                    println!("   Content length: {} characters", filtered_markdown.len());

                    // Update the markdown file with the message ID
                    let updated_content = add_slack_message_id(&markdown_content, &message_id);
                    if let Err(e) = fs::write(&file_path, updated_content) {
                        eprintln!("⚠️  Warning: Could not update file with message ID: {}", e);
                    } else {
                        println!("📝 Updated markdown file with message ID for future updates");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error posting to Slack: {}", e);
                    match e {
                        slack::SlackError::HttpError(_) => {
                            eprintln!("   This might be a network connectivity issue.");
                        }
                        slack::SlackError::ApiError(ref msg) => {
                            eprintln!("   Slack API error: {}", msg);
                            if msg.contains("invalid_auth") || msg.contains("not_authed") {
                                eprintln!("   Check your Slack bot token.");
                            }
                        }
                        slack::SlackError::ConfigError(_) => {
                            eprintln!("   Check your slack_bot_token in ~/slaist/config.toml");
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use todoist::Todo;

    #[test]
    fn test_parse_existing_markdown() {
        let markdown = r#"## Active Todos

- [ ] Write tests
- [ ] Fix bug in parser

## Completed Todos

- [x] Review code
- [x] Update documentation *(marked as finished)*
- [x] Deploy to production

*No completed todos yet.*
"#;

        let (todos, notes) = parse_existing_markdown(markdown);

        assert_eq!(todos.len(), 5);
        assert_eq!(todos[0], ("Write tests".to_string(), false));
        assert_eq!(todos[1], ("Fix bug in parser".to_string(), false));
        assert_eq!(todos[2], ("Review code".to_string(), true));
        assert_eq!(todos[3], ("Update documentation".to_string(), true));
        assert_eq!(todos[4], ("Deploy to production".to_string(), true));
        assert_eq!(notes, None);
    }

    #[test]
    fn test_parse_empty_markdown() {
        let markdown = r#"## Active Todos

_No active todos found! 🎉_

## Completed Todos

_No completed todos yet._
"#;

        let (todos, notes) = parse_existing_markdown(markdown);
        assert_eq!(todos.len(), 0);
        assert_eq!(notes, None);
    }

    #[test]
    fn test_generate_markdown_content() {
        let current_todos = vec![
            Todo {
                id: "1".to_string(),
                user_id: "user1".to_string(),
                project_id: "project1".to_string(),
                section_id: None,
                parent_id: None,
                content: "Active task".to_string(),
                description: None,
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: false,
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: None,
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 1,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
            Todo {
                id: "2".to_string(),
                user_id: "user1".to_string(),
                project_id: "project1".to_string(),
                section_id: None,
                parent_id: None,
                content: "Completed task".to_string(),
                description: None,
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: true,
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: Some("2023-01-01T12:00:00Z".to_string()),
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 2,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
        ];

        let existing_todos = vec![
            ("Active task".to_string(), false),
            ("Old task that disappeared".to_string(), false),
            ("Already completed task".to_string(), true),
        ];

        let markdown = generate_markdown_content(&current_todos, &existing_todos, None, None);

        assert!(markdown.contains("## Active Todos"));
        assert!(markdown.contains("- [ ] Active task"));
        assert!(markdown.contains("## Completed Todos"));
        assert!(markdown.contains("- [x] Completed task"));
        assert!(markdown.contains("- [x] Old task that disappeared *(marked as finished)*"));
        assert!(markdown.contains("Already completed task")); // Should preserve previously completed tasks
    }

    #[test]
    fn test_generate_markdown_content_no_todos() {
        let current_todos = vec![];
        let existing_todos = vec![];
        let markdown = generate_markdown_content(&current_todos, &existing_todos, None, None);

        assert!(markdown.contains("_No active todos found! 🎉_"));
        assert!(markdown.contains("_No completed todos yet._"));
    }

    #[test]
    fn test_generate_markdown_content_missing_todos() {
        let current_todos = vec![];
        let existing_todos = vec![
            ("Missing task 1".to_string(), false),
            ("Missing task 2".to_string(), false),
            ("Already completed".to_string(), true),
        ];

        let markdown = generate_markdown_content(&current_todos, &existing_todos, None, None);

        assert!(markdown.contains("- [x] Missing task 1 *(marked as finished)*"));
        assert!(markdown.contains("- [x] Missing task 2 *(marked as finished)*"));
        assert!(markdown.contains("Already completed")); // Should preserve previously completed tasks
    }

    #[test]
    fn test_markdown_generation_with_filter_edge_case() {
        // Test case where a todo moves from active to completed
        let current_todos = vec![Todo {
            id: "1".to_string(),
            user_id: "user1".to_string(),
            project_id: "project1".to_string(),
            section_id: None,
            parent_id: None,
            content: "Task that was completed".to_string(),
            description: None,
            priority: 1,
            labels: vec![],
            due: None,
            deadline: None,
            duration: None,
            checked: true,
            is_deleted: false,
            added_at: "2023-01-01T00:00:00Z".to_string(),
            completed_at: Some("2023-01-01T12:00:00Z".to_string()),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            child_order: 1,
            day_order: None,
            is_collapsed: None,
            added_by_uid: None,
            assigned_by_uid: None,
            responsible_uid: None,
        }];

        let existing_todos = vec![
            ("Task that was completed".to_string(), false), // Was active in markdown
            ("Task that disappeared".to_string(), false),   // No longer in API
        ];

        let markdown = generate_markdown_content(&current_todos, &existing_todos, None, None);

        // Should show the task as completed (from API)
        assert!(markdown.contains("- [x] Task that was completed"));
        // Should NOT double-mark it as finished
        assert!(!markdown.contains("- [x] Task that was completed *(marked as finished)*"));
        // Should mark the disappeared task as finished
        assert!(markdown.contains("- [x] Task that disappeared *(marked as finished)*"));
    }

    #[test]
    fn test_preserve_previously_finished_todos() {
        // Test case where previously marked-as-finished todos are preserved
        let current_todos = vec![Todo {
            id: "1".to_string(),
            user_id: "user1".to_string(),
            project_id: "project1".to_string(),
            section_id: None,
            parent_id: None,
            content: "New active task".to_string(),
            description: None,
            priority: 1,
            labels: vec![],
            due: None,
            deadline: None,
            duration: None,
            checked: false,
            is_deleted: false,
            added_at: "2023-01-01T00:00:00Z".to_string(),
            completed_at: None,
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            child_order: 1,
            day_order: None,
            is_collapsed: None,
            added_by_uid: None,
            assigned_by_uid: None,
            responsible_uid: None,
        }];

        let existing_todos = vec![
            ("Old task marked as finished".to_string(), true), // Was already marked as finished
            ("Another old finished task".to_string(), true),   // Was already marked as finished
            ("Task that just disappeared".to_string(), false), // New missing task
        ];

        let markdown = generate_markdown_content(&current_todos, &existing_todos, None, None);

        // Should preserve previously finished todos
        assert!(markdown.contains("- [x] Old task marked as finished"));
        assert!(markdown.contains("- [x] Another old finished task"));
        // Should mark new missing task as finished
        assert!(markdown.contains("- [x] Task that just disappeared *(marked as finished)*"));
        // Should show active task
        assert!(markdown.contains("- [ ] New active task"));
    }

    #[test]
    fn test_multiple_iteration_workflow() {
        // Test that simulates multiple iterations of the app running
        // This ensures previously marked-as-finished todos are preserved across iterations

        // Initial state: some active todos
        let initial_todos = vec![
            Todo {
                id: "1".to_string(),
                user_id: "user1".to_string(),
                project_id: "project1".to_string(),
                section_id: None,
                parent_id: None,
                content: "Task A".to_string(),
                description: None,
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: false,
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: None,
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 1,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
            Todo {
                id: "2".to_string(),
                user_id: "user1".to_string(),
                project_id: "project1".to_string(),
                section_id: None,
                parent_id: None,
                content: "Task B".to_string(),
                description: None,
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: false,
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: None,
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 2,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
        ];

        // First iteration: generate markdown from initial todos
        let first_markdown = generate_markdown_content(&initial_todos, &[], None, None);
        assert!(first_markdown.contains("- [ ] Task A"));
        assert!(first_markdown.contains("- [ ] Task B"));

        // Parse the first markdown as if it were saved to file
        let (first_parsed, _) = parse_existing_markdown(&first_markdown);

        // Second iteration: Task B disappears (maybe completed outside the filter)
        let second_todos = vec![initial_todos[0].clone()]; // Only Task A remains
        let second_markdown = generate_markdown_content(&second_todos, &first_parsed, None, None);

        // Task B should be marked as finished
        assert!(second_markdown.contains("- [ ] Task A"));
        assert!(second_markdown.contains("- [x] Task B *(marked as finished)*"));

        // Parse the second markdown
        let (second_parsed, _) = parse_existing_markdown(&second_markdown);

        // Third iteration: Task A also disappears, new Task C appears
        let third_todos = vec![Todo {
            id: "3".to_string(),
            user_id: "user1".to_string(),
            project_id: "project1".to_string(),
            section_id: None,
            parent_id: None,
            content: "Task C".to_string(),
            description: None,
            priority: 1,
            labels: vec![],
            due: None,
            deadline: None,
            duration: None,
            checked: false,
            is_deleted: false,
            added_at: "2023-01-01T00:00:00Z".to_string(),
            completed_at: None,
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            child_order: 3,
            day_order: None,
            is_collapsed: None,
            added_by_uid: None,
            assigned_by_uid: None,
            responsible_uid: None,
        }];

        let third_markdown = generate_markdown_content(&third_todos, &second_parsed, None, None);

        // Should preserve Task B as finished from previous iteration (without suffix)
        // Should mark Task A as newly finished (with suffix)
        // Should show Task C as active
        assert!(third_markdown.contains("- [ ] Task C"));
        assert!(third_markdown.contains("- [x] Task B"));
        assert!(!third_markdown.contains("- [x] Task B *(marked as finished)*"));
        assert!(third_markdown.contains("- [x] Task A *(marked as finished)*"));

        // Verify both tasks are preserved in completed section
        let completed_section = third_markdown.split("## Completed Todos").nth(1).unwrap();
        assert!(completed_section.contains("Task A"));
        assert!(completed_section.contains("Task B"));
    }

    #[test]
    fn test_extract_slack_message_id() {
        let content_with_id = r#"<!-- slack_message_id: 1234567890.123456 -->
## Active Todos

- [ ] Test task
"#;
        let extracted = extract_slack_message_id(content_with_id);
        assert_eq!(extracted, Some("1234567890.123456".to_string()));

        let content_without_id = r#"## Active Todos

- [ ] Test task
"#;
        let extracted = extract_slack_message_id(content_without_id);
        assert_eq!(extracted, None);
    }

    #[test]
    fn test_add_slack_message_id() {
        let content = r#"## Active Todos

- [ ] Test task
"#;
        let result = add_slack_message_id(content, "1234567890.123456");
        assert!(result.starts_with("<!-- slack_message_id: 1234567890.123456 -->"));
        assert!(result.contains("## Active Todos"));
    }

    #[test]
    fn test_update_slack_message_id() {
        let content = r#"<!-- slack_message_id: old_id -->
## Active Todos

- [ ] Test task
"#;
        let result = add_slack_message_id(content, "new_id");
        assert!(result.contains("<!-- slack_message_id: new_id -->"));
        assert!(!result.contains("old_id"));
    }

    #[test]
    fn test_filter_slack_metadata() {
        let content = r#"<!-- slack_message_id: 1234567890.123456 -->
## Active Todos

- [ ] Test task

## Completed Todos

- [x] Done task
"#;
        let filtered = filter_slack_metadata(content);
        assert!(!filtered.contains("slack_message_id"));
        assert!(filtered.contains("## Active Todos"));
        assert!(filtered.contains("- [ ] Test task"));
        assert!(filtered.contains("## Completed Todos"));
    }

    #[test]
    fn test_validate_message_id() {
        // Valid message IDs
        assert!(validate_message_id("1234567890.123456"));
        assert!(validate_message_id("1609459200.000001"));
        assert!(validate_message_id("0.0"));

        // Invalid message IDs
        assert!(!validate_message_id("invalid_id"));
        assert!(!validate_message_id("1234567890"));
        assert!(!validate_message_id("1234567890."));
        assert!(!validate_message_id(".123456"));
        assert!(!validate_message_id("1234567890.123456.789"));
        assert!(!validate_message_id("abc.123456"));
        assert!(!validate_message_id("1234567890.abc"));
        assert!(!validate_message_id(""));
    }

    #[test]
    fn test_generate_markdown_content_with_message_id() {
        let markdown = generate_markdown_content(&[], &[], Some("1234567890.123456"), None);
        assert!(markdown.starts_with("<!-- slack_message_id: 1234567890.123456 -->"));
        assert!(markdown.contains("## Active Todos"));
    }

    #[test]
    fn test_message_id_preservation_during_regeneration() {
        // Create initial markdown with a message ID
        let initial_content = r#"<!-- slack_message_id: 1234567890.123456 -->
## Active Todos

- [ ] Task A
- [ ] Task B

## Completed Todos

- [x] Task C
"#;

        // Parse the existing content
        let (existing_todos, _) = parse_existing_markdown(initial_content);
        let message_id = extract_slack_message_id(initial_content);

        // Simulate new todos from API
        let new_todos = vec![
            Todo {
                id: "1".to_string(),
                user_id: "123".to_string(),
                project_id: "456".to_string(),
                section_id: None,
                parent_id: None,
                content: "Task A".to_string(),
                description: Some("".to_string()),
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: false,
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: None,
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 1,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
            Todo {
                id: "2".to_string(),
                user_id: "123".to_string(),
                project_id: "456".to_string(),
                section_id: None,
                parent_id: None,
                content: "Task D".to_string(), // New task
                description: Some("".to_string()),
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: false,
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: None,
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 2,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
        ];

        // Generate new markdown content
        let regenerated_content =
            generate_markdown_content(&new_todos, &existing_todos, message_id.as_deref(), None);

        // Verify the message ID is preserved
        assert!(regenerated_content.starts_with("<!-- slack_message_id: 1234567890.123456 -->"));

        // Verify the content structure is correct
        assert!(regenerated_content.contains("## Active Todos"));
        assert!(regenerated_content.contains("- [ ] Task A"));
        assert!(regenerated_content.contains("- [ ] Task D"));
        assert!(regenerated_content.contains("## Completed Todos"));
        assert!(regenerated_content.contains("- [x] Task B *(marked as finished)*")); // Task B disappeared
        assert!(regenerated_content.contains("- [x] Task C")); // Task C was already completed

        // Verify we can extract the message ID from the regenerated content
        let extracted_id = extract_slack_message_id(&regenerated_content);
        assert_eq!(extracted_id, Some("1234567890.123456".to_string()));
    }

    #[test]
    fn test_expand_tilde_path_with_tilde_only() {
        let result = expand_tilde_path("~");
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        assert_eq!(result, PathBuf::from(home_dir));
    }

    #[test]
    fn test_expand_tilde_path_with_tilde_slash() {
        let result = expand_tilde_path("~/documents/todos");
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let expected = Path::new(&home_dir).join("documents/todos");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_expand_tilde_path_without_tilde() {
        let path = "/absolute/path/to/todos";
        let result = expand_tilde_path(path);
        assert_eq!(result, PathBuf::from(path));
    }

    #[test]
    fn test_expand_tilde_path_relative() {
        let path = "relative/path/to/todos";
        let result = expand_tilde_path(path);
        assert_eq!(result, PathBuf::from(path));
    }

    #[test]
    fn test_get_todos_directory_with_config() {
        let config = Config {
            todoist_api_token: "test".to_string(),
            slack_bot_token: "test".to_string(),
            slack_channel: None,
            filter: None,
            todos_directory: Some("~/custom/todos".to_string()),
        };

        let result = get_todos_directory(&config);
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let expected = Path::new(&home_dir).join("custom/todos");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_todos_directory_without_config() {
        let config = Config {
            todoist_api_token: "test".to_string(),
            slack_bot_token: "test".to_string(),
            slack_channel: None,
            filter: None,
            todos_directory: None,
        };

        let result = get_todos_directory(&config);
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let expected = Path::new(&home_dir).join("slaist");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_markdown_with_notes() {
        let markdown = r#"## Active Todos

- [ ] Write tests
- [ ] Fix bug in parser

## Completed Todos

- [x] Review code

---
# My Notes

This is some important information that I want to keep.

## Meeting Notes
- Discussed the new feature
- Need to refactor the parser

Some additional thoughts..."#;

        let (todos, notes) = parse_existing_markdown(markdown);

        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0], ("Write tests".to_string(), false));
        assert_eq!(todos[1], ("Fix bug in parser".to_string(), false));
        assert_eq!(todos[2], ("Review code".to_string(), true));

        let expected_notes = r#"# My Notes

This is some important information that I want to keep.

## Meeting Notes
- Discussed the new feature
- Need to refactor the parser

Some additional thoughts..."#;

        assert_eq!(notes, Some(expected_notes.to_string()));
    }

    #[test]
    fn test_generate_markdown_with_preserved_notes() {
        let current_todos = vec![Todo {
            id: "1".to_string(),
            user_id: "user1".to_string(),
            project_id: "proj1".to_string(),
            section_id: None,
            parent_id: None,
            content: "New task".to_string(),
            description: None,
            priority: 1,
            labels: vec![],
            due: None,
            deadline: None,
            duration: None,
            checked: false,
            is_deleted: false,
            added_at: "2023-01-01T00:00:00Z".to_string(),
            completed_at: None,
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            child_order: 1,
            day_order: None,
            is_collapsed: None,
            added_by_uid: None,
            assigned_by_uid: None,
            responsible_uid: None,
        }];

        let existing_todos = vec![("Old task".to_string(), false)];

        let notes = r#"# My Notes

This is preserved content.

## Important Links
- https://example.com"#;

        let markdown =
            generate_markdown_content(&current_todos, &existing_todos, None, Some(notes));

        assert!(markdown.contains("- [ ] New task"));
        assert!(markdown.contains("- [x] Old task *(marked as finished)*"));
        assert!(markdown.contains("---"));
        assert!(markdown.contains("# My Notes"));
        assert!(markdown.contains("This is preserved content."));
        assert!(markdown.contains("https://example.com"));
    }

    #[test]
    fn test_notes_section_ends_with_newline() {
        let notes_without_newline = "# My Notes\nSome content";
        let notes_with_newline = "# My Notes\nSome content\n";

        let markdown1 = generate_markdown_content(&[], &[], None, Some(notes_without_newline));
        let markdown2 = generate_markdown_content(&[], &[], None, Some(notes_with_newline));

        // Both should end with exactly one newline
        assert!(markdown1.ends_with('\n'));
        assert!(markdown2.ends_with('\n'));

        // Neither should end with double newlines
        assert!(!markdown1.ends_with("\n\n"));
        assert!(!markdown2.ends_with("\n\n"));
    }

    #[test]
    fn test_full_workflow_with_notes_preservation() {
        // Test the complete workflow: parse markdown with notes, generate new content, preserve notes
        let original_content = r#"## Active Todos

- [ ] Task A
- [ ] Task B

## Completed Todos

- [x] Task C

---
# My Personal Notes

## Meeting Notes
- Important discussion about feature X
- Deadline is next Friday

## Links
- [Documentation](https://example.com/docs)
- [GitHub Issue](https://github.com/example/issue)

Some additional thoughts and reminders..."#;

        // Parse the original content
        let (parsed_todos, parsed_notes) = parse_existing_markdown(original_content);

        // Verify parsing worked correctly
        assert_eq!(parsed_todos.len(), 3);
        assert_eq!(parsed_todos[0], ("Task A".to_string(), false));
        assert_eq!(parsed_todos[1], ("Task B".to_string(), false));
        assert_eq!(parsed_todos[2], ("Task C".to_string(), true));
        assert!(parsed_notes.is_some());
        let notes = parsed_notes.as_ref().unwrap();
        assert!(notes.contains("# My Personal Notes"));
        assert!(notes.contains("Meeting Notes"));
        assert!(notes.contains("https://example.com/docs"));

        // Simulate new todos from API (Task A completed, Task B missing, new Task D)
        let new_todos = vec![
            Todo {
                id: "1".to_string(),
                user_id: "user1".to_string(),
                project_id: "proj1".to_string(),
                section_id: None,
                parent_id: None,
                content: "Task A".to_string(),
                description: None,
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: true, // Now completed
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: Some("2023-01-01T12:00:00Z".to_string()),
                updated_at: "2023-01-01T12:00:00Z".to_string(),
                child_order: 1,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
            Todo {
                id: "4".to_string(),
                user_id: "user1".to_string(),
                project_id: "proj1".to_string(),
                section_id: None,
                parent_id: None,
                content: "Task D".to_string(),
                description: None,
                priority: 1,
                labels: vec![],
                due: None,
                deadline: None,
                duration: None,
                checked: false, // New active task
                is_deleted: false,
                added_at: "2023-01-01T00:00:00Z".to_string(),
                completed_at: None,
                updated_at: "2023-01-01T00:00:00Z".to_string(),
                child_order: 4,
                day_order: None,
                is_collapsed: None,
                added_by_uid: None,
                assigned_by_uid: None,
                responsible_uid: None,
            },
        ];

        // Generate new markdown content
        let regenerated_content =
            generate_markdown_content(&new_todos, &parsed_todos, None, parsed_notes.as_deref());

        // Verify the regenerated content
        assert!(regenerated_content.contains("## Active Todos"));
        assert!(regenerated_content.contains("- [ ] Task D")); // New active task
        assert!(regenerated_content.contains("## Completed Todos"));
        assert!(regenerated_content.contains("- [x] Task A")); // Completed in API
        assert!(regenerated_content.contains("- [x] Task C")); // Previously completed, preserved
        assert!(regenerated_content.contains("- [x] Task B *(marked as finished)*")); // Missing from API

        // Most importantly, verify notes are preserved
        assert!(regenerated_content.contains("---"));
        assert!(regenerated_content.contains("# My Personal Notes"));
        assert!(regenerated_content.contains("Meeting Notes"));
        assert!(regenerated_content.contains("- Important discussion about feature X"));
        assert!(regenerated_content.contains("[Documentation](https://example.com/docs)"));
        assert!(regenerated_content.contains("Some additional thoughts and reminders..."));

        // Verify the notes section is at the end
        let parts: Vec<&str> = regenerated_content.split("---").collect();
        assert_eq!(parts.len(), 2);
        let notes_section = parts[1];
        assert!(notes_section.contains("# My Personal Notes"));
    }
}
