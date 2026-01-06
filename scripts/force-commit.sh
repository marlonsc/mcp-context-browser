#!/bin/bash

# Force Commit Script for MCP Context Browser
# This script adds all changes, commits with force, and pushes to GitHub

set -e  # Exit on any error

echo "🔄 Starting force commit process..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Check if there are uncommitted changes
if git diff --quiet && git diff --staged --quiet; then
    echo "ℹ️  No changes to commit"
    exit 0
fi

# Add all changes
echo "📁 Adding all changes..."
git add -A

# Commit with force flag (allow empty commits)
COMMIT_MSG="Force commit: $(date '+%Y-%m-%d %H:%M:%S') - Automated update"
echo "💾 Committing changes: $COMMIT_MSG"
git commit --allow-empty -m "$COMMIT_MSG"

# Push with force
echo "🚀 Pushing to remote repository..."
if git push --force-with-lease origin main 2>/dev/null; then
    echo "✅ Successfully pushed with --force-with-lease"
elif git push --force origin main 2>/dev/null; then
    echo "✅ Successfully pushed with --force"
else
    echo "❌ Error: Failed to push changes"
    exit 1
fi

echo "🎉 Force commit and push completed successfully!"
echo "📊 Repository is now clean and up-to-date on GitHub"