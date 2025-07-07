# Slaist - Todoist To Slack sync

A Rust application that continuously fetches Todoist todo's and posts them to a Slack channel. It can both continuously monitor your todos and post daily summaries to Slack.

## Prerequisites

1. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
2. **Todoist API Token**: Get your token from [Todoist Integrations](https://todoist.com/prefs/integrations)
3. **Slack Integration** (optional): Slack Bot Token for posting messages

## Quick Start

1. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd slaist
   cargo build --release
   ```

2. **Set your API token**:
   ```bash
   export TODOIST_API_TOKEN="your_api_token_here"
   ```

3. **Set up Slack integration** (optional):
   ```bash
   export SLACK_BOT_TOKEN="xoxb-your-bot-token-here"
   ```

4. **Run the continuous refresh**:
   ```bash
   cargo run --package slaist
   ```

   Or use the demo script:
   ```bash
   ./demo.sh
   ```

5. **Post today's todos to Slack**:
   ```bash
   ./post-to-slack.sh
   ```

   Or run directly:
   ```bash
   cargo run --bin slack-post
   ```

6. **Check your setup**:
   ```bash
   ./check-setup.sh
   ```

## File Output

Each refresh creates/updates a markdown file in `~/slaist/[date].md` with:

- **Structured markdown**: Proper headers and formatting
- **GitHub-style checkboxes**: `- [ ]` for incomplete, `- [x]` for completed todos
- **All todos**: Complete list with priorities, due dates, and labels
- **Statistics**: Summary of todo counts and status
- **High priority section**: Separate listing of urgent tasks
- **Timestamp**: When the data was last updated

Example file: `~/slaist/2023-12-08.md`

## Configuration

The application uses environment variables for configuration:

- `TODOIST_API_TOKEN`: Your Todoist API token (required)
- `SLACK_BOT_TOKEN`: Slack bot token for posting messages (optional)
- `SLACK_CHANNEL`: Slack channel to post to (optional, defaults to #general)

## Slack Integration

The application supports posting your daily todos to Slack using a bot token.

### Setting up Slack Bot Token

1. Create a Slack app at [api.slack.com](https://api.slack.com/apps)
3. Go to "OAuth & Permissions" and add the `chat:write` scope
4. Install the app to your workspace
5. Copy the Bot User OAuth Token
6. Set the environment variable: `export SLACK_BOT_TOKEN="your_bot_token"`
7. Optionally set the channel: `export SLACK_CHANNEL="#your-channel-name"`

### Posting to Slack

Once configured, you can post today's todos to Slack using:

```bash
./post-to-slack.sh
```

This will:
- Find today's todo markdown file in `~/slaist/`
- Format it as a Slack message
- Post it to your configured Slack channel

## Example Usage

### Basic Workflow

1. **Start the continuous todo monitoring**:
   ```bash
   export TODOIST_API_TOKEN="your_token_here"
   cargo run --package slaist
   ```
   This creates/updates `~/slaist/YYYY-MM-DD.md` files every 10 seconds.

2. **Post today's todos to Slack**:
   ```bash
   export SLACK_BOT_TOKEN="xoxb-your-bot-token-here"
   export SLACK_CHANNEL="#your-channel-name"  # Optional, defaults to #general
   ./post-to-slack.sh
   ```

### Sample Slack Message

When posted to Slack, your todos will appear formatted like this:

```
📅 Daily Todos - 2025-07-07

## Active Todos

- [ ] Check Appsignal errors
- [ ] Get the production env ready for hosted collector
- [ ] Work on more collector feedback
- [ ] Development team call
- [ ] Hosted collector + installation flow

## Completed Today

- [x] Review pull request #123
- [x] Update documentation
```

### Automation

You can automate daily Slack posts by setting up a cron job:

```bash
# Post todos to Slack every weekday at 9 AM
0 9 * * 1-5 cd /path/to/slaist && ./post-to-slack.sh
```

### Running Tests

```bash
# Test the Todoist client library
cargo test --package todoist

# Test the main application
cargo test --package slaist

# Test the Slack integration
cargo test --package slack
```

## Health Check

Use the health check script to verify your setup:

```bash
./check-setup.sh
```

This will check:
- Rust installation
- Project build status
- Environment variables (API tokens)
- Directory structure
- Script permissions
- Existing todo files

## License

This project is licensed under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request
