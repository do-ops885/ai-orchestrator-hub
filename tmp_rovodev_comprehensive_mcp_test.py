#!/usr/bin/env python3
"""
Comprehensive MCP Testing Suite for AI Orchestrator Hub
Tests all available MCP tools and functionality
"""

import json
import subprocess
import sys
import time
from typing import Dict, Any, List, Optional

class MCPTester:
    def __init__(self, binary_path: str = "/workspaces/ai-orchestrator-hub/backend/target/debug/mcp_server"):
        self.binary_path = binary_path
        self.test_results = []

    def run_mcp_command(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Run a single MCP command and return the response"""
        try:
            request_json = json.dumps(request)
            # Use the same approach as the working bash script
            result = subprocess.run(
                f"echo '{request_json}' | {self.binary_path}",
                shell=True,
                text=True,
                capture_output=True,
                timeout=30
            )

            if result.stdout.strip():
                # The output might have multiple lines, get the last JSON line
                lines = result.stdout.strip().split('\n')
                for line in reversed(lines):
                    line = line.strip()
                    if line.startswith('{') and 'jsonrpc' in line:
                        return json.loads(line)
                return {"error": f"No valid JSON response found in output: {result.stdout}"}
            else:
                return {"error": f"No output. stderr: {result.stderr}"}

        except subprocess.TimeoutExpired:
            return {"error": "Command timed out"}
        except json.JSONDecodeError as e:
            return {"error": f"JSON decode error: {e}"}
        except Exception as e:
            return {"error": f"Unexpected error: {e}"}
    
    def test_initialize(self) -> bool:
        """Test MCP initialization"""
        print("Testing MCP initialization...")
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "clientInfo": {
                    "name": "comprehensive-test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "serverInfo" in response["result"]
        
        self.test_results.append({
            "test": "initialize",
            "success": success,
            "response": response
        })
        
        print(f"✅ Initialize: {'PASS' if success else 'FAIL'}")
        return success
    
    def test_list_tools(self) -> bool:
        """Test listing available tools"""
        print("Testing tool listing...")
        request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "tools" in response["result"]
        
        if success:
            tools = response["result"]["tools"]
            print(f"Found {len(tools)} available tools:")
            for tool in tools:
                print(f"  - {tool['name']}: {tool.get('description', 'No description')}")
        
        self.test_results.append({
            "test": "list_tools",
            "success": success,
            "response": response
        })
        
        print(f"✅ List Tools: {'PASS' if success else 'FAIL'}")
        return success
    
    def test_system_info(self) -> bool:
        """Test system info tool"""
        print("Testing system info tool...")
        request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "system_info",
                "arguments": {}
            }
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "content" in response["result"]
        
        self.test_results.append({
            "test": "system_info",
            "success": success,
            "response": response
        })
        
        print(f"✅ System Info: {'PASS' if success else 'FAIL'}")
        return success
    
    def test_create_agent(self) -> Optional[str]:
        """Test creating a swarm agent"""
        print("Testing agent creation...")
        request = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "create_swarm_agent",
                "arguments": {
                    "type": "worker"
                }
            }
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "content" in response["result"]
        agent_id = None
        
        if success:
            try:
                content = json.loads(response["result"]["content"][0]["text"])
                agent_id = content.get("agent_id")
                success = content.get("success", False)
            except:
                success = False
        
        self.test_results.append({
            "test": "create_agent",
            "success": success,
            "response": response,
            "agent_id": agent_id
        })
        
        print(f"✅ Create Agent: {'PASS' if success else 'FAIL'}")
        if agent_id:
            print(f"   Created agent: {agent_id}")
        return agent_id if success else None
    
    def test_assign_task(self) -> bool:
        """Test task assignment"""
        print("Testing task assignment...")
        request = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "assign_swarm_task",
                "arguments": {
                    "description": "Comprehensive test task for MCP validation",
                    "priority": "High"
                }
            }
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "content" in response["result"]
        
        self.test_results.append({
            "test": "assign_task",
            "success": success,
            "response": response
        })
        
        print(f"✅ Assign Task: {'PASS' if success else 'FAIL'}")
        return success
    
    def test_nlp_analysis(self) -> bool:
        """Test NLP analysis tool"""
        print("Testing NLP analysis...")
        request = {
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "analyze_with_nlp",
                "arguments": {
                    "text": "The multiagent hive system provides intelligent swarm coordination for complex distributed computing tasks with advanced neural network processing."
                }
            }
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "content" in response["result"]
        
        self.test_results.append({
            "test": "nlp_analysis",
            "success": success,
            "response": response
        })
        
        print(f"✅ NLP Analysis: {'PASS' if success else 'FAIL'}")
        return success
    
    def test_swarm_status(self) -> bool:
        """Test getting swarm status"""
        print("Testing swarm status...")
        request = {
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "get_swarm_status",
                "arguments": {}
            }
        }
        
        response = self.run_mcp_command(request)
        success = "result" in response and "content" in response["result"]
        
        self.test_results.append({
            "test": "swarm_status",
            "success": success,
            "response": response
        })
        
        print(f"✅ Swarm Status: {'PASS' if success else 'FAIL'}")
        return success
    
    def test_advanced_tools(self) -> None:
        """Test advanced MCP tools"""
        advanced_tests = [
            {
                "name": "create_specialized_workflow",
                "args": {
                    "name": "MCP Test Workflow",
                    "type": "testing",  # Use valid workflow type
                    "steps": [
                        {"name": "setup", "agent_type": "coordinator", "description": "Setup test environment"},
                        {"name": "execute", "agent_type": "worker", "description": "Execute test cases"},
                        {"name": "cleanup", "agent_type": "worker", "description": "Clean up test data"}
                    ],
                    "parallel_execution": False
                }
            },
            {
                "name": "agent_performance_analytics",
                "args": {
                    "analysis_type": "comprehensive",
                    "time_range": "1h"
                }
            },
            {
                "name": "dynamic_swarm_scaling",
                "args": {
                    "action": "auto_scale"
                }
            },
            {
                "name": "cross_agent_communication",
                "args": {
                    "action": "broadcast",
                    "from_agent": "test_coordinator",
                    "message": "MCP comprehensive test in progress",
                    "message_type": "info",
                    "channel": "test"
                }
            },
            {
                "name": "knowledge_sharing",
                "args": {
                    "action": "get_insights",
                    "insight_type": "trending",
                    "time_range": "1h"
                }
            }
        ]
        
        for i, test in enumerate(advanced_tests, 8):
            print(f"Testing {test['name']}...")
            request = {
                "jsonrpc": "2.0",
                "id": i,
                "method": "tools/call",
                "params": {
                    "name": test["name"],
                    "arguments": test["args"]
                }
            }
            
            response = self.run_mcp_command(request)
            success = "result" in response
            
            self.test_results.append({
                "test": test["name"],
                "success": success,
                "response": response
            })
            
            print(f"✅ {test['name']}: {'PASS' if success else 'FAIL'}")
    
    def run_comprehensive_test(self) -> None:
        """Run the complete test suite"""
        print("=" * 60)
        print("🚀 Starting Comprehensive MCP Test Suite")
        print("=" * 60)
        
        # Core functionality tests
        self.test_initialize()
        self.test_list_tools()
        self.test_system_info()
        
        # Agent and task management
        agent_id = self.test_create_agent()
        self.test_assign_task()
        
        # Analysis and status
        self.test_nlp_analysis()
        self.test_swarm_status()
        
        # Advanced functionality
        self.test_advanced_tools()
        
        # Generate summary
        self.print_summary()
    
    def print_summary(self) -> None:
        """Print test results summary"""
        print("\n" + "=" * 60)
        print("📊 TEST RESULTS SUMMARY")
        print("=" * 60)
        
        total_tests = len(self.test_results)
        passed_tests = sum(1 for result in self.test_results if result["success"])
        failed_tests = total_tests - passed_tests
        
        print(f"Total Tests: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {failed_tests}")
        print(f"Success Rate: {(passed_tests/total_tests)*100:.1f}%")
        
        if failed_tests > 0:
            print("\n❌ FAILED TESTS:")
            for result in self.test_results:
                if not result["success"]:
                    print(f"  - {result['test']}: {result.get('response', {}).get('error', 'Unknown error')}")
        
        print("\n✅ PASSED TESTS:")
        for result in self.test_results:
            if result["success"]:
                print(f"  - {result['test']}")
        
        # Save detailed results
        with open("tmp_rovodev_mcp_test_results.json", "w") as f:
            json.dump(self.test_results, f, indent=2)
        print(f"\n📄 Detailed results saved to: tmp_rovodev_mcp_test_results.json")

if __name__ == "__main__":
    tester = MCPTester()
    tester.run_comprehensive_test()