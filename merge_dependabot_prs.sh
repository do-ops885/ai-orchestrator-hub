#!/bin/bash

REPO="https://github.com/do-ops885/ai-orchestrator-hub"

echo "=== Dependabot PR Merge Report ==="
echo "Repository: $REPO"
echo "Date: $(date)"
echo ""

# List open PRs by dependabot
echo "Fetching open PRs authored by dependabot..."
PRS=$(gh pr list --author "dependabot[bot]" --state open --repo "$REPO" --json number,title,headRefName,mergeable --jq '.[] | @base64')

if [ -z "$PRS" ]; then
    echo "No open PRs found from dependabot."
    exit 0
fi

echo "Found the following open dependabot PRs:"
echo "$PRS" | while read -r pr_data; do
    pr_info=$(echo "$pr_data" | base64 -d)
    number=$(echo "$pr_info" | jq -r '.number')
    title=$(echo "$pr_info" | jq -r '.title')
    head=$(echo "$pr_info" | jq -r '.headRefName')
    mergeable=$(echo "$pr_info" | jq -r '.mergeable')
    
    echo "PR #$number: $title (branch: $head, mergeable: $mergeable)"
done
echo ""

# Process each PR
echo "$PRS" | while read -r pr_data; do
    pr_info=$(echo "$pr_data" | base64 -d)
    number=$(echo "$pr_info" | jq -r '.number')
    title=$(echo "$pr_info" | jq -r '.title')
    head=$(echo "$pr_info" | jq -r '.headRefName')
    mergeable=$(echo "$pr_info" | jq -r '.mergeable')
    
    echo "=== Processing PR #$number: $title ==="
    
    # Check mergeable status
    if [ "$mergeable" != "MERGEABLE" ]; then
        echo "❌ PR #$number is not mergeable (status: $mergeable). Skipping."
        echo ""
        continue
    fi
    
    # Check CI status
    echo "Checking CI status for PR #$number..."
    CHECKS=$(gh pr checks "$number" --repo "$REPO" 2>&1)
    EXIT_CODE=$?
    
    # Parse checks regardless of exit code
    FAILED_CHECKS=$(echo "$CHECKS" | awk 'NR>1 {print $2}' | grep -c "fail")
    TOTAL_ACTIVE_CHECKS=$(echo "$CHECKS" | awk 'NR>1 {print $2}' | grep -v "skipping" | wc -l)
    
    echo "CI Status: $FAILED_CHECKS failed out of $TOTAL_ACTIVE_CHECKS active checks."
    
    if [ "$FAILED_CHECKS" -gt 0 ]; then
        echo "❌ PR #$number has $FAILED_CHECKS failed checks."
        echo "Check details:"
        echo "$CHECKS"
        echo "Skipping merge."
        echo ""
        continue
    fi
    
    if [ "$TOTAL_ACTIVE_CHECKS" -eq 0 ]; then
        echo "⚠️  No active checks found for PR #$number."
        echo "Skipping merge to be safe."
        echo ""
        continue
    fi
    
    echo "✅ All $TOTAL_ACTIVE_CHECKS active checks passed for PR #$number."
    
    # Attempt merge
    echo "Attempting to merge PR #$number..."
    MERGE_OUTPUT=$(gh pr merge "$number" --merge --repo "$REPO" 2>&1)
    MERGE_EXIT=$?
    
    if [ $MERGE_EXIT -eq 0 ]; then
        echo "✅ Successfully merged PR #$number."
        echo "Merge output: $MERGE_OUTPUT"
    else
        echo "❌ Failed to merge PR #$number."
        echo "Error: $MERGE_OUTPUT"
    fi
    
    echo ""
done

echo "=== Merge Report Complete ==="
