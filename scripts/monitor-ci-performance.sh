#!/bin/bash

# CI Performance Monitoring Script
# Monitors CI workflow performance and provides optimization insights

set -e

echo "📊 CI Performance Monitor"
echo "========================"

# Get recent workflow runs
echo "🔍 Analyzing recent workflow runs..."

# Get PR validation runs
echo ""
echo "📋 PR Validation Performance:"
echo "-----------------------------"

# Use gh CLI to get workflow run data
if command -v gh &> /dev/null; then
    # Get recent PR validation runs
    echo "Recent PR validation runs:"
    gh run list --workflow="PR Validation" --limit 5 --json status,conclusion,createdAt,updatedAt | jq -r '.[] | "\(.status) \(.conclusion) \(.createdAt) \(.updatedAt)"' 2>/dev/null || echo "No recent runs found"

    echo ""
    echo "📈 Performance Metrics:"
    echo "----------------------"

    # Calculate average duration for recent runs
    recent_runs=$(gh run list --workflow="PR Validation" --limit 10 --json createdAt,updatedAt 2>/dev/null | jq -r '.[] | "\(.createdAt) \(.updatedAt)"' 2>/dev/null || echo "")

    if [ -n "$recent_runs" ]; then
        echo "Analyzing $(echo "$recent_runs" | wc -l) recent runs..."

        # Calculate average duration (simplified)
        total_duration=0
        count=0

        while IFS= read -r line; do
            if [ -n "$line" ]; then
                created=$(echo "$line" | cut -d' ' -f1)
                updated=$(echo "$line" | cut -d' ' -f2)

                # Simple duration calculation (in seconds)
                created_ts=$(date -d "$created" +%s 2>/dev/null || echo "0")
                updated_ts=$(date -d "$updated" +%s 2>/dev/null || echo "0")

                if [ "$created_ts" != "0" ] && [ "$updated_ts" != "0" ]; then
                    duration=$((updated_ts - created_ts))
                    total_duration=$((total_duration + duration))
                    count=$((count + 1))
                fi
            fi
        done <<< "$recent_runs"

        if [ $count -gt 0 ]; then
            avg_duration=$((total_duration / count))
            echo "• Average PR validation duration: ${avg_duration}s"
            echo "• Total runs analyzed: $count"

            if [ $avg_duration -gt 60 ]; then
                echo "⚠️  WARNING: Average duration is high (>60s)"
                echo "   Consider optimization opportunities"
            elif [ $avg_duration -lt 30 ]; then
                echo "✅ Good performance: Average duration <30s"
            else
                echo "📊 Moderate performance: Average duration 30-60s"
            fi
        fi
    else
        echo "No workflow run data available"
    fi
else
    echo "❌ GitHub CLI not available - install with: brew install gh"
fi

echo ""
echo "💡 Optimization Opportunities:"
echo "------------------------------"

# Check for workflow optimization opportunities
if [ -f ".github/workflows/pr-validation-optimized.yml" ]; then
    echo "✅ Optimized PR validation workflow available"
    echo "   Consider switching from pr-validation.yml to pr-validation-optimized.yml"
else
    echo "⚠️  No optimized workflow found"
    echo "   Run: ./scripts/optimize-ci-workflows.sh"
fi

# Check for caching
if grep -r "actions/cache" .github/workflows/ 2>/dev/null; then
    echo "✅ Caching detected in workflows"
else
    echo "⚠️  No caching found - add caching for better performance"
fi

# Check for change detection
if grep -r "needs.*outputs" .github/workflows/ 2>/dev/null; then
    echo "✅ Change detection detected"
else
    echo "⚠️  No change detection found - add to skip unnecessary jobs"
fi

echo ""
echo "📈 Performance Targets:"
echo "----------------------"
echo "• PR validation: <30 seconds"
echo "• Build time: <5 minutes for full build"
echo "• Cache hit rate: >80%"
echo "• Job skip rate: >50% for small changes"

echo ""
echo "🔧 Quick Optimization Actions:"
echo "------------------------------"
echo "1. Enable the optimized PR validation workflow"
echo "2. Review and optimize cache keys"
echo "3. Add performance monitoring to workflows"
echo "4. Set up alerts for performance regressions"

echo ""
echo "📊 Monitoring Recommendations:"
echo "-----------------------------"
echo "• Track cache hit rates"
echo "• Monitor job skip percentages"
echo "• Set up alerts for >2x performance degradation"
echo "• Review monthly for optimization opportunities"