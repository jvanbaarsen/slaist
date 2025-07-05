use std::env;
use todoist::{TodoistClient, TodoistError};

#[tokio::main]
async fn main() -> Result<(), TodoistError> {
    // Get the API token from environment variable
    let token =
        env::var("TODOIST_API_TOKEN").expect("TODOIST_API_TOKEN environment variable must be set");

    // Create a new Todoist client
    let client = TodoistClient::new(token);

    println!("Fetching all todos from Todoist...\n");

    // Fetch all todos
    match client.get_all_todos().await {
        Ok(todos) => {
            println!("Found {} todos:", todos.len());
            println!("{:-<80}", "");

            for todo in &todos {
                println!("📋 {}", todo.content);

                if let Some(description) = &todo.description {
                    if !description.is_empty() {
                        println!("   Description: {}", description);
                    }
                }

                println!("   ID: {}", todo.id);
                println!("   Priority: {}", todo.priority);
                println!("   Completed: {}", todo.is_completed);

                if let Some(due) = &todo.due {
                    println!("   Due: {}", due.string);
                }

                if !todo.labels.is_empty() {
                    println!("   Labels: {}", todo.labels.join(", "));
                }

                println!("   URL: {}", todo.url);
                println!();
            }

            // Show some statistics
            let completed_count = todos.iter().filter(|t| t.is_completed).count();
            let active_count = todos.len() - completed_count;
            let high_priority_count = todos.iter().filter(|t| t.priority >= 3).count();

            println!("{:-<80}", "");
            println!("📊 Statistics:");
            println!("   Total todos: {}", todos.len());
            println!("   Active todos: {}", active_count);
            println!("   Completed todos: {}", completed_count);
            println!("   High priority todos: {}", high_priority_count);
        }
        Err(e) => {
            eprintln!("Error fetching todos: {}", e);
            return Err(e);
        }
    }

    println!("\n{:-<80}", "");
    println!("Fetching todos with filters...\n");

    // Example: Fetch todos with high priority only
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
        Ok(high_priority_todos) => {
            println!("High priority todos ({}):", high_priority_todos.len());
            for todo in &high_priority_todos {
                println!("🔥 {} (Priority: {})", todo.content, todo.priority);
            }
        }
        Err(e) => {
            eprintln!("Error fetching high priority todos: {}", e);
        }
    }

    println!("\n{:-<80}", "");
    println!("Fetching all projects...\n");

    // Fetch all projects to show project names with todos
    match client.get_all_projects().await {
        Ok(projects) => {
            println!("Your projects:");
            for project in &projects {
                println!("📁 {} (ID: {})", project.name, project.id);

                // Fetch todos for this specific project
                match client
                    .get_todos_with_filters(Some(&project.id), None, None, None, None, None)
                    .await
                {
                    Ok(project_todos) => {
                        println!("   {} todos in this project", project_todos.len());
                        for todo in project_todos.iter().take(3) {
                            println!("   - {}", todo.content);
                        }
                        if project_todos.len() > 3 {
                            println!("   ... and {} more", project_todos.len() - 3);
                        }
                    }
                    Err(e) => {
                        eprintln!("   Error fetching project todos: {}", e);
                    }
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error fetching projects: {}", e);
        }
    }

    Ok(())
}
