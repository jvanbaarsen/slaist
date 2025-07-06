# Slaist - Todoist To Slack sync

A Rust application that continuously fetches Todoist todo's and posts them to a Slack channel

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

### Running Tests

```bash
# Test the Todoist client library
cargo test --package todoist

# Test the main application
cargo test --package slaist
```

## License

This project is licensed under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request
