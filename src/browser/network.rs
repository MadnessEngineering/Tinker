//! Network request/response monitoring and analysis

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, error};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Unique request ID
    pub id: String,
    /// Request method (GET, POST, etc.)
    pub method: String,
    /// Request URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (base64 encoded if binary)
    pub body: Option<String>,
    /// Request timestamp
    pub timestamp: u64,
    /// Request initiator (script, document, etc.)
    pub initiator: String,
    /// Resource type (document, stylesheet, script, xhr, etc.)
    pub resource_type: String,
    /// Whether request was blocked
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    /// Request ID this response belongs to
    pub request_id: String,
    /// Response status code
    pub status_code: u16,
    /// Response status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body (base64 encoded if binary)
    pub body: Option<String>,
    /// Response timestamp
    pub timestamp: u64,
    /// Response size in bytes
    pub size: u64,
    /// Whether response was from cache
    pub from_cache: bool,
    /// MIME type
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    /// Event type
    pub event_type: NetworkEventType,
    /// Request information
    pub request: Option<NetworkRequest>,
    /// Response information
    pub response: Option<NetworkResponse>,
    /// Timing information
    pub timing: Option<NetworkTiming>,
    /// Error information
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEventType {
    RequestStarted,
    RequestBlocked,
    ResponseReceived,
    ResponseFinished,
    RequestFailed,
    LoadingFinished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTiming {
    /// DNS lookup time in milliseconds
    pub dns_lookup: Option<f64>,
    /// Connection time in milliseconds
    pub connect: Option<f64>,
    /// SSL handshake time in milliseconds
    pub ssl: Option<f64>,
    /// Time to first byte in milliseconds
    pub ttfb: Option<f64>,
    /// Download time in milliseconds
    pub download: Option<f64>,
    /// Total request time in milliseconds
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFilter {
    /// Filter by URL pattern
    pub url_pattern: Option<String>,
    /// Filter by method
    pub method: Option<String>,
    /// Filter by resource type
    pub resource_type: Option<String>,
    /// Filter by status code range
    pub status_code_min: Option<u16>,
    pub status_code_max: Option<u16>,
    /// Only include failed requests
    pub failed_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total number of requests
    pub total_requests: u32,
    /// Total number of failed requests
    pub failed_requests: u32,
    /// Total bytes transferred
    pub total_bytes: u64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Fastest response time in milliseconds
    pub fastest_response: f64,
    /// Slowest response time in milliseconds
    pub slowest_response: f64,
    /// Requests by resource type
    pub by_resource_type: HashMap<String, u32>,
    /// Requests by status code
    pub by_status_code: HashMap<u16, u32>,
    /// Top domains by request count
    pub top_domains: Vec<(String, u32)>,
}

pub struct NetworkMonitor {
    /// Active network requests
    requests: HashMap<String, NetworkRequest>,
    /// Completed network transactions
    completed: Vec<NetworkEvent>,
    /// Network statistics
    stats: NetworkStats,
    /// Request filters
    filters: Vec<NetworkFilter>,
    /// Whether monitoring is enabled
    enabled: bool,
    /// Maximum number of completed requests to keep
    max_history: usize,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            completed: Vec::new(),
            stats: NetworkStats::default(),
            filters: Vec::new(),
            enabled: true,
            max_history: 1000,
        }
    }

    /// Start monitoring network requests
    pub fn start_monitoring(&mut self) {
        info!("🌐 Starting network monitoring");
        self.enabled = true;
        self.requests.clear();
        self.completed.clear();
        self.stats = NetworkStats::default();
    }

    /// Stop monitoring network requests
    pub fn stop_monitoring(&mut self) {
        info!("🛑 Stopping network monitoring");
        self.enabled = false;
    }

    /// Record a new network request
    pub fn record_request(&mut self, request: NetworkRequest) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!("📤 Recording network request: {} {}", request.method, request.url);
        
        // Update stats
        self.stats.total_requests += 1;
        
        // Apply filters
        if self.should_record(&request) {
            let event = NetworkEvent {
                event_type: NetworkEventType::RequestStarted,
                request: Some(request.clone()),
                response: None,
                timing: None,
                error: None,
            };
            
            self.requests.insert(request.id.clone(), request);
            self.add_event(event);
        }

        Ok(())
    }

    /// Record a network response
    pub fn record_response(&mut self, response: NetworkResponse) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!("📥 Recording network response: {} for {}", response.status_code, response.request_id);

        if let Some(request) = self.requests.get(&response.request_id).cloned() {
            // Calculate timing if possible
            let timing = self.calculate_timing(&request, &response);

            let event = NetworkEvent {
                event_type: NetworkEventType::ResponseReceived,
                request: Some(request.clone()),
                response: Some(response.clone()),
                timing: timing.clone(),
                error: None,
            };

            // Update stats
            self.update_stats(&request, &response, &timing);
            
            self.add_event(event);
        }

        Ok(())
    }

    /// Record a failed network request
    pub fn record_failure(&mut self, request_id: &str, error: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!("❌ Recording network failure for {}: {}", request_id, error);

        if let Some(request) = self.requests.get(request_id).cloned() {
            let event = NetworkEvent {
                event_type: NetworkEventType::RequestFailed,
                request: Some(request),
                response: None,
                timing: None,
                error: Some(error.to_string()),
            };

            self.stats.failed_requests += 1;
            self.add_event(event);
        }

        Ok(())
    }

    /// Add a network filter
    pub fn add_filter(&mut self, filter: NetworkFilter) {
        debug!("🔍 Adding network filter: {:?}", filter);
        self.filters.push(filter);
    }

    /// Clear all network filters
    pub fn clear_filters(&mut self) {
        debug!("🧹 Clearing all network filters");
        self.filters.clear();
    }

    /// Get network statistics
    pub fn get_stats(&self) -> &NetworkStats {
        &self.stats
    }

    /// Get recent network events
    pub fn get_recent_events(&self, limit: Option<usize>) -> Vec<&NetworkEvent> {
        let limit = limit.unwrap_or(50);
        self.completed.iter().rev().take(limit).collect()
    }

    /// Get events matching a filter
    pub fn get_filtered_events(&self, filter: &NetworkFilter) -> Vec<&NetworkEvent> {
        self.completed.iter()
            .filter(|event| self.event_matches_filter(event, filter))
            .collect()
    }

    /// Export network data as HAR (HTTP Archive) format
    pub fn export_har(&self) -> Result<String> {
        let har = serde_json::json!({
            "log": {
                "version": "1.2",
                "creator": {
                    "name": "Tinker Browser",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "entries": self.completed.iter()
                    .filter_map(|event| self.event_to_har_entry(event))
                    .collect::<Vec<_>>()
            }
        });

        Ok(serde_json::to_string_pretty(&har)?)
    }

    /// Generate JavaScript for network monitoring injection
    pub fn get_monitoring_script(&self) -> String {
        r#"
// Network monitoring injection
(function() {
    const originalFetch = window.fetch;
    const originalXHR = window.XMLHttpRequest;
    
    // Monitor fetch requests
    window.fetch = function(...args) {
        const request = {
            id: 'fetch_' + Date.now() + '_' + Math.random(),
            method: args[1]?.method || 'GET',
            url: args[0].toString(),
            headers: Object.fromEntries((args[1]?.headers || new Headers())),
            body: args[1]?.body || null,
            timestamp: Date.now(),
            initiator: 'fetch',
            resource_type: 'fetch'
        };
        
        window.parent.postMessage({
            type: 'network_request',
            data: request
        }, '*');
        
        return originalFetch.apply(this, args).then(response => {
            const responseData = {
                request_id: request.id,
                status_code: response.status,
                status_text: response.statusText,
                headers: Object.fromEntries(response.headers),
                timestamp: Date.now(),
                from_cache: false,
                mime_type: response.headers.get('content-type') || 'unknown'
            };
            
            window.parent.postMessage({
                type: 'network_response',
                data: responseData
            }, '*');
            
            return response;
        }).catch(error => {
            window.parent.postMessage({
                type: 'network_error',
                data: {
                    request_id: request.id,
                    error: error.message
                }
            }, '*');
            throw error;
        });
    };
    
    // Monitor XMLHttpRequest
    const xhrProto = XMLHttpRequest.prototype;
    const originalOpen = xhrProto.open;
    const originalSend = xhrProto.send;
    
    xhrProto.open = function(method, url, ...args) {
        this._tinkerRequest = {
            id: 'xhr_' + Date.now() + '_' + Math.random(),
            method: method,
            url: url,
            headers: {},
            timestamp: Date.now(),
            initiator: 'xhr',
            resource_type: 'xhr'
        };
        return originalOpen.apply(this, [method, url, ...args]);
    };
    
    xhrProto.send = function(body) {
        if (this._tinkerRequest) {
            this._tinkerRequest.body = body;
            
            window.parent.postMessage({
                type: 'network_request',
                data: this._tinkerRequest
            }, '*');
            
            this.addEventListener('load', () => {
                const responseData = {
                    request_id: this._tinkerRequest.id,
                    status_code: this.status,
                    status_text: this.statusText,
                    headers: this.getAllResponseHeaders().split('\r\n')
                        .filter(line => line)
                        .reduce((headers, line) => {
                            const [key, value] = line.split(': ');
                            headers[key] = value;
                            return headers;
                        }, {}),
                    body: this.responseText,
                    timestamp: Date.now(),
                    size: this.responseText.length,
                    from_cache: false,
                    mime_type: this.getResponseHeader('content-type') || 'unknown'
                };
                
                window.parent.postMessage({
                    type: 'network_response',
                    data: responseData
                }, '*');
            });
            
            this.addEventListener('error', () => {
                window.parent.postMessage({
                    type: 'network_error',
                    data: {
                        request_id: this._tinkerRequest.id,
                        error: 'XMLHttpRequest failed'
                    }
                }, '*');
            });
        }
        
        return originalSend.apply(this, [body]);
    };
    
    console.log('Tinker network monitoring injected successfully');
})();
        "#.to_string()
    }

    fn should_record(&self, request: &NetworkRequest) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        self.filters.iter().any(|filter| {
            // URL pattern match
            if let Some(pattern) = &filter.url_pattern {
                if !request.url.contains(pattern) {
                    return false;
                }
            }

            // Method match
            if let Some(method) = &filter.method {
                if request.method != *method {
                    return false;
                }
            }

            // Resource type match
            if let Some(resource_type) = &filter.resource_type {
                if request.resource_type != *resource_type {
                    return false;
                }
            }

            true
        })
    }

    fn event_matches_filter(&self, event: &NetworkEvent, filter: &NetworkFilter) -> bool {
        // Implementation for filtering events
        if let Some(request) = &event.request {
            return self.should_record(request);
        }
        true
    }

    fn calculate_timing(&self, request: &NetworkRequest, response: &NetworkResponse) -> Option<NetworkTiming> {
        let total = (response.timestamp - request.timestamp) as f64;
        Some(NetworkTiming {
            dns_lookup: None,  // Would need browser API access
            connect: None,     // Would need browser API access
            ssl: None,         // Would need browser API access
            ttfb: None,        // Would need browser API access
            download: None,    // Would need browser API access
            total,
        })
    }

    fn update_stats(&mut self, request: &NetworkRequest, response: &NetworkResponse, timing: &Option<NetworkTiming>) {
        self.stats.total_bytes += response.size;
        
        if let Some(timing) = timing {
            let response_time = timing.total;
            
            if self.stats.total_requests == 1 {
                self.stats.avg_response_time = response_time;
                self.stats.fastest_response = response_time;
                self.stats.slowest_response = response_time;
            } else {
                self.stats.avg_response_time = 
                    (self.stats.avg_response_time * (self.stats.total_requests - 1) as f64 + response_time) 
                    / self.stats.total_requests as f64;
                
                if response_time < self.stats.fastest_response {
                    self.stats.fastest_response = response_time;
                }
                if response_time > self.stats.slowest_response {
                    self.stats.slowest_response = response_time;
                }
            }
        }

        // Update resource type counts
        *self.stats.by_resource_type.entry(request.resource_type.clone()).or_insert(0) += 1;
        
        // Update status code counts
        *self.stats.by_status_code.entry(response.status_code).or_insert(0) += 1;

        // Update domain stats
        if let Ok(url) = url::Url::parse(&request.url) {
            if let Some(domain) = url.host_str() {
                let entry = self.stats.top_domains.iter_mut()
                    .find(|(d, _)| d == domain);
                
                if let Some((_, count)) = entry {
                    *count += 1;
                } else {
                    self.stats.top_domains.push((domain.to_string(), 1));
                }
                
                // Keep only top 10 domains
                self.stats.top_domains.sort_by(|a, b| b.1.cmp(&a.1));
                self.stats.top_domains.truncate(10);
            }
        }
    }

    fn add_event(&mut self, event: NetworkEvent) {
        self.completed.push(event);
        
        // Maintain history limit
        if self.completed.len() > self.max_history {
            self.completed.remove(0);
        }
    }

    fn event_to_har_entry(&self, event: &NetworkEvent) -> Option<serde_json::Value> {
        if let (Some(request), Some(response)) = (&event.request, &event.response) {
            Some(serde_json::json!({
                "request": {
                    "method": request.method,
                    "url": request.url,
                    "headers": request.headers,
                    "bodySize": request.body.as_ref().map_or(0, |b| b.len())
                },
                "response": {
                    "status": response.status_code,
                    "statusText": response.status_text,
                    "headers": response.headers,
                    "bodySize": response.size
                },
                "time": event.timing.as_ref().map_or(0.0, |t| t.total),
                "timings": event.timing.as_ref().map(|t| serde_json::json!({
                    "blocked": -1,
                    "dns": t.dns_lookup.unwrap_or(-1.0),
                    "connect": t.connect.unwrap_or(-1.0),
                    "send": 0,
                    "wait": t.ttfb.unwrap_or(-1.0),
                    "receive": t.download.unwrap_or(-1.0),
                    "ssl": t.ssl.unwrap_or(-1.0)
                }))
            }))
        } else {
            None
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            failed_requests: 0,
            total_bytes: 0,
            avg_response_time: 0.0,
            fastest_response: 0.0,
            slowest_response: 0.0,
            by_resource_type: HashMap::new(),
            by_status_code: HashMap::new(),
            top_domains: Vec::new(),
        }
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_monitor_creation() {
        let monitor = NetworkMonitor::new();
        assert!(monitor.enabled);
        assert_eq!(monitor.stats.total_requests, 0);
    }

    #[test]
    fn test_request_recording() {
        let mut monitor = NetworkMonitor::new();
        let request = NetworkRequest {
            id: "test_123".to_string(),
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: HashMap::new(),
            body: None,
            timestamp: 1234567890,
            initiator: "fetch".to_string(),
            resource_type: "xhr".to_string(),
            blocked: false,
        };

        monitor.record_request(request).unwrap();
        assert_eq!(monitor.stats.total_requests, 1);
        assert_eq!(monitor.requests.len(), 1);
    }
}