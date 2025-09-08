#!/usr/bin/env python3
"""
Comprehensive MCP Implementation Test Suite
Tests all aspects of the AI Orchestrator Hub MCP server implementation
"""

import json
import requests
import subprocess
import time
import sys
import os
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from datetime import datetime

# Test configuration
MCP_HTTP_URL = "http://localhost:3001/api/mcp"
MCP_HEALTH_URL = "http://localhost:3001/api/mcp/health"
TIMEOUT = 10

@dataclass
class TestResult:
    name: str
    passed: bool
    message: str
    duration: float
    details: Optional[Dict[str, Any]] = None

class MCPTestSuite:
    def __init__(self):
        self.results: List[TestResult] = []
        self.start_time = time.time()

    def log(self, message: str, level: str = "INFO"):
        timestamp = datetime.now().strftime("%H:%M:%S")
        print(f"[{timestamp}] [{level}] {message}")

    def run_test(self, test_name: str, test_func):
        """Run a single test and record results"""
        self.log(f"Running test: {test_name}")
        start_time = time.time()

        try:
            result = test_func()
            duration = time.time() - start_time

            if isinstance(result, tuple):
                passed, message, details = result
            else:
                passed, message, details = result, "Test completed", None

            self.results.append(TestResult(test_name, passed, message, duration, details))

            status = "✅ PASS" if passed else "❌ FAIL"
            self.log(f"{status} {test_name}: {message} ({duration:.2f}s)")

        except Exception as e:
            duration = time.time() - start_time
            self.results.append(TestResult(test_name, False, f"Exception: {str(e)}", duration))
            self.log(f"❌ FAIL {test_name}: Exception: {str(e)} ({duration:.2f}s)", "ERROR")

    def mcp_request(self, method: str, params: Optional[Dict] = None) -> Dict:
        """Make an MCP JSON-RPC request"""
        request_data = {
            "jsonrpc": "2.0",
            "id": int(time.time() * 1000),
            "method": method
        }
        if params:
            request_data["params"] = params

        response = requests.post(MCP_HTTP_URL, json=request_data, timeout=TIMEOUT)
        response.raise_for_status()
        return response.json()

    def test_service_health(self):
        """Test MCP service health endpoint"""
        try:
            response = requests.get(MCP_HEALTH_URL, timeout=TIMEOUT)
            if response.status_code != 200:
                return False, f"Health check failed with status {response.status_code}", None

            health_data = response.json()
            expected_fields = ["service", "status", "hive_connected"]

            for field in expected_fields:
                if field not in health_data:
                    return False, f"Missing field in health response: {field}", health_data

            if health_data.get("status") != "healthy":
                return False, f"Service not healthy: {health_data.get('status')}", health_data

            if not health_data.get("hive_connected"):
                return False, "Hive not connected", health_data

            return True, "Health check passed", health_data

        except requests.RequestException as e:
            return False, f"Health check request failed: {str(e)}", None

    def test_mcp_initialize(self):
        """Test MCP server initialization"""
        try:
            response = self.mcp_request("initialize", {
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })

            if "error" in response:
                return False, f"Initialize failed: {response['error']}", response

            result = response.get("result", {})
            expected_fields = ["protocolVersion", "capabilities", "serverInfo"]

            for field in expected_fields:
                if field not in result:
                    return False, f"Missing field in initialize response: {field}", result

            server_info = result.get("serverInfo", {})
            if server_info.get("name") != "multiagent-hive-mcp":
                return False, f"Unexpected server name: {server_info.get('name')}", result

            return True, "Initialize successful", result

        except Exception as e:
            return False, f"Initialize request failed: {str(e)}", None

    def test_tools_list(self):
        """Test listing available MCP tools"""
        try:
            response = self.mcp_request("tools/list")

            if "error" in response:
                return False, f"Tools list failed: {response['error']}", response

            result = response.get("result", {})
            tools = result.get("tools", [])

            if not tools:
                return False, "No tools returned", result

            # Expected tools (should be 12 total)
            expected_tools = {
                "create_swarm_agent", "assign_swarm_task", "get_swarm_status",
                "analyze_with_nlp", "coordinate_agents", "echo", "system_info",
                "list_agents", "list_tasks", "get_agent_details",
                "get_task_details", "batch_create_agents"
            }

            tool_names = {tool.get("name") for tool in tools}
            missing_tools = expected_tools - tool_names

            if missing_tools:
                return False, f"Missing expected tools: {missing_tools}", {"tools": tool_names}

            # Verify tool structure
            for tool in tools:
                required_fields = ["name", "description", "inputSchema"]
                for field in required_fields:
                    if field not in tool:
                        return False, f"Tool {tool.get('name')} missing field: {field}", tool

            return True, f"Found {len(tools)} tools (expected 12)", {"tool_count": len(tools), "tools": tool_names}

        except Exception as e:
            return False, f"Tools list request failed: {str(e)}", None

    def test_echo_tool(self):
        """Test the echo tool"""
        try:
            test_message = "Hello from MCP test suite!"
            response = self.mcp_request("tools/call", {
                "name": "echo",
                "arguments": {"message": test_message}
            })

            if "error" in response:
                return False, f"Echo tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in echo response", result

            text_content = content[0].get("text", "")
            if test_message not in text_content:
                return False, f"Echo response doesn't contain test message", {"response": text_content}

            return True, "Echo tool working correctly", {"response": text_content}

        except Exception as e:
            return False, f"Echo tool request failed: {str(e)}", None

    def test_system_info_tool(self):
        """Test the system_info tool"""
        try:
            response = self.mcp_request("tools/call", {
                "name": "system_info",
                "arguments": {}
            })

            if "error" in response:
                return False, f"System info tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in system info response", result

            text_content = content[0].get("text", "")
            try:
                system_data = json.loads(text_content)
                expected_fields = ["hostname", "platform", "architecture", "cpu_count"]

                for field in expected_fields:
                    if field not in system_data:
                        return False, f"Missing field in system info: {field}", system_data

                return True, "System info tool working correctly", system_data

            except json.JSONDecodeError:
                return False, "System info response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"System info tool request failed: {str(e)}", None

    def test_create_agent_tool(self):
        """Test the create_swarm_agent tool"""
        try:
            response = self.mcp_request("tools/call", {
                "name": "create_swarm_agent",
                "arguments": {"agent_type": "Worker"}
            })

            if "error" in response:
                return False, f"Create agent tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in create agent response", result

            text_content = content[0].get("text", "")
            try:
                agent_data = json.loads(text_content)

                if not agent_data.get("success"):
                    return False, f"Agent creation failed: {agent_data.get('message')}", agent_data

                if "agent_id" not in agent_data:
                    return False, "No agent_id in response", agent_data

                return True, f"Agent created successfully: {agent_data.get('agent_id')}", agent_data

            except json.JSONDecodeError:
                return False, "Create agent response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"Create agent tool request failed: {str(e)}", None

    def test_batch_create_agents_tool(self):
        """Test the batch_create_agents tool"""
        try:
            response = self.mcp_request("tools/call", {
                "name": "batch_create_agents",
                "arguments": {"count": 3, "agent_type": "Worker"}
            })

            if "error" in response:
                return False, f"Batch create agents tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in batch create agents response", result

            text_content = content[0].get("text", "")
            try:
                batch_data = json.loads(text_content)

                if not batch_data.get("success"):
                    return False, f"Batch agent creation failed: {batch_data.get('message')}", batch_data

                created_count = batch_data.get("created_count", 0)
                if created_count != 3:
                    return False, f"Expected 3 agents, got {created_count}", batch_data

                agent_ids = batch_data.get("agent_ids", [])
                if len(agent_ids) != 3:
                    return False, f"Expected 3 agent IDs, got {len(agent_ids)}", batch_data

                return True, f"Batch created {created_count} agents successfully", batch_data

            except json.JSONDecodeError:
                return False, "Batch create agents response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"Batch create agents tool request failed: {str(e)}", None

    def test_get_swarm_status_tool(self):
        """Test the get_swarm_status tool"""
        try:
            response = self.mcp_request("tools/call", {
                "name": "get_swarm_status",
                "arguments": {}
            })

            if "error" in response:
                return False, f"Get swarm status tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in swarm status response", result

            text_content = content[0].get("text", "")
            try:
                status_data = json.loads(text_content)
                expected_fields = ["hive_id", "metrics", "swarm_center", "total_energy"]

                for field in expected_fields:
                    if field not in status_data:
                        return False, f"Missing field in swarm status: {field}", status_data

                metrics = status_data.get("metrics", {})
                if "total_agents" not in metrics:
                    return False, "Missing total_agents in metrics", status_data

                return True, "Swarm status retrieved successfully", status_data

            except json.JSONDecodeError:
                return False, "Swarm status response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"Get swarm status tool request failed: {str(e)}", None

    def test_list_agents_tool(self):
        """Test the list_agents tool"""
        try:
            response = self.mcp_request("tools/call", {
                "name": "list_agents",
                "arguments": {"agent_type": "Worker"}
            })

            if "error" in response:
                return False, f"List agents tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in list agents response", result

            text_content = content[0].get("text", "")
            try:
                agents_data = json.loads(text_content)
                expected_fields = ["total_agents", "active_agents", "filter_applied"]

                for field in expected_fields:
                    if field not in agents_data:
                        return False, f"Missing field in list agents: {field}", agents_data

                return True, f"Listed agents: {agents_data.get('total_agents')} total", agents_data

            except json.JSONDecodeError:
                return False, "List agents response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"List agents tool request failed: {str(e)}", None

    def test_nlp_analysis_tool(self):
        """Test the analyze_with_nlp tool"""
        try:
            test_text = "This is an amazing implementation! The MCP server is working perfectly."
            response = self.mcp_request("tools/call", {
                "name": "analyze_with_nlp",
                "arguments": {"text": test_text}
            })

            if "error" in response:
                return False, f"NLP analysis tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in NLP analysis response", result

            text_content = content[0].get("text", "")
            try:
                nlp_data = json.loads(text_content)

                # Should contain analysis results
                if "analysis" not in nlp_data and "sentiment" not in nlp_data:
                    return False, "No analysis or sentiment in NLP response", nlp_data

                return True, "NLP analysis completed successfully", nlp_data

            except json.JSONDecodeError:
                return False, "NLP analysis response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"NLP analysis tool request failed: {str(e)}", None

    def test_coordinate_agents_tool(self):
        """Test the coordinate_agents tool"""
        try:
            response = self.mcp_request("tools/call", {
                "name": "coordinate_agents",
                "arguments": {"strategy": "balanced"}
            })

            if "error" in response:
                return False, f"Coordinate agents tool failed: {response['error']}", response

            result = response.get("result", {})
            content = result.get("content", [])

            if not content:
                return False, "No content in coordinate agents response", result

            text_content = content[0].get("text", "")
            try:
                coord_data = json.loads(text_content)

                if "success" not in coord_data and "strategy" not in coord_data:
                    return False, "No success or strategy in coordination response", coord_data

                return True, "Agent coordination completed successfully", coord_data

            except json.JSONDecodeError:
                return False, "Coordinate agents response not valid JSON", {"response": text_content}

        except Exception as e:
            return False, f"Coordinate agents tool request failed: {str(e)}", None

    def test_error_handling(self):
        """Test MCP error handling"""
        try:
            # Test invalid method
            response = self.mcp_request("invalid/method")

            if "error" not in response:
                return False, "Expected error for invalid method", response

            error = response["error"]
            if error.get("code") != -32601:  # Method not found
                return False, f"Expected error code -32601, got {error.get('code')}", error

            # Test invalid tool call
            response = self.mcp_request("tools/call", {
                "name": "nonexistent_tool",
                "arguments": {}
            })

            if "error" not in response:
                return False, "Expected error for nonexistent tool", response

            return True, "Error handling working correctly", None

        except Exception as e:
            return False, f"Error handling test failed: {str(e)}", None

    def test_resources_list(self):
        """Test listing MCP resources"""
        try:
            response = self.mcp_request("resources/list")

            if "error" in response:
                return False, f"Resources list failed: {response['error']}", response

            result = response.get("result", {})
            resources = result.get("resources", [])

            # Should have at least the hive status resource
            if not resources:
                return False, "No resources returned", result

            # Check for hive status resource
            hive_resource = None
            for resource in resources:
                if resource.get("uri") == "hive://status":
                    hive_resource = resource
                    break

            if not hive_resource:
                return False, "Missing hive://status resource", {"resources": resources}

            return True, f"Found {len(resources)} resources", {"resources": resources}

        except Exception as e:
            return False, f"Resources list request failed: {str(e)}", None

    def test_resource_read(self):
        """Test reading MCP resources"""
        try:
            response = self.mcp_request("resources/read", {
                "uri": "hive://status"
            })

            if "error" in response:
                return False, f"Resource read failed: {response['error']}", response

            result = response.get("result", {})
            contents = result.get("contents", [])

            if not contents:
                return False, "No contents in resource read response", result

            content = contents[0]
            if content.get("uri") != "hive://status":
                return False, f"Unexpected resource URI: {content.get('uri')}", content

            text_content = content.get("text", "")
            try:
                status_data = json.loads(text_content)
                if "hive_id" not in status_data:
                    return False, "Invalid hive status data", status_data

                return True, "Resource read successful", status_data

            except json.JSONDecodeError:
                return False, "Resource content not valid JSON", {"content": text_content}

        except Exception as e:
            return False, f"Resource read request failed: {str(e)}", None

    def run_all_tests(self):
        """Run the complete test suite"""
        self.log("🚀 Starting MCP Implementation Test Suite")
        self.log("=" * 60)

        # Core MCP protocol tests
        self.run_test("Service Health Check", self.test_service_health)
        self.run_test("MCP Initialize", self.test_mcp_initialize)
        self.run_test("Tools List", self.test_tools_list)
        self.run_test("Resources List", self.test_resources_list)
        self.run_test("Resource Read", self.test_resource_read)
        self.run_test("Error Handling", self.test_error_handling)

        # Tool functionality tests
        self.run_test("Echo Tool", self.test_echo_tool)
        self.run_test("System Info Tool", self.test_system_info_tool)
        self.run_test("Create Agent Tool", self.test_create_agent_tool)
        self.run_test("Batch Create Agents Tool", self.test_batch_create_agents_tool)
        self.run_test("Get Swarm Status Tool", self.test_get_swarm_status_tool)
        self.run_test("List Agents Tool", self.test_list_agents_tool)
        self.run_test("NLP Analysis Tool", self.test_nlp_analysis_tool)
        self.run_test("Coordinate Agents Tool", self.test_coordinate_agents_tool)

        # Generate report
        self.generate_report()

    def generate_report(self):
        """Generate test report"""
        total_time = time.time() - self.start_time
        passed_tests = [r for r in self.results if r.passed]
        failed_tests = [r for r in self.results if not r.passed]

        self.log("=" * 60)
        self.log("📊 TEST SUITE RESULTS")
        self.log("=" * 60)

        self.log(f"Total Tests: {len(self.results)}")
        self.log(f"Passed: {len(passed_tests)} ✅")
        self.log(f"Failed: {len(failed_tests)} ❌")
        self.log(f"Success Rate: {len(passed_tests)/len(self.results)*100:.1f}%")
        self.log(f"Total Time: {total_time:.2f}s")

        if failed_tests:
            self.log("\n❌ FAILED TESTS:")
            for test in failed_tests:
                self.log(f"  - {test.name}: {test.message}")

        if passed_tests:
            self.log(f"\n✅ PASSED TESTS: {len(passed_tests)}")

        # Overall status
        if len(failed_tests) == 0:
            self.log("\n🎉 ALL TESTS PASSED! MCP Implementation is working correctly.")
            return True
        else:
            self.log(f"\n⚠️  {len(failed_tests)} tests failed. Please check the implementation.")
            return False

def check_service_running():
    """Check if MCP service is running"""
    try:
        result = subprocess.run(
            ["./scripts/run-mcp-service.sh", "status"],
            capture_output=True,
            text=True,
            cwd="/workspaces/ai-orchestrator-hub"
        )
        return result.returncode == 0
    except Exception:
        return False

def main():
    """Main test runner"""
    print("🔍 MCP Implementation Test Suite")
    print("=" * 60)

    # Check if service is running
    if not check_service_running():
        print("❌ MCP service is not running. Please start it first:")
        print("   ./scripts/run-mcp-service.sh start")
        sys.exit(1)

    # Wait a moment for service to be ready
    time.sleep(2)

    # Run tests
    test_suite = MCPTestSuite()
    success = test_suite.run_all_tests()

    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
