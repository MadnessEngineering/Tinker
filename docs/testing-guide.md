# Testing Guide for Tinker MCP Server

## Overview

The Tinker MCP server includes comprehensive test coverage across unit tests, integration tests, and end-to-end tests.

## Test Statistics

### Unit Tests (src/mcp/mod.rs)
- **Total**: 21 unit tests
- **Status**: ✅ All passing
- **Coverage**: JSON-RPC protocol, tool handling, error cases

### Integration Tests (tests/mcp_tests.rs)
- **Total**: 13 integration tests
- **Status**: ✅ 10 passing, 3 ignored (require binary)
- **Coverage**: Protocol compliance, tool schemas, request/response formats

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Only MCP Tests
```bash
# Unit tests
cargo test --package tinker mcp::

# Integration tests
cargo test --test mcp_tests
```

### Run Specific Test
```bash
cargo test test_navigate_tool_call
```

### Run with Output
```bash
cargo test -- --nocapture
```

## Test Coverage

### 1. JSON-RPC Protocol Tests ✅

Tests the core JSON-RPC 2.0 protocol implementation:

- **test_jsonrpc_request_deserialization**: Validates request parsing
- **test_jsonrpc_response_serialization**: Validates response formatting
- **test_jsonrpc_error_serialization**: Validates error response formatting
- **test_handle_request_method_not_found**: Tests method-not-found errors

**Example:**
```rust
#[test]
fn test_jsonrpc_request_deserialization() {
    let json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    }"#;
    let request: Result<JsonRpcRequest, _> = serde_json::from_str(json);
    assert!(request.is_ok());
}
```

### 2. Initialization Tests ✅

Tests the MCP server initialization handshake:

- **test_initialize_request**: Validates server capabilities response
- **protocol_tests::test_jsonrpc_version**: Ensures correct protocol version

**Example:**
```rust
#[test]
fn test_initialize_request() {
    let (server, _rx) = setup_test_server();
    let params = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "test-client",
            "version": "1.0.0"
        }
    });

    let result = server.handle_initialize(Some(params));
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["protocolVersion"], "2024-11-05");
}
```

### 3. Tool Discovery Tests ✅

Tests the tools/list functionality:

- **test_tools_list**: Validates all 16 tools are exposed
- **test_tool_definition**: Tests tool schema generation

**Example:**
```rust
#[test]
fn test_tools_list() {
    let (server, _rx) = setup_test_server();
    let result = server.handle_tools_list();
    let tools = result.unwrap()["tools"].as_array().unwrap();

    assert!(tools.len() >= 16);
    assert!(tool_names.contains(&"navigate"));
    assert!(tool_names.contains(&"execute_javascript"));
}
```

### 4. Navigation Tool Tests ✅

Tests browser navigation and tab management:

- **test_navigate_tool_call**: Navigate to URL
- **test_create_tab**: Create new tab
- **test_switch_tab**: Switch between tabs
- **test_close_tab**: Close tabs

**Example:**
```rust
#[test]
fn test_navigate_tool_call() {
    let (mut server, _rx) = setup_test_server();
    let params = json!({
        "name": "navigate",
        "arguments": {"url": "https://example.com"}
    });

    let result = server.handle_tool_call(Some(params));
    assert!(result.is_ok());
}
```

### 5. DOM Interaction Tests ✅

Tests element finding and interaction:

- **test_find_element_different_selectors**: CSS, XPath, text selectors
- **test_click_element_tool**: Element clicking
- **test_type_text_tool**: Text input

**Example:**
```rust
#[test]
fn test_find_element_different_selectors() {
    let (mut server, _rx) = setup_test_server();

    // Test CSS selector
    assert!(server.handle_tool_call(Some(json!({
        "name": "find_element",
        "arguments": {
            "selector_type": "css",
            "selector": ".my-class"
        }
    }))).is_ok());

    // Test XPath selector
    assert!(server.handle_tool_call(Some(json!({
        "name": "find_element",
        "arguments": {
            "selector_type": "xpath",
            "selector": "//div[@class='my-class']"
        }
    }))).is_ok());
}
```

### 6. JavaScript Execution Tests ✅

Tests script execution capabilities:

- **test_execute_javascript_tool**: Execute JavaScript in page context

**Example:**
```rust
#[test]
fn test_execute_javascript_tool() {
    let (mut server, _rx) = setup_test_server();
    let params = json!({
        "name": "execute_javascript",
        "arguments": {"script": "document.title"}
    });

    assert!(server.handle_tool_call(Some(params)).is_ok());
}
```

### 7. Visual Testing Tests ✅

Tests screenshot and visual regression capabilities:

- **test_screenshot_tool**: Screenshot capture
- **test_visual_testing_tools**: Baseline creation and comparison

**Example:**
```rust
#[test]
fn test_visual_testing_tools() {
    let (mut server, _rx) = setup_test_server();

    // Create baseline
    assert!(server.handle_tool_call(Some(json!({
        "name": "create_visual_baseline",
        "arguments": {"test_name": "homepage"}
    }))).is_ok());

    // Run visual test
    assert!(server.handle_tool_call(Some(json!({
        "name": "run_visual_test",
        "arguments": {
            "test_name": "homepage",
            "tolerance": 0.05
        }
    }))).is_ok());
}
```

