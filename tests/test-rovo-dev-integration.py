#!/usr/bin/env python3
"""
Rovo Dev MCP Integration Test Suite
Tests the integration between Rovo Dev CLI and the AI Orchestrator Hub MCP server
"""

import json
import subprocess
import time
import sys
import os
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass

@dataclass
class IntegrationTestResult:
    name: str
    passed: bool
    message: str
    duration: float
    output: Optional[str] = None

class RovoDevIntegrationTests:
    def __init__(self):
        self.results: List[IntegrationTestResult] = []
        self.mcp_config_path = Path("/home/codespace/.rovodev/mcp.json")

    def log(self, message: str, level: str = "INFO"):
        timestamp = time.strftime("%H:%M:%S")
        print(f"[{timestamp}] [{level}] {message}")

    def run_test(self, test_name: str, test_func):
        """Run a single integration test"""
        self.log(f"Running integration test: {test_name}")
        start_time = time.time()

        try:
            result = test_func()
            duration = time.time() - start_time

            if isinstance(result, tuple):
                passed, message, output = result
            else:
                passed, message, output = result, "Test completed", None

            self.results.append(IntegrationTestResult(test_name, passed, message, duration, output))

            status = "✅ PASS" if passed else "❌ FAIL"
            self.log(f"{status} {test_name}: {message} ({duration:.2f}s)")

        except Exception as e:
            duration = time.time() - start_time
            self.results.append(IntegrationTestResult(test_name, False, f"Exception: {str(e)}", duration))
            self.log(f"❌ FAIL {test_name}: Exception: {str(e)} ({duration:.2f}s)", "ERROR")

    def test_mcp_config_exists(self):
        """Test that MCP configuration file exists and is valid"""
        if not self.mcp_config_path.exists():
            return False, "MCP config file does not exist", None

        try:
            with open(self.mcp_config_path, 'r') as f:
                config = json.load(f)

            if "mcpServers" not in config:
                return False, "Missing mcpServers in config", str(config)

            if "multiagent-hive" not in config["mcpServers"]:
                return False, "Missing multiagent-hive server config", str(config)

            hive_config = config["mcpServers"]["multiagent-hive"]
            required_fields = ["command", "args", "cwd"]

            for field in required_fields:
                if field not in hive_config:
                    return False, f"Missing {field} in hive config", str(hive_config)

            # Check if command path exists
            command_path = Path(hive_config["command"])
            if not command_path.exists():
                return False, f"MCP server binary not found: {command_path}", None

            return True, "MCP config is valid", str(config)

        except json.JSONDecodeError as e:
            return False, f"Invalid JSON in config file: {str(e)}", None
        except Exception as e:
            return False, f"Error reading config file: {str(e)}", None

    def test_rovo_dev_cli_available(self):
        """Test that Rovo Dev CLI is available"""
        try:
            result = subprocess.run(
                ["acli", "rovodev", "--help"],
                capture_output=True,
                text=True,
                timeout=10
            )

            if result.returncode != 0:
                return False, f"Rovo Dev CLI not working: {result.stderr}", result.stdout

            if "Atlassian Rovo Dev CLI" not in result.stdout:
                return False, "Unexpected Rovo Dev CLI output", result.stdout

            return True, "Rovo Dev CLI is available", result.stdout[:200]

        except subprocess.TimeoutExpired:
            return False, "Rovo Dev CLI command timed out", None
        except FileNotFoundError:
            return False, "Rovo Dev CLI not found (acli command not available)", None
        except Exception as e:
            return False, f"Error testing Rovo Dev CLI: {str(e)}", None

    def test_mcp_server_binary_exists(self):
        """Test that the MCP server binary exists and is executable"""
        binary_path = Path("/workspaces/ai-orchestrator-hub/backend/target/release/mcp_server")

        if not binary_path.exists():
            return False, f"MCP server binary not found: {binary_path}", None

        if not os.access(binary_path, os.X_OK):
            return False, f"MCP server binary not executable: {binary_path}", None

        # Test that it can start (briefly)
        try:
            proc = subprocess.Popen(
                [str(binary_path)],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                cwd="/workspaces/ai-orchestrator-hub"
            )

            # Send a simple test request
            test_request = json.dumps({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {"clientInfo": {"name": "test", "version": "1.0"}}
            }) + "\n"

            try:
                stdout, stderr = proc.communicate(input=test_request.encode(), timeout=5)
                proc.terminate()

                if proc.returncode is None:
                    proc.kill()

                # Check if we got a valid response
                if stdout:
                    try:
                        response = json.loads(stdout.decode().strip())
                        if "result" in response or "error" in response:
                            return True, "MCP server binary is working", response
                    except json.JSONDecodeError:
                        pass

                return True, "MCP server binary starts but response unclear", stdout.decode()[:200]

            except subprocess.TimeoutExpired:
                proc.kill()
                return False, "MCP server binary timed out", None

        except Exception as e:
            return False, f"Error testing MCP server binary: {str(e)}", None

    def test_service_script_functionality(self):
        """Test the MCP service management script"""
        script_path = Path("/workspaces/ai-orchestrator-hub/scripts/run-mcp-service.sh")

        if not script_path.exists():
            return False, "Service script not found", None

        try:
            # Test status command
            result = subprocess.run(
                ["./scripts/run-mcp-service.sh", "status"],
                capture_output=True,
                text=True,
                timeout=10,
                cwd="/workspaces/ai-orchestrator-hub"
            )

            # Should return 0 if running, non-zero if not running
            if "Service is running" in result.stdout:
                service_running = True
            elif "Service is not running" in result.stdout:
                service_running = False
            else:
                return False, f"Unexpected status output: {result.stdout}", result.stdout

            return True, f"Service script working (service running: {service_running})", result.stdout

        except subprocess.TimeoutExpired:
            return False, "Service script timed out", None
        except Exception as e:
            return False, f"Error testing service script: {str(e)}", None

    def test_workspace_structure(self):
        """Test that the workspace has the expected structure"""
        required_paths = [
            "/workspaces/ai-orchestrator-hub",
            "/workspaces/ai-orchestrator-hub/backend",
            "/workspaces/ai-orchestrator-hub/backend/src",
            "/workspaces/ai-orchestrator-hub/backend/src/communication",
            "/workspaces/ai-orchestrator-hub/backend/src/communication/mcp.rs",
            "/workspaces/ai-orchestrator-hub/backend/Cargo.toml",
            "/workspaces/ai-orchestrator-hub/scripts",
            "/workspaces/ai-orchestrator-hub/scripts/run-mcp-service.sh"
        ]

        missing_paths = []
        for path in required_paths:
            if not Path(path).exists():
                missing_paths.append(path)

        if missing_paths:
            return False, f"Missing required paths: {missing_paths}", None

        return True, "Workspace structure is correct", None

    def test_environment_variables(self):
        """Test that required environment variables are set"""
        # Check if EDITOR is set (needed for Rovo Dev config editing)
        editor = os.environ.get("EDITOR")
        if not editor:
            return False, "EDITOR environment variable not set", None

        # Check if PATH includes necessary directories
        path = os.environ.get("PATH", "")
        if "/home/codespace/.local/bin" not in path:
            return False, "Rovo Dev CLI path not in PATH", path

        return True, f"Environment variables are set correctly (EDITOR: {editor})", None

    def test_rovo_dev_config_structure(self):
        """Test the overall Rovo Dev configuration structure"""
        config_dir = Path("/home/codespace/.rovodev")

        if not config_dir.exists():
            return False, "Rovo Dev config directory does not exist", None

        expected_files = ["mcp.json", "config.yml"]
        existing_files = []
        missing_files = []

        for file in expected_files:
            file_path = config_dir / file
            if file_path.exists():
                existing_files.append(file)
            else:
                missing_files.append(file)

        if missing_files:
            return False, f"Missing config files: {missing_files}", f"Existing: {existing_files}"

        return True, f"Rovo Dev config structure is complete", f"Files: {existing_files}"

    def test_mcp_server_compilation(self):
        """Test that the MCP server can be compiled"""
        try:
            result = subprocess.run(
                ["cargo", "check", "--bin", "mcp_server"],
                capture_output=True,
                text=True,
                timeout=60,
                cwd="/workspaces/ai-orchestrator-hub/backend"
            )

            if result.returncode != 0:
                return False, f"MCP server compilation failed: {result.stderr}", result.stdout

            return True, "MCP server compiles successfully", result.stdout[:200]

        except subprocess.TimeoutExpired:
            return False, "MCP server compilation timed out", None
        except Exception as e:
            return False, f"Error testing MCP server compilation: {str(e)}", None

    def test_documentation_files(self):
        """Test that documentation files exist"""
        doc_files = [
            "/workspaces/ai-orchestrator-hub/ROVO_DEV_INTEGRATION.md",
            "/workspaces/ai-orchestrator-hub/advanced-rovo-dev-examples.md",
            "/workspaces/ai-orchestrator-hub/MCP_SERVICE_README.md"
        ]

        existing_docs = []
        missing_docs = []

        for doc_file in doc_files:
            if Path(doc_file).exists():
                existing_docs.append(Path(doc_file).name)
            else:
                missing_docs.append(Path(doc_file).name)

        if missing_docs:
            return False, f"Missing documentation files: {missing_docs}", f"Existing: {existing_docs}"

        return True, f"Documentation files are present", f"Files: {existing_docs}"

    def run_all_tests(self):
        """Run all integration tests"""
        self.log("🔗 Starting Rovo Dev MCP Integration Test Suite")
        self.log("=" * 60)

        # Configuration and setup tests
        self.run_test("MCP Config File", self.test_mcp_config_exists)
        self.run_test("Rovo Dev CLI Available", self.test_rovo_dev_cli_available)
        self.run_test("Environment Variables", self.test_environment_variables)
        self.run_test("Rovo Dev Config Structure", self.test_rovo_dev_config_structure)

        # Binary and compilation tests
        self.run_test("MCP Server Binary", self.test_mcp_server_binary_exists)
        self.run_test("MCP Server Compilation", self.test_mcp_server_compilation)

        # Infrastructure tests
        self.run_test("Workspace Structure", self.test_workspace_structure)
        self.run_test("Service Script", self.test_service_script_functionality)
        self.run_test("Documentation Files", self.test_documentation_files)

        # Generate report
        self.generate_report()

    def generate_report(self):
        """Generate integration test report"""
        passed_tests = [r for r in self.results if r.passed]
        failed_tests = [r for r in self.results if not r.passed]

        self.log("=" * 60)
        self.log("📊 INTEGRATION TEST RESULTS")
        self.log("=" * 60)

        self.log(f"Total Tests: {len(self.results)}")
        self.log(f"Passed: {len(passed_tests)} ✅")
        self.log(f"Failed: {len(failed_tests)} ❌")
        self.log(f"Success Rate: {len(passed_tests)/len(self.results)*100:.1f}%")

        if failed_tests:
            self.log("\n❌ FAILED INTEGRATION TESTS:")
            for test in failed_tests:
                self.log(f"  - {test.name}: {test.message}")
                if test.output:
                    self.log(f"    Output: {test.output[:100]}...")

        if passed_tests:
            self.log(f"\n✅ PASSED INTEGRATION TESTS: {len(passed_tests)}")

        # Overall status
        if len(failed_tests) == 0:
            self.log("\n🎉 ALL INTEGRATION TESTS PASSED! Rovo Dev MCP integration is ready.")
            return True
        else:
            self.log(f"\n⚠️  {len(failed_tests)} integration tests failed. Please check the setup.")
            return False

def main():
    """Main integration test runner"""
    print("🔗 Rovo Dev MCP Integration Test Suite")
    print("=" * 60)

    # Run integration tests
    test_suite = RovoDevIntegrationTests()
    success = test_suite.run_all_tests()

    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
