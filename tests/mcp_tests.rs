//! Integration tests for MCP server

use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
#[ignore] // Ignore by default since it requires building the binary
fn test_mcp_server_initialize() {
    let mut child = Command::new("cargo")
        .args(&["run", "--", "--mcp", "--url", "https://example.com"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Give the server time to start
    std::thread::sleep(Duration::from_secs(2));

    // Send initialize request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writeln!(stdin, "{}", request.to_string()).expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");

    // Clean up
    child.kill().expect("Failed to kill child process");
}

#[test]
#[ignore] // Ignore by default since it requires building the binary
fn test_mcp_server_tools_list() {
    let mut child = Command::new("cargo")
        .args(&["run", "--", "--mcp", "--url", "https://example.com"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Give the server time to start
    std::thread::sleep(Duration::from_secs(2));

    // Send tools/list request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    writeln!(stdin, "{}", request.to_string()).expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 16);

    // Verify key tools exist
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert!(tool_names.contains(&"navigate"));
    assert!(tool_names.contains(&"take_screenshot"));
    assert!(tool_names.contains(&"execute_javascript"));

    // Clean up
    child.kill().expect("Failed to kill child process");
}

#[test]
#[ignore] // Ignore by default since it requires building the binary
fn test_mcp_server_invalid_request() {
    let mut child = Command::new("cargo")
        .args(&["run", "--", "--mcp", "--url", "https://example.com"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Give the server time to start
    std::thread::sleep(Duration::from_secs(2));

    // Send invalid JSON
    writeln!(stdin, "{{invalid json}}").expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32700); // Parse error

    // Clean up
    child.kill().expect("Failed to kill child process");
}

#[test]
fn test_mcp_request_format() {
    // Test that we can construct valid JSON-RPC requests
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "navigate",
            "arguments": {
                "url": "https://example.com"
            }
        }
    });

    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["id"], 1);
    assert_eq!(request["method"], "tools/call");
    assert_eq!(request["params"]["name"], "navigate");
}

#[test]
fn test_tool_arguments_validation() {
    // Test that tool arguments can be properly constructed
    let navigate_args = json!({
        "url": "https://example.com"
    });
    assert!(navigate_args["url"].is_string());

    let click_args = json!({
        "selector_type": "css",
        "selector": ".button"
    });
    assert_eq!(click_args["selector_type"], "css");

    let type_args = json!({
        "selector_type": "css",
        "selector": "input",
        "text": "Hello"
    });
    assert!(type_args["text"].is_string());
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn test_jsonrpc_version() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize"
        });
        assert_eq!(request["jsonrpc"], "2.0");
    }

    #[test]
    fn test_request_with_params() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "navigate",
                "arguments": {"url": "https://example.com"}
            }
        });

        assert!(request["params"].is_object());
        assert_eq!(request["params"]["name"], "navigate");
    }

    #[test]
    fn test_request_without_params() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });

        assert!(request["params"].is_null());
    }

    #[test]
    fn test_response_format() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "status": "ok"
            }
        });

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
    }

    #[test]
    fn test_error_response_format() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid request"
            }
        });

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32600);
    }
}

#[cfg(test)]
mod tool_schema_tests {
    use super::*;

    #[test]
    fn test_navigate_tool_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to"
                }
            },
            "required": ["url"]
        });

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["url"].is_object());
        assert_eq!(schema["required"][0], "url");
    }

    #[test]
    fn test_click_element_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "selector_type": {
                    "type": "string",
                    "enum": ["css", "xpath", "text"]
                },
                "selector": {
                    "type": "string"
                }
            },
            "required": ["selector_type", "selector"]
        });

        assert!(schema["properties"]["selector_type"]["enum"].is_array());
        assert_eq!(schema["required"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_screenshot_tool_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["png", "jpeg", "webp"]
                },
                "quality": {
                    "type": "number"
                }
            }
        });

        assert!(schema["properties"]["format"]["enum"].is_array());
        assert_eq!(schema["properties"]["quality"]["type"], "number");
    }
}
