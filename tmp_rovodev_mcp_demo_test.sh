#!/bin/bash

# Comprehensive MCP Demo Test Script
# Tests all functionality from mcp_demo.json

echo "🚀 Starting Comprehensive MCP Demo Tests..."
echo "============================================"

MCP_SERVER="/workspaces/ai-orchestrator-hub/backend/target/release/mcp_server"

if [ ! -f "$MCP_SERVER" ]; then
    echo "❌ MCP server binary not found at $MCP_SERVER"
    exit 1
fi

echo "✅ MCP server binary found"

# Test counter
TEST_COUNT=0
PASSED_COUNT=0

run_test() {
    local test_name="$1"
    local json_request="$2"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    echo
    echo "🧪 Test $TEST_COUNT: $test_name"
    echo "Request: $json_request"
    
    response=$(echo "$json_request" | $MCP_SERVER 2>/dev/null)
    
    if echo "$response" | jq -e '.result' > /dev/null 2>&1; then
        echo "✅ PASS - $test_name"
        echo "Response: $response" | jq -C .
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        echo "❌ FAIL - $test_name"
        echo "Response: $response"
    fi
}

# Test 1: Initialize
run_test "Initialize MCP Server" \
'{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"clientInfo": {"name": "demo-client", "version": "1.0.0"}}}'

# Test 2: List Tools
run_test "List Available Tools" \
'{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}'

# Test 3: Create Agent
run_test "Create Swarm Agent" \
'{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "create_swarm_agent", "arguments": {"type": "worker"}}}'

# Test 4: Assign Task
run_test "Assign Swarm Task" \
'{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "assign_swarm_task", "arguments": {"description": "Analyze codebase for security vulnerabilities", "priority": "High"}}}'

# Test 5: Get Status
run_test "Get Swarm Status" \
'{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "get_swarm_status", "arguments": {}}}'

# Test 6: NLP Analysis
run_test "Analyze Text with NLP" \
'{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "analyze_with_nlp", "arguments": {"text": "The multiagent hive system provides intelligent swarm coordination for complex tasks."}}}'

# Test 7: Create Workflow
run_test "Create Specialized Workflow" \
'{"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "create_specialized_workflow", "arguments": {"name": "Security Audit Workflow", "type": "security_audit", "steps": [{"name": "static_analysis", "agent_type": "specialist", "description": "Run static code analysis"}, {"name": "vulnerability_scan", "agent_type": "worker", "description": "Scan for known vulnerabilities"}, {"name": "report_generation", "agent_type": "coordinator", "description": "Generate security report"}], "parallel_execution": false}}}'

# Test 8: Performance Analytics
run_test "Agent Performance Analytics" \
'{"jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": {"name": "agent_performance_analytics", "arguments": {"analysis_type": "comprehensive", "time_range": "24h"}}}'

# Test 9: Dynamic Scaling
run_test "Dynamic Swarm Scaling" \
'{"jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {"name": "dynamic_swarm_scaling", "arguments": {"action": "auto_scale"}}}'

# Test 10: Agent Communication
run_test "Cross Agent Communication" \
'{"jsonrpc": "2.0", "id": 10, "method": "tools/call", "params": {"name": "cross_agent_communication", "arguments": {"action": "broadcast", "from_agent": "coordinator_01", "message": "System maintenance scheduled in 1 hour", "message_type": "info", "channel": "general"}}}'

# Test 11: Knowledge Sharing
run_test "Knowledge Sharing" \
'{"jsonrpc": "2.0", "id": 11, "method": "tools/call", "params": {"name": "knowledge_sharing", "arguments": {"action": "get_insights", "insight_type": "trending", "time_range": "24h"}}}'

# Test 12: System Info
run_test "System Information" \
'{"jsonrpc": "2.0", "id": 12, "method": "tools/call", "params": {"name": "system_info", "arguments": {}}}'

# Summary
echo
echo "============================================"
echo "📊 TEST SUMMARY"
echo "============================================"
echo "Total Tests: $TEST_COUNT"
echo "Passed: $PASSED_COUNT"
echo "Failed: $((TEST_COUNT - PASSED_COUNT))"
echo "Success Rate: $(echo "scale=1; $PASSED_COUNT * 100 / $TEST_COUNT" | bc -l)%"

if [ $PASSED_COUNT -eq $TEST_COUNT ]; then
    echo "🎉 ALL TESTS PASSED!"
    exit 0
else
    echo "⚠️  Some tests failed. Review output above."
    exit 1
fi