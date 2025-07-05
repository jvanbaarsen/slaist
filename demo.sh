#!/bin/bash

# Demo script for Todoist continuous refresh functionality
# This script demonstrates the 10-second refresh feature

echo "🚀 Todoist Continuous Refresh Demo"
echo "=================================="
echo ""

# Check if API token is set
if [ -z "$TODOIST_API_TOKEN" ]; then
    echo "⚠️  TODOIST_API_TOKEN environment variable not set"
    echo ""
    echo "To run this demo, you need to:"
    echo "1. Get your API token from https://todoist.com/prefs/integrations"
    echo "2. Export it as an environment variable:"
    echo "   export TODOIST_API_TOKEN=\"your_actual_token_here\""
    echo ""
    echo "Example usage:"
    echo "   export TODOIST_API_TOKEN=\"abcd1234567890\""
    echo "   ./demo.sh"
    echo ""
    exit 1
fi

echo "✅ API token found"
echo "📱 Starting continuous refresh (every 10 seconds)"
echo "🛑 Press Ctrl+C to stop"
echo ""

# Add a small delay to let user read the message
sleep 2

# Run the Rust application
cargo run --package slaist
