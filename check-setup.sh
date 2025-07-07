#!/bin/bash

# check-setup.sh - Health check script for Slaist setup

set -e

echo "🔍 Slaist Setup Health Check"
echo "============================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the slaist project root directory"
    exit 1
fi

# Check Rust installation
echo "🦀 Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi
echo "✅ Rust/Cargo found: $(cargo --version)"

# Check if project builds
echo "🔨 Checking if project builds..."
if ! cargo build --release &> /dev/null; then
    echo "❌ Project build failed. Run 'cargo build --release' for details"
    exit 1
fi
echo "✅ Project builds successfully"

# Check Todoist API token
echo "🔑 Checking Todoist API token..."
if [ -z "$TODOIST_API_TOKEN" ]; then
    echo "⚠️  TODOIST_API_TOKEN environment variable not set"
    echo "   Set it with: export TODOIST_API_TOKEN=\"your_token_here\""
    echo "   Get your token from: https://todoist.com/prefs/integrations"
    TODOIST_OK=false
else
    echo "✅ TODOIST_API_TOKEN is set"
    TODOIST_OK=true
fi

# Check Slack configuration
echo "💬 Checking Slack configuration..."
if [ -n "$SLACK_BOT_TOKEN" ]; then
    echo "✅ SLACK_BOT_TOKEN is set"
    if [ -n "$SLACK_CHANNEL" ]; then
        echo "✅ SLACK_CHANNEL is set to: $SLACK_CHANNEL"
    else
        echo "ℹ️  SLACK_CHANNEL not set (will use #general)"
    fi
    SLACK_OK=true
else
    echo "⚠️  SLACK_BOT_TOKEN is not set"
    echo "   Set it with: export SLACK_BOT_TOKEN=\"xoxb-your-bot-token-here\""
    echo "   Get your token from: https://api.slack.com/apps"
    echo "   Optionally set channel: export SLACK_CHANNEL=\"#your-channel-name\""
    SLACK_OK=false
fi

# Check slaist directory
echo "📁 Checking slaist directory..."
SLAIST_DIR="$HOME/slaist"
if [ ! -d "$SLAIST_DIR" ]; then
    echo "⚠️  Directory $SLAIST_DIR does not exist (will be created on first run)"
else
    echo "✅ Directory $SLAIST_DIR exists"
    # Check for existing todo files
    TODO_FILES=$(find "$SLAIST_DIR" -name "*.md" -type f | wc -l)
    if [ "$TODO_FILES" -gt 0 ]; then
        echo "✅ Found $TODO_FILES existing todo file(s)"
        echo "   Latest files:"
        find "$SLAIST_DIR" -name "*.md" -type f | sort | tail -3 | sed 's/^/     /'
    else
        echo "ℹ️  No existing todo files found (will be created on first run)"
    fi
fi

# Check if scripts are executable
echo "🔧 Checking script permissions..."
if [ -x "post-to-slack.sh" ]; then
    echo "✅ post-to-slack.sh is executable"
else
    echo "⚠️  post-to-slack.sh is not executable. Run: chmod +x post-to-slack.sh"
fi

# Summary
echo ""
echo "📋 Setup Summary"
echo "================"
echo "Build Status: ✅ OK"
if [ "$TODOIST_OK" = true ]; then
    echo "Todoist API: ✅ OK"
else
    echo "Todoist API: ❌ NEEDS SETUP"
fi
if [ "$SLACK_OK" = true ]; then
    echo "Slack Integration: ✅ OK"
else
    echo "Slack Integration: ❌ NEEDS SETUP"
fi

echo ""
if [ "$TODOIST_OK" = true ] && [ "$SLACK_OK" = true ]; then
    echo "🎉 All systems ready! You can now:"
    echo "   • Run: cargo run --package slaist (to start monitoring)"
    echo "   • Run: ./post-to-slack.sh (to post to Slack)"
elif [ "$TODOIST_OK" = true ]; then
    echo "⚠️  Ready for todo monitoring, but Slack integration needs setup"
    echo "   • Run: cargo run --package slaist (to start monitoring)"
    echo "   • Set up Slack integration for posting"
else
    echo "❌ Setup incomplete. Please configure missing components above."
fi
