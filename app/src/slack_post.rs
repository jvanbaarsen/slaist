use chrono::Utc;
use slack::SlackClient;
use std::env;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📤 Slack Post - Sending Today's Todos");
    println!("=====================================");

    // Get today's date
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d");

    // Construct the path to today's markdown file
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let slaist_dir = Path::new(&home_dir).join("slaist");
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

    // Create Slack client
    let slack_client = match SlackClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("❌ Error creating Slack client: {}", e);
            eprintln!("   Please set the SLACK_BOT_TOKEN environment variable:");
            eprintln!("   export SLACK_BOT_TOKEN=\"xoxb-your-bot-token-here\"");
            eprintln!("   ");
            eprintln!("   To get a bot token:");
            eprintln!("   1. Go to https://api.slack.com/apps");
            eprintln!("   2. Create a new app or select an existing one");
            eprintln!("   3. Go to 'OAuth & Permissions' and add 'chat:write' scope");
            eprintln!("   4. Install the app to your workspace");
            eprintln!("   5. Copy the 'Bot User OAuth Token'");
            eprintln!("   ");
            eprintln!("   Optional: Set SLACK_CHANNEL to specify the channel:");
            eprintln!("   export SLACK_CHANNEL=\"#your-channel-name\"");
            eprintln!("   (defaults to #general if not set)");
            return Ok(());
        }
    };

    // Prepare the message
    let message = format!("📅 *Daily Todos - {}*\n\n{}", date_str, markdown_content);

    println!("🚀 Sending message to Slack...");

    // Get channel from environment or use default
    let channel = env::var("SLACK_CHANNEL").unwrap_or_else(|_| "#general".to_string());

    println!("📢 Posting to channel: {}", channel);

    // Post to Slack
    match slack_client.post_message(&message, &channel).await {
        Ok(_) => {
            println!("✅ Successfully posted today's todos to Slack!");
            println!("   Date: {}", date_str);
            println!("   Channel: {}", channel);
            println!("   Content length: {} characters", markdown_content.len());
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
                    eprintln!("   Check your SLACK_BOT_TOKEN environment variable.");
                }
                _ => {}
            }
        }
    }

    Ok(())
}
