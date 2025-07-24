//! HTTP and WebSocket API server for browser control

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Path, Query, State},
    response::Response,
    routing::{get, post},
    Router,
    Json,
};
use axum::extract::ws::Message;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, error};
use tokio::sync::broadcast;
use std::collections::HashMap;

use crate::event::{BrowserEvent, BrowserCommand};

#[derive(Clone)]
pub struct ApiState {
    command_tx: broadcast::Sender<BrowserCommand>,
    event_rx: Arc<Mutex<broadcast::Receiver<BrowserEvent>>>,
}

#[derive(Serialize, Deserialize)]
pub struct NavigateRequest {
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTabRequest {
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct TabActionRequest {
    id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ScreenshotRequest {
    options: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct BaselineRequest {
    test_name: String,
    options: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct VisualTestRequest {
    test_name: String,
    tolerance: Option<f64>,
    options: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ElementSelectorRequest {
    selector: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct InteractionRequest {
    selector: serde_json::Value,
    interaction: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct HighlightRequest {
    selector: serde_json::Value,
    color: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct WaitConditionRequest {
    condition: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct JavaScriptRequest {
    script: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkFilterRequest {
    filter: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

pub async fn start_api_server(
    command_tx: broadcast::Sender<BrowserCommand>,
    event_rx: broadcast::Receiver<BrowserEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = ApiState {
        command_tx,
        event_rx: Arc::new(Mutex::new(event_rx)),
    };

    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Browser control endpoints
        .route("/api/navigate", post(navigate))
        .route("/api/tabs", post(create_tab))
        .route("/api/tabs/:id", axum::routing::delete(close_tab))
        .route("/api/tabs/:id/activate", post(activate_tab))
        
        // Visual testing endpoints
        .route("/api/screenshot", post(take_screenshot))
        .route("/api/visual/baseline", post(create_baseline))
        .route("/api/visual/test", post(run_visual_test))
        
        // DOM inspection endpoints
        .route("/api/element/find", post(find_element))
        .route("/api/element/interact", post(interact_element))
        .route("/api/element/highlight", post(highlight_element))
        .route("/api/element/wait", post(wait_for_condition))
        .route("/api/page/info", get(get_page_info))
        .route("/api/javascript/execute", post(execute_javascript))
        
        // Network monitoring endpoints
        .route("/api/network/start", post(start_network_monitoring))
        .route("/api/network/stop", post(stop_network_monitoring))
        .route("/api/network/stats", get(get_network_stats))
        .route("/api/network/export", get(export_network_har))
        .route("/api/network/filter", post(add_network_filter))
        .route("/api/network/clear-filters", post(clear_network_filters))
        
        // WebSocket endpoint for real-time control
        .route("/ws", get(websocket_handler))
        
        // Static info endpoints
        .route("/api/info", get(browser_info))
        
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
    info!("🚀 Tinker API server listening on http://{}", addr);
    info!("🔌 WebSocket endpoint: ws://{}/ws", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "name": "Tinker Browser API"
    })))
}

async fn browser_info() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(serde_json::json!({
        "name": "Tinker",
        "version": env!("CARGO_PKG_VERSION"),
        "engine": "WebView",
        "capabilities": [
            "navigation",
            "tab_management", 
            "event_streaming",
            "mqtt_integration",
            "recording_replay",
            "screenshot_capture",
            "visual_testing",
            "baseline_comparison",
            "dom_inspection",
            "element_interaction",
            "javascript_injection",
            "wait_conditions",
            "network_monitoring",
            "network_analysis",
            "har_export"
        ],
        "endpoints": {
            "navigate": "POST /api/navigate",
            "create_tab": "POST /api/tabs",
            "close_tab": "DELETE /api/tabs/{id}",
            "activate_tab": "POST /api/tabs/{id}/activate",
            "screenshot": "POST /api/screenshot",
            "create_baseline": "POST /api/visual/baseline",
            "run_visual_test": "POST /api/visual/test",
            "find_element": "POST /api/element/find",
            "interact_element": "POST /api/element/interact",
            "highlight_element": "POST /api/element/highlight",
            "wait_condition": "POST /api/element/wait",
            "page_info": "GET /api/page/info",
            "execute_javascript": "POST /api/javascript/execute",
            "start_network_monitoring": "POST /api/network/start",
            "stop_network_monitoring": "POST /api/network/stop",
            "get_network_stats": "GET /api/network/stats",
            "export_network_har": "GET /api/network/export",
            "add_network_filter": "POST /api/network/filter",
            "clear_network_filters": "POST /api/network/clear-filters",
            "websocket": "WS /ws"
        }
    })))
}

async fn navigate(
    State(state): State<ApiState>,
    Json(request): Json<NavigateRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Navigate to {}", request.url);
    
    let command = BrowserCommand::Navigate { url: request.url.clone() };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success(format!("Navigating to {}", request.url))),
        Err(e) => Json(ApiResponse::error(format!("Failed to send navigate command: {}", e))),
    }
}

async fn create_tab(
    State(state): State<ApiState>,
    Json(request): Json<CreateTabRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Create tab with URL {}", request.url);
    
    let command = BrowserCommand::CreateTab { url: request.url.clone() };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success(format!("Creating tab with URL {}", request.url))),
        Err(e) => Json(ApiResponse::error(format!("Failed to send create tab command: {}", e))),
    }
}

async fn close_tab(
    State(state): State<ApiState>,
    Path(id): Path<usize>,
) -> Json<ApiResponse<String>> {
    debug!("API: Close tab {}", id);
    
    let command = BrowserCommand::CloseTab { id };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success(format!("Closing tab {}", id))),
        Err(e) => Json(ApiResponse::error(format!("Failed to send close tab command: {}", e))),
    }
}

async fn activate_tab(
    State(state): State<ApiState>,
    Path(id): Path<usize>,
) -> Json<ApiResponse<String>> {
    debug!("API: Activate tab {}", id);
    
    let command = BrowserCommand::SwitchTab { id };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success(format!("Activating tab {}", id))),
        Err(e) => Json(ApiResponse::error(format!("Failed to send activate tab command: {}", e))),
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: ApiState) {
    info!("🔌 New WebSocket connection established");
    
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = state.event_rx.lock().unwrap().resubscribe();
    
    // Spawn task to forward browser events to WebSocket
    let sender_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let message = serde_json::json!({
                "type": "event",
                "data": event
            });
            
            if let Err(e) = sender.send(Message::Text(message.to_string())).await {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });
    
    // Handle incoming WebSocket commands
    let command_tx = state.command_tx.clone();
    let receiver_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("WebSocket received: {}", text);
                    
                    if let Ok(command) = serde_json::from_str::<BrowserCommand>(&text) {
                        if let Err(e) = command_tx.send(command) {
                            error!("Failed to send command from WebSocket: {}", e);
                        }
                    } else {
                        error!("Invalid WebSocket command format: {}", text);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = sender_task => {},
        _ = receiver_task => {},
    }
    
    info!("🔌 WebSocket connection terminated");
}

