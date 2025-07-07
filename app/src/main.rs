use chrono::Utc;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use todoist::{Todo, TodoistClient, TodoistError};
use tokio::time::{Duration, sleep};

/// Parse existing markdown file to extract todo items
fn parse_existing_markdown(content: &str) -> Vec<(String, bool)> {
    let mut todos = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let trimmed = line.trim();
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
    }
    todos
}

/// Generate markdown content with comparison logic
fn generate_markdown_content(current_todos: &[Todo], existing_todos: &[(String, bool)]) -> String {
    let mut content = String::new();

    // Create a set of current todo contents for fast lookup
    let current_todo_contents: HashSet<String> = current_todos
        .iter()
        .map(|todo| todo.content.clone())
        .collect();

    // Active todos section
    content.push_str("## Active Todos\n\n");

    let active_todos: Vec<_> = current_todos.iter().filter(|todo| !todo.checked).collect();
    if active_todos.is_empty() {
        content.push_str("*No active todos found! 🎉*\n\n");
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
        content.push_str("*No completed todos yet.*\n\n");
    }

    content
}

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

        // Read existing markdown file if it exists
        let existing_todos = if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => parse_existing_markdown(&content),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
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
        let markdown_content = generate_markdown_content(&all_current_todos, &existing_todos);

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

        println!("\n{:-<60}", "");
        println!("⏳ Waiting 10 seconds until next refresh...");
        println!();

        // Wait 10 seconds before next iteration
        sleep(Duration::from_secs(10)).await;
        iteration += 1;
    }
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

        let todos = parse_existing_markdown(markdown);
        assert_eq!(todos.len(), 5);

        assert_eq!(todos[0], ("Write tests".to_string(), false));
        assert_eq!(todos[1], ("Fix bug in parser".to_string(), false));
        assert_eq!(todos[2], ("Review code".to_string(), true));
        assert_eq!(todos[3], ("Update documentation".to_string(), true));
        assert_eq!(todos[4], ("Deploy to production".to_string(), true));
    }

    #[test]
    fn test_parse_empty_markdown() {
        let markdown = r#"## Active Todos

*No active todos found! 🎉*

## Completed Todos

*No completed todos yet.*
"#;

        let todos = parse_existing_markdown(markdown);
        assert_eq!(todos.len(), 0);
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

        let markdown = generate_markdown_content(&current_todos, &existing_todos);

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

        let markdown = generate_markdown_content(&current_todos, &existing_todos);

        assert!(markdown.contains("*No active todos found! 🎉*"));
        assert!(markdown.contains("*No completed todos yet.*"));
    }

    #[test]
    fn test_generate_markdown_content_missing_todos() {
        let current_todos = vec![];
        let existing_todos = vec![
            ("Missing task 1".to_string(), false),
            ("Missing task 2".to_string(), false),
            ("Already completed".to_string(), true),
        ];

        let markdown = generate_markdown_content(&current_todos, &existing_todos);

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

        let markdown = generate_markdown_content(&current_todos, &existing_todos);

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

        let markdown = generate_markdown_content(&current_todos, &existing_todos);

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
        let first_markdown = generate_markdown_content(&initial_todos, &[]);
        assert!(first_markdown.contains("- [ ] Task A"));
        assert!(first_markdown.contains("- [ ] Task B"));

        // Parse the first markdown as if it were saved to file
        let first_parsed = parse_existing_markdown(&first_markdown);

        // Second iteration: Task B disappears (maybe completed outside the filter)
        let second_todos = vec![initial_todos[0].clone()]; // Only Task A remains
        let second_markdown = generate_markdown_content(&second_todos, &first_parsed);

        // Task B should be marked as finished
        assert!(second_markdown.contains("- [ ] Task A"));
        assert!(second_markdown.contains("- [x] Task B *(marked as finished)*"));

        // Parse the second markdown
        let second_parsed = parse_existing_markdown(&second_markdown);

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

        let third_markdown = generate_markdown_content(&third_todos, &second_parsed);

        // Should preserve Task B as finished from previous iteration (without suffix)
        // Should mark Task A as newly finished (with suffix)
        // Should show Task C as active
        assert!(third_markdown.contains("- [ ] Task C"));
        assert!(third_markdown.contains("- [x] Task A *(marked as finished)*"));
        assert!(third_markdown.contains("- [x] Task B"));
        assert!(!third_markdown.contains("- [x] Task B *(marked as finished)*"));

        // Verify both tasks are preserved in completed section
        let completed_section = third_markdown.split("## Completed Todos").nth(1).unwrap();
        assert!(completed_section.contains("Task A"));
        assert!(completed_section.contains("Task B"));
    }
}
