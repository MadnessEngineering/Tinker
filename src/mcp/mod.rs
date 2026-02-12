//! Model Context Protocol (MCP) server for browser control
//!
//! This module implements an MCP server that exposes browser automation capabilities
//! to AI agents like Claude. It implements the JSON-RPC 2.0 protocol over stdio.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::event::{BrowserCommand, BrowserEvent};

/// MCP protocol version
const MCP_VERSION: &str = "2024-11-05";

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCP server state
pub struct McpServer {
    command_tx: broadcast::Sender<BrowserCommand>,
    event_rx: broadcast::Receiver<BrowserEvent>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(
        command_tx: broadcast::Sender<BrowserCommand>,
        event_rx: broadcast::Receiver<BrowserEvent>,
    ) -> Self {
        Self {
            command_tx,
            event_rx,
        }
    }

    /// Run the MCP server (blocking)
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting MCP server on stdio");
        info!("📡 Ready to receive MCP protocol commands");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            debug!("Received MCP request: {}", line);

            // Parse JSON-RPC request
            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => self.handle_request(request),
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                },
            };

            // Send response
            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tool_call(request.params),
            "resources/list" => self.handle_resources_list(),
            "prompts/list" => self.handle_prompts_list(),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(error),
            },
        }
    }

    /// Handle initialize request
    fn handle_initialize(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        debug!("Handling initialize request: {:?}", params);

        Ok(json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                },
                "prompts": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "tinker-browser-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    /// Handle tools/list request
    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        debug!("Handling tools/list request");

        let tools = vec![
            self.tool_definition(
                "navigate",
                "Navigate the browser to a URL",
                json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to navigate to"
                        }
                    },
                    "required": ["url"]
                }),
            ),
            self.tool_definition(
                "create_tab",
                "Create a new browser tab",
                json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to open in the new tab"
                        }
                    },
                    "required": ["url"]
                }),
            ),
            self.tool_definition(
                "close_tab",
                "Close a browser tab",
                json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "number",
                            "description": "The tab ID to close"
                        }
                    },
                    "required": ["id"]
                }),
            ),
            self.tool_definition(
                "switch_tab",
                "Switch to a different tab",
                json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "number",
                            "description": "The tab ID to switch to"
                        }
                    },
                    "required": ["id"]
                }),
            ),
            self.tool_definition(
                "take_screenshot",
                "Capture a screenshot of the current page",
                json!({
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "description": "Image format (png, jpeg, webp)",
                            "enum": ["png", "jpeg", "webp"]
                        },
                        "quality": {
                            "type": "number",
                            "description": "Image quality for JPEG (0-100)"
                        }
                    }
                }),
            ),
            self.tool_definition(
                "find_element",
                "Find DOM elements using CSS selector, XPath, or text content",
                json!({
                    "type": "object",
                    "properties": {
                        "selector_type": {
                            "type": "string",
                            "description": "Type of selector (css, xpath, text)",
                            "enum": ["css", "xpath", "text"]
                        },
                        "selector": {
                            "type": "string",
                            "description": "The selector string"
                        }
                    },
                    "required": ["selector_type", "selector"]
                }),
            ),
            self.tool_definition(
                "click_element",
                "Click on a DOM element",
                json!({
                    "type": "object",
                    "properties": {
                        "selector_type": {
                            "type": "string",
                            "description": "Type of selector (css, xpath, text)",
                            "enum": ["css", "xpath", "text"]
                        },
                        "selector": {
                            "type": "string",
                            "description": "The selector string"
                        }
                    },
                    "required": ["selector_type", "selector"]
                }),
            ),
            self.tool_definition(
                "type_text",
                "Type text into an input element",
                json!({
                    "type": "object",
                    "properties": {
                        "selector_type": {
                            "type": "string",
                            "description": "Type of selector (css, xpath, text)",
                            "enum": ["css", "xpath", "text"]
                        },
                        "selector": {
                            "type": "string",
                            "description": "The selector string"
                        },
                        "text": {
                            "type": "string",
                            "description": "The text to type"
                        }
                    },
                    "required": ["selector_type", "selector", "text"]
                }),
            ),
            self.tool_definition(
                "get_page_info",
                "Get information about the current page (title, URL, HTML)",
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            self.tool_definition(
                "execute_javascript",
                "Execute JavaScript code in the page context",
                json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "JavaScript code to execute"
                        }
                    },
                    "required": ["script"]
                }),
            ),
            self.tool_definition(
                "start_network_monitoring",
                "Start monitoring network requests and responses",
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            self.tool_definition(
                "stop_network_monitoring",
                "Stop monitoring network requests",
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            self.tool_definition(
                "get_network_stats",
                "Get network monitoring statistics",
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            self.tool_definition(
                "export_network_har",
                "Export network traffic as HAR file",
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            self.tool_definition(
                "create_visual_baseline",
                "Create a visual baseline for regression testing",
                json!({
                    "type": "object",
                    "properties": {
                        "test_name": {
                            "type": "string",
                            "description": "Name of the visual test"
                        }
                    },
                    "required": ["test_name"]
                }),
            ),
            self.tool_definition(
                "run_visual_test",
                "Run a visual regression test against baseline",
                json!({
                    "type": "object",
                    "properties": {
                        "test_name": {
                            "type": "string",
                            "description": "Name of the visual test"
                        },
                        "tolerance": {
                            "type": "number",
                            "description": "Tolerance for differences (0.0 to 1.0, default 0.1)"
                        }
                    },
                    "required": ["test_name"]
                }),
            ),
        ];

        Ok(json!({
            "tools": tools
        }))
    }

    /// Handle tools/call request
    fn handle_tool_call(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: missing params".to_string(),
            data: None,
        })?;

        let tool_name = params["name"].as_str().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: missing tool name".to_string(),
            data: None,
        })?;

        let arguments = params["arguments"].clone();

        debug!("Tool call: {} with args: {:?}", tool_name, arguments);

        let command = match tool_name {
            "navigate" => {
                let url = arguments["url"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: url".to_string(),
                    data: None,
                })?;
                BrowserCommand::Navigate {
                    url: url.to_string(),
                }
            }
            "create_tab" => {
                let url = arguments["url"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: url".to_string(),
                    data: None,
                })?;
                BrowserCommand::CreateTab {
                    url: url.to_string(),
                }
            }
            "close_tab" => {
                let id = arguments["id"].as_u64().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: id".to_string(),
                    data: None,
                })? as usize;
                BrowserCommand::CloseTab { id }
            }
            "switch_tab" => {
                let id = arguments["id"].as_u64().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: id".to_string(),
                    data: None,
                })? as usize;
                BrowserCommand::SwitchTab { id }
            }
            "take_screenshot" => BrowserCommand::TakeScreenshot {
                options: Some(arguments.clone()),
            },
            "find_element" => {
                let selector_type = arguments["selector_type"]
                    .as_str()
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing required parameter: selector_type".to_string(),
                        data: None,
                    })?;
                let selector = arguments["selector"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: selector".to_string(),
                    data: None,
                })?;

                BrowserCommand::FindElement {
                    selector: json!({
                        "type": selector_type,
                        "value": selector
                    }),
                }
            }
            "click_element" => {
                let selector_type = arguments["selector_type"]
                    .as_str()
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing required parameter: selector_type".to_string(),
                        data: None,
                    })?;
                let selector = arguments["selector"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: selector".to_string(),
                    data: None,
                })?;

                BrowserCommand::InteractElement {
                    selector: json!({
                        "type": selector_type,
                        "value": selector
                    }),
                    interaction: json!({
                        "type": "click"
                    }),
                }
            }
            "type_text" => {
                let selector_type = arguments["selector_type"]
                    .as_str()
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing required parameter: selector_type".to_string(),
                        data: None,
                    })?;
                let selector = arguments["selector"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: selector".to_string(),
                    data: None,
                })?;
                let text = arguments["text"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: text".to_string(),
                    data: None,
                })?;

                BrowserCommand::InteractElement {
                    selector: json!({
                        "type": selector_type,
                        "value": selector
                    }),
                    interaction: json!({
                        "type": "type",
                        "text": text
                    }),
                }
            }
            "get_page_info" => BrowserCommand::GetPageInfo,
            "execute_javascript" => {
                let script = arguments["script"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: script".to_string(),
                    data: None,
                })?;
                BrowserCommand::ExecuteJavaScript {
                    script: script.to_string(),
                }
            }
            "start_network_monitoring" => BrowserCommand::StartNetworkMonitoring,
            "stop_network_monitoring" => BrowserCommand::StopNetworkMonitoring,
            "get_network_stats" => BrowserCommand::GetNetworkStats,
            "export_network_har" => BrowserCommand::ExportNetworkHAR,
            "create_visual_baseline" => {
                let test_name = arguments["test_name"]
                    .as_str()
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing required parameter: test_name".to_string(),
                        data: None,
                    })?;
                BrowserCommand::CreateBaseline {
                    test_name: test_name.to_string(),
                    options: None,
                }
            }
            "run_visual_test" => {
                let test_name = arguments["test_name"]
                    .as_str()
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing required parameter: test_name".to_string(),
                        data: None,
                    })?;
                let tolerance = arguments["tolerance"].as_f64().unwrap_or(0.1);
                BrowserCommand::RunVisualTest {
                    test_name: test_name.to_string(),
                    tolerance,
                    options: None,
                }
            }
            _ => {
                return Err(JsonRpcError {
                    code: -32601,
                    message: format!("Unknown tool: {}", tool_name),
                    data: None,
                });
            }
        };

        // Send command to browser
        self.command_tx.send(command).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to send command: {}", e),
            data: None,
        })?;

        // Return success - in a real implementation, we'd wait for the response
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Command '{}' sent successfully", tool_name)
            }]
        }))
    }

    /// Handle resources/list request
    fn handle_resources_list(&self) -> Result<Value, JsonRpcError> {
        debug!("Handling resources/list request");
        Ok(json!({
            "resources": []
        }))
    }

    /// Handle prompts/list request
    fn handle_prompts_list(&self) -> Result<Value, JsonRpcError> {
        debug!("Handling prompts/list request");
        Ok(json!({
            "prompts": []
        }))
    }

    /// Create a tool definition
    fn tool_definition(&self, name: &str, description: &str, input_schema: Value) -> Value {
        json!({
            "name": name,
            "description": description,
            "inputSchema": input_schema
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    fn setup_test_server() -> (McpServer, broadcast::Receiver<BrowserCommand>) {
        let (command_tx, command_rx) = broadcast::channel(100);
        let (_event_tx, event_rx) = broadcast::channel(100);
        (McpServer::new(command_tx, event_rx), command_rx)
    }

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

        let response = result.unwrap();
        assert_eq!(response["protocolVersion"], "2024-11-05");
        assert_eq!(response["serverInfo"]["name"], "tinker-browser-mcp");
        assert!(response["capabilities"]["tools"].is_object());
    }

    #[test]
    fn test_tools_list() {
        let (server, _rx) = setup_test_server();
        let result = server.handle_tools_list();
        assert!(result.is_ok());

        let response = result.unwrap();
        let tools = response["tools"].as_array().unwrap();

        // Should have all 16 tools
        assert!(tools.len() >= 16);

        // Check for specific tools
        let tool_names: Vec<&str> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();

        assert!(tool_names.contains(&"navigate"));
        assert!(tool_names.contains(&"create_tab"));
        assert!(tool_names.contains(&"click_element"));
        assert!(tool_names.contains(&"execute_javascript"));
        assert!(tool_names.contains(&"take_screenshot"));
    }

    #[test]
    fn test_navigate_tool_call() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "navigate",
            "arguments": {
                "url": "https://example.com"
            }
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["content"].is_array());
        let content = &response["content"][0];
        assert_eq!(content["type"], "text");
        assert!(content["text"].as_str().unwrap().contains("navigate"));
    }

    #[test]
    fn test_missing_tool_name() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "arguments": {
                "url": "https://example.com"
            }
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("missing tool name"));
    }

    #[test]
    fn test_unknown_tool() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "nonexistent_tool",
            "arguments": {}
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Unknown tool"));
    }

    #[test]
    fn test_missing_required_parameter() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "navigate",
            "arguments": {}
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing required parameter"));
    }

    #[test]
    fn test_click_element_tool() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "click_element",
            "arguments": {
                "selector_type": "css",
                "selector": ".button"
            }
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["content"][0]["text"].as_str().unwrap().contains("click_element"));
    }

    #[test]
    fn test_type_text_tool() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "type_text",
            "arguments": {
                "selector_type": "css",
                "selector": "input[name='search']",
                "text": "Hello World"
            }
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_javascript_tool() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "execute_javascript",
            "arguments": {
                "script": "document.title"
            }
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_screenshot_tool() {
        let (mut server, _rx) = setup_test_server();
        let params = json!({
            "name": "take_screenshot",
            "arguments": {
                "format": "png"
            }
        });

        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_network_monitoring_tools() {
        let (mut server, _rx) = setup_test_server();

        // Start monitoring
        let params = json!({
            "name": "start_network_monitoring",
            "arguments": {}
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        // Get stats
        let params = json!({
            "name": "get_network_stats",
            "arguments": {}
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        // Stop monitoring
        let params = json!({
            "name": "stop_network_monitoring",
            "arguments": {}
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_visual_testing_tools() {
        let (mut server, _rx) = setup_test_server();

        // Create baseline
        let params = json!({
            "name": "create_visual_baseline",
            "arguments": {
                "test_name": "homepage"
            }
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        // Run visual test
        let params = json!({
            "name": "run_visual_test",
            "arguments": {
                "test_name": "homepage",
                "tolerance": 0.05
            }
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_tab_management_tools() {
        let (mut server, _rx) = setup_test_server();

        // Create tab
        let params = json!({
            "name": "create_tab",
            "arguments": {
                "url": "https://example.com"
            }
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        // Switch tab
        let params = json!({
            "name": "switch_tab",
            "arguments": {
                "id": 1
            }
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());

        // Close tab
        let params = json!({
            "name": "close_tab",
            "arguments": {
                "id": 1
            }
        });
        let result = server.handle_tool_call(Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_resources_list() {
        let (server, _rx) = setup_test_server();
        let result = server.handle_resources_list();
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["resources"].is_array());
    }

    #[test]
    fn test_prompts_list() {
        let (server, _rx) = setup_test_server();
        let result = server.handle_prompts_list();
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response["prompts"].is_array());
    }

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

        let request = request.unwrap();
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "initialize");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        assert!(json_str.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("\"code\":-32600"));
        assert!(json_str.contains("Invalid request"));
    }

    #[test]
    fn test_handle_request_method_not_found() {
        let (mut server, _rx) = setup_test_server();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "nonexistent_method".to_string(),
            params: None,
        };

        let response = server.handle_request(request);
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Method not found"));
    }

    #[test]
    fn test_tool_definition() {
        let (server, _rx) = setup_test_server();
        let definition = server.tool_definition(
            "test_tool",
            "A test tool",
            json!({
                "type": "object",
                "properties": {
                    "param": {"type": "string"}
                }
            }),
        );

        assert_eq!(definition["name"], "test_tool");
        assert_eq!(definition["description"], "A test tool");
        assert!(definition["inputSchema"].is_object());
        assert_eq!(definition["inputSchema"]["type"], "object");
    }

    #[test]
    fn test_find_element_different_selectors() {
        let (mut server, _rx) = setup_test_server();

        // CSS selector
        let params = json!({
            "name": "find_element",
            "arguments": {
                "selector_type": "css",
                "selector": ".my-class"
            }
        });
        assert!(server.handle_tool_call(Some(params)).is_ok());

        // XPath selector
        let params = json!({
            "name": "find_element",
            "arguments": {
                "selector_type": "xpath",
                "selector": "//div[@class='my-class']"
            }
        });
        assert!(server.handle_tool_call(Some(params)).is_ok());

        // Text selector
        let params = json!({
            "name": "find_element",
            "arguments": {
                "selector_type": "text",
                "selector": "Click me"
            }
        });
        assert!(server.handle_tool_call(Some(params)).is_ok());
    }
}