async fn take_screenshot(
    State(state): State<ApiState>,
    Json(request): Json<ScreenshotRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Take screenshot");
    
    let command = BrowserCommand::TakeScreenshot { options: request.options };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Screenshot command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send screenshot command: {}", e))),
    }
}

async fn create_baseline(
    State(state): State<ApiState>,
    Json(request): Json<BaselineRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Create baseline for test '{}'", request.test_name);
    
    let command = BrowserCommand::CreateBaseline { 
        test_name: request.test_name.clone(),
        options: request.options 
    };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success(format!("Baseline creation started for '{}'", request.test_name))),
        Err(e) => Json(ApiResponse::error(format!("Failed to send baseline command: {}", e))),
    }
}

async fn run_visual_test(
    State(state): State<ApiState>,
    Json(request): Json<VisualTestRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Run visual test '{}'", request.test_name);
    
    let tolerance = request.tolerance.unwrap_or(0.1); // Default 10% tolerance
    let command = BrowserCommand::RunVisualTest { 
        test_name: request.test_name.clone(),
        tolerance,
        options: request.options 
    };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success(format!("Visual test '{}' started", request.test_name))),
        Err(e) => Json(ApiResponse::error(format!("Failed to send visual test command: {}", e))),
    }
}

async fn find_element(
    State(state): State<ApiState>,
    Json(request): Json<ElementSelectorRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Find element");
    
    let command = BrowserCommand::FindElement { selector: request.selector };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Element search command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send find element command: {}", e))),
    }
}

