# Slaist - Todoist Continuous Refresh

A Rust application that continuously fetches and displays your Todoist todos with automatic refresh every 10 seconds.

## Features

- 🔄 **Continuous Refresh**: Automatically fetches todos every 10 seconds
- 📊 **Real-time Statistics**: Shows active, completed, and high-priority todo counts
- 🎯 **Priority Visualization**: Color-coded priority indicators
- 📅 **Due Date Tracking**: Highlights todos due today
- 🔥 **High Priority Focus**: Separate section for urgent tasks
- 🌐 **Error Handling**: Graceful handling of network issues and API errors

## Prerequisites

1. **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
2. **Todoist API Token**: Get your token from [Todoist Integrations](https://todoist.com/prefs/integrations)

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

3. **Run the continuous refresh**:
   ```bash
   cargo run --package slaist
   ```

   Or use the demo script:
   ```bash
   ./demo.sh
   ```

## Usage

Once running, the application will:

1. **Connect to Todoist** using your API token
2. **Display all todos** with priority indicators and due dates
3. **Show statistics** including active, completed, and high-priority counts
4. **Highlight urgent tasks** in a separate high-priority section
5. **Refresh automatically** every 10 seconds
6. **Continue until stopped** with Ctrl+C

## Example Output

```
🚀 Todoist Client - Continuous Refresh
=======================================
📱 Fetching todos every 10 seconds... (Press Ctrl+C to stop)

🔄 Refresh #1 - 2023-12-08 14:30:15 UTC
------------------------------------------------------------
📋 Your Todos (8 total):
1 📝 🔴 Complete project proposal
     📅 Due: today
     🏷️  Labels: @work, @urgent
2 📝 🟠 Review code changes
3 📝 🟡 Buy groceries
     📅 Due: tomorrow
4 📝 ⚪ Call dentist
   ... and 4 more todos

📊 Statistics:
   📝 Total todos: 8
   ✅ Active todos: 6
   ✔️  Completed todos: 2
   🔥 High priority: 2
   📅 Due today: 1

🔥 High Priority Todos (2):
   📝 Complete project proposal (P4)
   📝 Review code changes (P3)

------------------------------------------------------------
⏳ Waiting 10 seconds until next refresh...
```

## Configuration

The application uses environment variables for configuration:

- `TODOIST_API_TOKEN`: Your Todoist API token (required)

## Error Handling

The application handles various error scenarios:

- **Invalid API Token**: Clear message with instructions
- **Network Issues**: Retry with next refresh cycle
- **API Rate Limits**: Graceful handling with error display
- **Connection Problems**: Continues running and retries

## Development

### Project Structure

```
slaist/
├── app/                    # Main application
│   ├── src/main.rs        # Continuous refresh logic
│   └── Cargo.toml         # App dependencies
├── todoist/               # Todoist API client library
│   ├── src/lib.rs         # Client implementation
│   ├── examples/          # Usage examples
│   ├── tests/             # Integration tests
│   └── Cargo.toml         # Library dependencies
├── demo.sh                # Demo script
└── README.md              # This file
```

### Running Tests

```bash
# Test the Todoist client library
cargo test --package todoist

# Test the main application
cargo test --package slaist
```

### Example Usage

```bash
# Run with detailed example
cargo run --example fetch_todos

# Run the continuous refresh
cargo run --package slaist

# Run with demo script
./demo.sh
```

## Todoist API Features

The underlying library supports:

- ✅ Fetch all todos
- ✅ Filter todos by project, section, label, priority
- ✅ Get individual todos
- ✅ Mark todos as completed
- ✅ Create new todos
- ✅ Fetch all projects
- ✅ Comprehensive error handling

## Dependencies

- **tokio**: Async runtime
- **reqwest**: HTTP client
- **serde**: JSON serialization
- **chrono**: Date/time handling
- **thiserror**: Error handling

## License

This project is licensed under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Support

For issues and questions:
1. Check the error messages for API token or network issues
2. Verify your Todoist API token is valid
3. Check your internet connection
4. Review the logs for detailed error information