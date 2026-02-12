#!/usr/bin/env python3
"""
Test script for Tinker MCP server

This script demonstrates how to interact with the Tinker browser through
the Model Context Protocol (MCP) server.

The MCP server implements JSON-RPC 2.0 over stdio, allowing AI agents
like Claude to control the browser.
"""

import json
import subprocess
import sys
import time

class McpClient:
    """Simple MCP client for testing"""

    def __init__(self, server_path):
        """Initialize the MCP client by starting the server process"""
        self.process = subprocess.Popen(
            [server_path, "--mcp", "--url", "https://example.com"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        # Give the server a moment to start
        time.sleep(2)

    def send_request(self, method, params=None, request_id=1):
        """Send a JSON-RPC request to the MCP server"""
        request = {
            "jsonrpc": "2.0",
            "method": method,
            "id": request_id
        }
        if params is not None:
            request["params"] = params

        request_json = json.dumps(request)
        print(f"\n→ Sending: {request_json}")

        self.process.stdin.write(request_json + "\n")
        self.process.stdin.flush()

        # Read response
        response_line = self.process.stdout.readline()
        if response_line:
            print(f"← Received: {response_line.strip()}")
            return json.loads(response_line)
        return None

    def close(self):
        """Close the MCP client and terminate the server"""
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=5)

def main():
    """Run MCP server tests"""
    print("=" * 70)
    print("Tinker MCP Server Test")
    print("=" * 70)

    # Determine server path
    server_path = "./target/debug/tinker.exe" if sys.platform == "win32" else "./target/debug/tinker"

    print(f"\nStarting MCP server: {server_path}")
    print("Note: Make sure you've built the project with 'cargo build' first!")

    try:
        client = McpClient(server_path)

        # Test 1: Initialize
        print("\n" + "=" * 70)
        print("TEST 1: Initialize MCP server")
        print("=" * 70)
        response = client.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })

        if response and "result" in response:
            print("✅ Initialize successful!")
            print(f"   Server: {response['result'].get('serverInfo', {}).get('name')}")
            print(f"   Version: {response['result'].get('serverInfo', {}).get('version')}")
        else:
            print("❌ Initialize failed!")

        # Test 2: List tools
        print("\n" + "=" * 70)
        print("TEST 2: List available tools")
        print("=" * 70)
        response = client.send_request("tools/list", {})

        if response and "result" in response:
            tools = response['result'].get('tools', [])
            print(f"✅ Found {len(tools)} tools:")
            for tool in tools[:5]:  # Show first 5
                print(f"   - {tool['name']}: {tool['description']}")
            if len(tools) > 5:
                print(f"   ... and {len(tools) - 5} more")
        else:
            print("❌ List tools failed!")

        # Test 3: Navigate to a URL
        print("\n" + "=" * 70)
        print("TEST 3: Navigate to URL")
        print("=" * 70)
        response = client.send_request("tools/call", {
            "name": "navigate",
            "arguments": {
                "url": "https://www.rust-lang.org"
            }
        }, request_id=3)

        if response and "result" in response:
            print("✅ Navigate command sent!")
            print(f"   Response: {response['result']}")
        else:
            print("❌ Navigate failed!")

        # Test 4: Take a screenshot
        print("\n" + "=" * 70)
        print("TEST 4: Take screenshot")
        print("=" * 70)
        response = client.send_request("tools/call", {
            "name": "take_screenshot",
            "arguments": {
                "format": "png"
            }
        }, request_id=4)

        if response and "result" in response:
            print("✅ Screenshot command sent!")
            print(f"   Response: {response['result']}")
        else:
            print("❌ Screenshot failed!")

        # Test 5: Get page info
        print("\n" + "=" * 70)
        print("TEST 5: Get page info")
        print("=" * 70)
        response = client.send_request("tools/call", {
            "name": "get_page_info",
            "arguments": {}
        }, request_id=5)

        if response and "result" in response:
            print("✅ Get page info command sent!")
            print(f"   Response: {response['result']}")
        else:
            print("❌ Get page info failed!")

        # Test 6: Execute JavaScript
        print("\n" + "=" * 70)
        print("TEST 6: Execute JavaScript")
        print("=" * 70)
        response = client.send_request("tools/call", {
            "name": "execute_javascript",
            "arguments": {
                "script": "document.title"
            }
        }, request_id=6)

        if response and "result" in response:
            print("✅ Execute JavaScript command sent!")
            print(f"   Response: {response['result']}")
        else:
            print("❌ Execute JavaScript failed!")

        print("\n" + "=" * 70)
        print("All tests completed!")
        print("=" * 70)

        # Give commands time to execute
        print("\nWaiting 5 seconds for commands to process...")
        time.sleep(5)

    except FileNotFoundError:
        print(f"\n❌ Error: Server executable not found at {server_path}")
        print("   Please run 'cargo build' first!")
        return 1
    except KeyboardInterrupt:
        print("\n\nTest interrupted by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    finally:
        print("\nClosing MCP client...")
        client.close()

    return 0

if __name__ == "__main__":
    sys.exit(main())