async fn interact_element(
    State(state): State<ApiState>,
    Json(request): Json<InteractionRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Interact with element");
    
    let command = BrowserCommand::InteractElement { 
        selector: request.selector,
        interaction: request.interaction 
    };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Element interaction command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send interaction command: {}", e))),
    }
}

async fn highlight_element(
    State(state): State<ApiState>,
    Json(request): Json<HighlightRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Highlight element");
    
    let command = BrowserCommand::HighlightElement { 
        selector: request.selector,
        color: request.color 
    };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Element highlight command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send highlight command: {}", e))),
    }
}

async fn wait_for_condition(
    State(state): State<ApiState>,
    Json(request): Json<WaitConditionRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Wait for condition");
    
    let command = BrowserCommand::WaitForCondition { condition: request.condition };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Wait condition command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send wait condition command: {}", e))),
    }
}

async fn get_page_info(
    State(state): State<ApiState>,
) -> Json<ApiResponse<String>> {
    debug!("API: Get page info");
    
    let command = BrowserCommand::GetPageInfo;
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Page info command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send page info command: {}", e))),
    }
}

async fn execute_javascript(
    State(state): State<ApiState>,
    Json(request): Json<JavaScriptRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Execute JavaScript");
    
    let command = BrowserCommand::ExecuteJavaScript { script: request.script };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("JavaScript execution command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to send JavaScript command: {}", e))),
    }
}

async fn start_network_monitoring(
    State(state): State<ApiState>,
) -> Json<ApiResponse<String>> {
    debug!("API: Start network monitoring");
    
    let command = BrowserCommand::StartNetworkMonitoring;
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Network monitoring started".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to start network monitoring: {}", e))),
    }
}

async fn stop_network_monitoring(
    State(state): State<ApiState>,
) -> Json<ApiResponse<String>> {
    debug!("API: Stop network monitoring");
    
    let command = BrowserCommand::StopNetworkMonitoring;
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Network monitoring stopped".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to stop network monitoring: {}", e))),
    }
}

async fn get_network_stats(
    State(state): State<ApiState>,
) -> Json<ApiResponse<String>> {
    debug!("API: Get network stats");
    
    let command = BrowserCommand::GetNetworkStats;
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Network stats command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to get network stats: {}", e))),
    }
}

async fn export_network_har(
    State(state): State<ApiState>,
) -> Json<ApiResponse<String>> {
    debug!("API: Export network HAR");
    
    let command = BrowserCommand::ExportNetworkHAR;
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Network HAR export command sent".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to export network HAR: {}", e))),
    }
}

async fn add_network_filter(
    State(state): State<ApiState>,
    Json(request): Json<NetworkFilterRequest>,
) -> Json<ApiResponse<String>> {
    debug!("API: Add network filter");
    
    let command = BrowserCommand::AddNetworkFilter { filter: request.filter };
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Network filter added".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to add network filter: {}", e))),
    }
}

async fn clear_network_filters(
    State(state): State<ApiState>,
) -> Json<ApiResponse<String>> {
    debug!("API: Clear network filters");
    
    let command = BrowserCommand::ClearNetworkFilters;
    match state.command_tx.send(command) {
        Ok(_) => Json(ApiResponse::success("Network filters cleared".to_string())),
        Err(e) => Json(ApiResponse::error(format!("Failed to clear network filters: {}", e))),
    }
}
