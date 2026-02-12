# MCP Server Documentation

## Overview

The Tinker MCP (Model Context Protocol) server allows AI agents like Claude to control the browser through a standardized JSON-RPC 2.0 protocol over stdio. This enables powerful browser automation capabilities directly from AI assistants.

## Starting the MCP Server

To start Tinker in MCP server mode:

```bash
cargo run -- --mcp --url https://example.com
```

The MCP server will listen on stdin/stdout for JSON-RPC messages.

## Protocol

The MCP server implements the Model Context Protocol specification, using JSON-RPC 2.0 over stdio.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "navigate",
    "arguments": {
      "url": "https://example.com"
    }
  }
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "Command 'navigate' sent successfully"
    }]
  }
}
```

## Available Methods

### initialize

Initialize the MCP connection.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "my-client",
      "version": "1.0.0"
    }
  }
}
```

### tools/list

List all available browser control tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "navigate",
        "description": "Navigate the browser to a URL",
        "inputSchema": { ... }
      },
      ...
    ]
  }
}
```

### tools/call

Execute a browser control tool.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "navigate",
    "arguments": {
      "url": "https://rust-lang.org"
    }
  }
}
```

## Available Tools

### Navigation & Tab Management

#### navigate
Navigate the browser to a URL.

**Arguments:**
- `url` (string, required): The URL to navigate to

#### create_tab
Create a new browser tab.

**Arguments:**
- `url` (string, required): The URL to open in the new tab

#### close_tab
Close a browser tab.

**Arguments:**
- `id` (number, required): The tab ID to close

#### switch_tab
Switch to a different tab.

**Arguments:**
- `id` (number, required): The tab ID to switch to

### Visual Testing

#### take_screenshot
Capture a screenshot of the current page.

**Arguments:**
- `format` (string, optional): Image format (png, jpeg, webp)
- `quality` (number, optional): Image quality for JPEG (0-100)

#### create_visual_baseline
Create a visual baseline for regression testing.

**Arguments:**
- `test_name` (string, required): Name of the visual test

#### run_visual_test
Run a visual regression test against baseline.

**Arguments:**
- `test_name` (string, required): Name of the visual test
- `tolerance` (number, optional): Tolerance for differences (0.0 to 1.0, default 0.1)

### DOM Inspection & Interaction

#### find_element
Find DOM elements using CSS selector, XPath, or text content.

**Arguments:**
- `selector_type` (string, required): Type of selector (css, xpath, text)
- `selector` (string, required): The selector string

#### click_element
Click on a DOM element.

**Arguments:**
- `selector_type` (string, required): Type of selector (css, xpath, text)
- `selector` (string, required): The selector string

#### type_text
Type text into an input element.

**Arguments:**
- `selector_type` (string, required): Type of selector (css, xpath, text)
- `selector` (string, required): The selector string
- `text` (string, required): The text to type

#### get_page_info
Get information about the current page (title, URL, HTML).

**Arguments:** None

#### execute_javascript
Execute JavaScript code in the page context.

**Arguments:**
- `script` (string, required): JavaScript code to execute

### Network Monitoring

#### start_network_monitoring
Start monitoring network requests and responses.

**Arguments:** None

#### stop_network_monitoring
Stop monitoring network requests.

**Arguments:** None

#### get_network_stats
Get network monitoring statistics.

**Arguments:** None

#### export_network_har
Export network traffic as HAR file.

**Arguments:** None

## Example Usage

### Python Client

```python
import json
import subprocess

# Start the MCP server
process = subprocess.Popen(
    ["cargo", "run", "--", "--mcp", "--url", "https://example.com"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True
)

# Send a request
request = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
        "name": "navigate",
        "arguments": {"url": "https://rust-lang.org"}
    }
}

process.stdin.write(json.dumps(request) + "\n")
process.stdin.flush()

# Read response
response = json.loads(process.stdout.readline())
print(response)
```

### Node.js Client

```javascript
const { spawn } = require('child_process');
const readline = require('readline');

// Start the MCP server
const server = spawn('cargo', ['run', '--', '--mcp', '--url', 'https://example.com']);

const rl = readline.createInterface({
  input: server.stdout,
  output: server.stdin
});

// Send a request
const request = {
  jsonrpc: "2.0",
  id: 1,
  method: "tools/call",
  params: {
    name: "navigate",
    arguments: { url: "https://rust-lang.org" }
  }
};

server.stdin.write(JSON.stringify(request) + '\n');

// Read response
rl.on('line', (line) => {
  const response = JSON.parse(line);
  console.log(response);
});
```

## Integration with Claude Desktop

To use the MCP server with Claude Desktop, add this configuration to your Claude Desktop config file:

**Location:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

**Configuration:**
```json
{
  "mcpServers": {
    "tinker-browser": {
      "command": "cargo",
      "args": ["run", "--", "--mcp", "--url", "https://example.com"],
      "cwd": "/path/to/tinker"
    }
  }
}
```

Then restart Claude Desktop. You can then ask Claude to control the browser:

```
"Can you navigate to rust-lang.org and take a screenshot?"
```

## Testing

Run the included test script to verify the MCP server:

```bash
# Build first
cargo build

# Run tests
python test_mcp_server.py
```

## Combining with API Server

You can run both the MCP server and the REST API server simultaneously:

```bash
cargo run -- --mcp --api --url https://example.com
```

This allows both AI agent control (via MCP) and HTTP API access at the same time.

## Troubleshooting

### Server not responding

Make sure the server started successfully. Check stderr for any error messages:

```python
import subprocess

process = subprocess.Popen(
    ["cargo", "run", "--", "--mcp"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,  # Capture errors
    text=True
)

# Check for errors
errors = process.stderr.readline()
if errors:
    print(f"Error: {errors}")
```

### Invalid JSON-RPC format

Ensure your requests follow the JSON-RPC 2.0 specification:
- Must include `jsonrpc: "2.0"`
- Must include `id` (can be number or string)
- Must include `method`
- Optional `params` object

### Tool execution errors

Tool calls return success/error status. Check the response for error details:

```python
response = json.loads(process.stdout.readline())
if "error" in response:
    print(f"Error: {response['error']['message']}")
```

## Architecture

```
┌──────────────────┐
│   AI Agent       │
│   (Claude)       │
└────────┬─────────┘
         │ JSON-RPC 2.0
         │ over stdio
┌────────▼─────────┐
│   MCP Server     │
│   (This module)  │
└────────┬─────────┘
         │ BrowserCommand
         │ (tokio broadcast)
┌────────▼─────────┐
│ Browser Engine   │
│   (WebView)      │
└──────────────────┘
```

The MCP server:
1. Listens on stdin for JSON-RPC requests
2. Converts tool calls to `BrowserCommand` messages
3. Sends commands via tokio broadcast channel
4. Returns responses on stdout

This architecture allows the MCP server to run in parallel with the REST API server, both controlling the same browser instance.
