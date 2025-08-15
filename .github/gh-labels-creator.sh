#!/bin/bash

# Check if GitHub CLI and jq are installed
if ! command -v gh &> /dev/null || ! command -v jq &> /dev/null; then
    echo "Error: GitHub CLI (gh) and jq are required."
    echo "Install gh: https://cli.github.com/"
    echo "Install jq: https://stedolan.github.io/jq/"
    exit 1
fi

# Prompt for confirmation to delete all existing labels
read -p "Delete ALL existing labels? (y/N) " confirm
if [[ "$confirm" =~ ^[Yy]$ ]]; then
    echo "Deleting all existing labels..."
    gh label list --json name --jq '.[].name' | xargs -r -I {} gh label delete {} --yes || {
        echo "Error deleting labels. Continuing with label creation..."
    }
else
    echo "Skipping label deletion."
fi

gh label create "bug" --color d73a4a --description "Something isn't working"
gh label create "feature" --color a2eeef --description "New feature request"
gh label create "documentation" --color 0075ca --description "Improvements or additions to documentation"
gh label create "question" --color d876e3 --description "Further information is requested"
gh label create "discussion" --color 8b949e --description "Open-ended conversation or design discussion"
gh label create "security" --color b60205 --description "Security-related issue"

gh label create "priority: high" --color b60205 --description "Critical, needs immediate attention"
gh label create "priority: medium" --color fbca04 --description "Important but not urgent"
gh label create "priority: low" --color 0e8a16 --description "Low urgency, can wait"
gh label create "blocked" --color e4e669 --description "Cannot proceed due to dependency/blocker"

gh label create "status: in progress" --color 1d76db --description "Currently being worked on"
gh label create "status: needs review" --color dbab09 --description "Waiting for review"
gh label create "status: needs triage" --color e4e669 --description "Needs categorization or investigation"
gh label create "status: duplicate" --color cccccc --description "Duplicate of another issue/PR"
gh label create "status: wontfix" --color ffffff --description "Not planned to be fixed or implemented"

gh label create "refactor" --color 0366d6 --description "Code improvements without behavior change"
gh label create "performance" --color 5319e7 --description "Performance-related improvement"
gh label create "tests" --color f4c542 --description "Related to automated/manual tests"
gh label create "chore" --color fef2c0 --description "Maintenance task, tooling update, cleanup"
gh label create "deps" --color cfd3d7 --description "Dependency updates or changes"