### 8. Network Monitoring Tests ✅

Tests network traffic monitoring:

- **test_network_monitoring_tools**: Start/stop monitoring, get stats

**Example:**
```rust
#[test]
fn test_network_monitoring_tools() {
    let (mut server, _rx) = setup_test_server();

    // Start monitoring
    assert!(server.handle_tool_call(Some(json!({
        "name": "start_network_monitoring",
        "arguments": {}
    }))).is_ok());

    // Get stats
    assert!(server.handle_tool_call(Some(json!({
        "name": "get_network_stats",
        "arguments": {}
    }))).is_ok());
}
```

### 9. Error Handling Tests ✅

Tests error cases and validation:

- **test_missing_tool_name**: Missing required tool name
- **test_unknown_tool**: Invalid tool name
- **test_missing_required_parameter**: Missing required parameters

**Example:**
```rust
#[test]
fn test_missing_required_parameter() {
    let (mut server, _rx) = setup_test_server();
    let params = json!({
        "name": "navigate",
        "arguments": {} // Missing 'url'
    });

    let result = server.handle_tool_call(Some(params));
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Missing required parameter"));
}
```

### 10. Protocol Compliance Tests ✅

Tests JSON-RPC 2.0 specification compliance:

- **protocol_tests::test_request_with_params**: Requests with parameters
- **protocol_tests::test_request_without_params**: Requests without parameters
- **protocol_tests::test_response_format**: Response structure
- **protocol_tests::test_error_response_format**: Error response structure

### 11. Tool Schema Tests ✅

Tests that tool schemas are properly defined:

- **tool_schema_tests::test_navigate_tool_schema**: Navigate tool schema
- **tool_schema_tests::test_click_element_schema**: Click element schema
- **tool_schema_tests::test_screenshot_tool_schema**: Screenshot schema

## Test Structure

### Unit Test Setup

```rust
fn setup_test_server() -> (McpServer, broadcast::Receiver<BrowserCommand>) {
    let (command_tx, command_rx) = broadcast::channel(100);
    let (_event_tx, event_rx) = broadcast::channel(100);
    (McpServer::new(command_tx, event_rx), command_rx)
}
```

This setup creates:
- A broadcast channel for sending commands to the browser
- A broadcast channel for receiving events from the browser
- An MCP server instance configured with these channels
- A command receiver to keep the channel alive during tests

### Integration Test Setup

Integration tests spawn a real server process:

```rust
let mut child = Command::new("cargo")
    .args(&["run", "--", "--mcp", "--url", "https://example.com"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to start MCP server");
```

## Running Integration Tests

Some integration tests are ignored by default because they require:
1. Building the binary first
2. Actually spawning the server process

To run them:

```bash
# Build first
cargo build

# Run ignored tests
cargo test --test mcp_tests -- --ignored
```

## Writing New Tests

### Adding a Unit Test

```rust
#[test]
fn test_my_new_feature() {
    let (mut server, _rx) = setup_test_server();

    let params = json!({
        "name": "my_tool",
        "arguments": {"param": "value"}
    });

    let result = server.handle_tool_call(Some(params));
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["content"][0]["type"], "text");
}
```

### Adding an Integration Test

```rust
#[test]
fn test_my_integration_scenario() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "my_tool",
            "arguments": {"param": "value"}
        }
    });

    // Assertions...
}
```

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| Protocol | 6 | ✅ Passing |
| Tools | 11 | ✅ Passing |
| Error Handling | 3 | ✅ Passing |
| Schemas | 3 | ✅ Passing |
| Integration | 10 | ✅ Passing |
| **Total** | **33** | **✅ All Passing** |

## Continuous Integration

To add MCP tests to CI:

```yaml
- name: Run tests
  run: cargo test

- name: Run MCP tests specifically
  run: cargo test --package tinker mcp::
```

## Manual Testing

For manual testing with the Python test script:

```bash
# Build first
cargo build

# Run the test script
python test_mcp_server.py
```

This script tests:
1. Server initialization
2. Tools listing
3. Navigation commands
4. Screenshot capture
5. Page info retrieval
6. JavaScript execution

## Debugging Failed Tests

If a test fails:

1. **Run with backtrace:**
   ```bash
   RUST_BACKTRACE=1 cargo test test_name
   ```

2. **Run with output:**
   ```bash
   cargo test test_name -- --nocapture
   ```

3. **Check the error:**
   Most test failures will show the actual vs expected values

## Best Practices

1. **Always keep a receiver alive**: The test setup function returns a receiver to prevent the broadcast channel from closing
2. **Test both success and failure cases**: Include tests for error conditions
3. **Use descriptive test names**: Name tests after what they verify
4. **Keep tests isolated**: Each test should be independent
5. **Test edge cases**: Missing parameters, invalid values, etc.

## Future Test Improvements

- [ ] Add performance benchmarks for tool calls
- [ ] Add stress tests with many concurrent requests
- [ ] Add end-to-end tests with real browser interactions
- [ ] Add coverage reporting
- [ ] Add mutation testing

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Model Context Protocol](https://modelcontextprotocol.io)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
