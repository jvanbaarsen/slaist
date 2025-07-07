#!/bin/bash

# post-to-slack.sh - Convenience script to post today's todos to Slack

set -e

echo "🚀 Posting today's todos to Slack..."
echo "===================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the slaist project root directory"
    exit 1
fi

# Build the project if needed
echo "🔨 Building slack-post binary..."
cargo build --release --bin slack-post

# Run the slack-post binary
echo "📤 Running slack-post..."
./target/release/slack-post

echo "✅ Done!"
