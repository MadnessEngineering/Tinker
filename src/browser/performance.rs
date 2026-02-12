//! Browser performance monitoring and metrics collection
//!
//! This module provides comprehensive performance tracking including:
//! - Core Web Vitals (LCP, FID, CLS, INP, TTFB)
//! - Navigation timing
//! - Resource timing
//! - JavaScript profiling
//! - Memory usage tracking

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use anyhow::Result;

/// Core Web Vitals metrics
///
/// See: https://web.dev/vitals/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreWebVitals {
    /// Largest Contentful Paint (LCP) - Loading performance
    /// Good: < 2.5s, Needs improvement: 2.5s - 4s, Poor: > 4s
    pub lcp: Option<f64>,

    /// First Input Delay (FID) - Interactivity
    /// Good: < 100ms, Needs improvement: 100ms - 300ms, Poor: > 300ms
    pub fid: Option<f64>,

    /// Cumulative Layout Shift (CLS) - Visual stability
    /// Good: < 0.1, Needs improvement: 0.1 - 0.25, Poor: > 0.25
    pub cls: Option<f64>,

    /// Interaction to Next Paint (INP) - Responsiveness
    /// Good: < 200ms, Needs improvement: 200ms - 500ms, Poor: > 500ms
    pub inp: Option<f64>,

    /// Time to First Byte (TTFB) - Server response time
    /// Good: < 800ms, Needs improvement: 800ms - 1800ms, Poor: > 1800ms
    pub ttfb: Option<f64>,

    /// First Contentful Paint (FCP)
    /// Good: < 1.8s, Needs improvement: 1.8s - 3s, Poor: > 3s
    pub fcp: Option<f64>,
}

impl CoreWebVitals {
    pub fn new() -> Self {
        Self {
            lcp: None,
            fid: None,
            cls: None,
            inp: None,
            ttfb: None,
            fcp: None,
        }
    }

    /// Check if all metrics are within "good" thresholds
    pub fn is_good(&self) -> bool {
        self.lcp.map_or(true, |v| v < 2500.0) &&
        self.fid.map_or(true, |v| v < 100.0) &&
        self.cls.map_or(true, |v| v < 0.1) &&
        self.inp.map_or(true, |v| v < 200.0) &&
        self.ttfb.map_or(true, |v| v < 800.0) &&
        self.fcp.map_or(true, |v| v < 1800.0)
    }

    /// Get score (0-100) based on thresholds
    pub fn get_score(&self) -> u8 {
        let mut score = 0;
        let mut count = 0;

        if let Some(lcp) = self.lcp {
            score += if lcp < 2500.0 { 100 } else if lcp < 4000.0 { 50 } else { 0 };
            count += 1;
        }

        if let Some(fid) = self.fid {
            score += if fid < 100.0 { 100 } else if fid < 300.0 { 50 } else { 0 };
            count += 1;
        }

        if let Some(cls) = self.cls {
            score += if cls < 0.1 { 100 } else if cls < 0.25 { 50 } else { 0 };
            count += 1;
        }

        if count > 0 {
            (score / count) as u8
        } else {
            0
        }
    }
}

impl Default for CoreWebVitals {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation timing metrics based on Navigation Timing API
///
/// See: https://www.w3.org/TR/navigation-timing-2/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTiming {
    /// Time when navigation started
    pub navigation_start: u64,

    /// Unload event timing
    pub unload_event_start: Option<u64>,
    pub unload_event_end: Option<u64>,

    /// Redirect timing
    pub redirect_start: Option<u64>,
    pub redirect_end: Option<u64>,
    pub redirect_count: u32,

    /// Fetch/cache timing
    pub fetch_start: u64,
    pub domain_lookup_start: u64,
    pub domain_lookup_end: u64,
    pub connect_start: u64,
    pub connect_end: u64,
    pub secure_connection_start: Option<u64>,

    /// Request/response timing
    pub request_start: u64,
    pub response_start: u64,
    pub response_end: u64,

    /// DOM processing timing
    pub dom_interactive: u64,
    pub dom_content_loaded_event_start: u64,
    pub dom_content_loaded_event_end: u64,
    pub dom_complete: u64,

    /// Load event timing
    pub load_event_start: u64,
    pub load_event_end: u64,
}

impl NavigationTiming {
    /// Calculate total page load time in milliseconds
    pub fn total_load_time(&self) -> u64 {
        self.load_event_end.saturating_sub(self.navigation_start)
    }

    /// Calculate DNS lookup time in milliseconds
    pub fn dns_time(&self) -> u64 {
        self.domain_lookup_end.saturating_sub(self.domain_lookup_start)
    }

    /// Calculate TCP connection time in milliseconds
    pub fn tcp_time(&self) -> u64 {
        self.connect_end.saturating_sub(self.connect_start)
    }

    /// Calculate request time in milliseconds
    pub fn request_time(&self) -> u64 {
        self.response_start.saturating_sub(self.request_start)
    }

    /// Calculate response time in milliseconds
    pub fn response_time(&self) -> u64 {
        self.response_end.saturating_sub(self.response_start)
    }

    /// Calculate DOM processing time in milliseconds
    pub fn dom_processing_time(&self) -> u64 {
        self.dom_complete.saturating_sub(self.dom_interactive)
    }
}

/// Resource timing entry for individual resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    /// Resource name/URL
    pub name: String,

    /// Resource type (script, stylesheet, image, etc.)
    pub initiator_type: String,

    /// Start time relative to navigation start
    pub start_time: f64,

    /// Duration of the resource load
    pub duration: f64,

    /// Detailed timing breakdown
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub connect_end: f64,
    pub secure_connection_start: Option<f64>,
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,

    /// Transfer size in bytes
    pub transfer_size: u64,

    /// Encoded body size in bytes
    pub encoded_body_size: u64,

    /// Decoded body size in bytes
    pub decoded_body_size: u64,

    /// Whether resource was loaded from cache
    pub from_cache: bool,
}

impl ResourceTiming {
    /// Calculate DNS lookup time
    pub fn dns_time(&self) -> f64 {
        self.domain_lookup_end - self.domain_lookup_start
    }

    /// Calculate TCP connection time
    pub fn tcp_time(&self) -> f64 {
        self.connect_end - self.connect_start
    }

    /// Calculate time to first byte
    pub fn ttfb(&self) -> f64 {
        self.response_start - self.request_start
    }

    /// Calculate download time
    pub fn download_time(&self) -> f64 {
        self.response_end - self.response_start
    }
}

/// JavaScript execution profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaScriptProfile {
    /// Profile ID
    pub id: String,

    /// Script URL or identifier
    pub script_url: String,

    /// Start time
    pub start_time: u64,

    /// End time
    pub end_time: u64,

    /// Total execution time in milliseconds
    pub duration: f64,

    /// Function call breakdown
    pub functions: Vec<FunctionProfile>,

    /// Total number of function calls
    pub call_count: u32,
}

impl JavaScriptProfile {
    pub fn new(id: String, script_url: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id,
            script_url,
            start_time: now,
            end_time: 0,
            duration: 0.0,
            functions: Vec::new(),
            call_count: 0,
        }
    }

    pub fn finish(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.end_time = now;
        self.duration = (now - self.start_time) as f64;
    }
}

/// Individual function execution profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionProfile {
    /// Function name
    pub name: String,

    /// File and line number
    pub location: String,

    /// Self time in milliseconds (excluding callees)
    pub self_time: f64,

    /// Total time in milliseconds (including callees)
    pub total_time: f64,

    /// Number of times called
    pub call_count: u32,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// JavaScript heap size limit in bytes
    pub js_heap_size_limit: u64,

    /// Total allocated JavaScript heap size in bytes
    pub total_js_heap_size: u64,

    /// Used JavaScript heap size in bytes
    pub used_js_heap_size: u64,

    /// Number of DOM nodes
    pub dom_node_count: u32,

    /// Number of event listeners
    pub event_listener_count: u32,

    /// Number of detached DOM nodes
    pub detached_node_count: u32,

    /// Timestamp when metrics were collected
    pub timestamp: u64,
}

impl MemoryMetrics {
    /// Calculate heap usage percentage
    pub fn heap_usage_percentage(&self) -> f64 {
        if self.js_heap_size_limit > 0 {
            (self.used_js_heap_size as f64 / self.js_heap_size_limit as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if memory usage is concerning (> 80% of limit)
    pub fn is_high_memory_usage(&self) -> bool {
        self.heap_usage_percentage() > 80.0
    }
}

/// Custom performance marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMarker {
    /// Marker name
    pub name: String,

    /// Timestamp when marker was created
    pub timestamp: f64,

    /// Optional additional data
    pub metadata: Option<HashMap<String, String>>,
}

/// Custom performance measure (between two markers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasure {
    /// Measure name
    pub name: String,

    /// Start marker name
    pub start_mark: String,

    /// End marker name
    pub end_mark: String,

    /// Duration in milliseconds
    pub duration: f64,

    /// Timestamp when measure was created
    pub timestamp: f64,
}

/// Performance monitor state and metrics collector
pub struct PerformanceMonitor {
    /// Whether monitoring is active
    monitoring: bool,

    /// Core Web Vitals metrics
    core_web_vitals: CoreWebVitals,

    /// Navigation timing
    navigation_timing: Option<NavigationTiming>,

    /// Resource timing entries
    resource_timings: Vec<ResourceTiming>,

    /// JavaScript profiles
    js_profiles: Vec<JavaScriptProfile>,

    /// Current active profile
    active_profile: Option<JavaScriptProfile>,

    /// Memory snapshots
    memory_snapshots: Vec<MemoryMetrics>,

    /// Custom markers
    markers: Vec<PerformanceMarker>,

    /// Custom measures
    measures: Vec<PerformanceMeasure>,

    /// Maximum number of resource entries to keep
    max_resource_entries: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            monitoring: false,
            core_web_vitals: CoreWebVitals::new(),
            navigation_timing: None,
            resource_timings: Vec::new(),
            js_profiles: Vec::new(),
            active_profile: None,
            memory_snapshots: Vec::new(),
            markers: Vec::new(),
            measures: Vec::new(),
            max_resource_entries: 500,
        }
    }

    /// Start performance monitoring
    pub fn start_monitoring(&mut self) {
        info!("Starting performance monitoring");
        self.monitoring = true;
        self.reset();
    }

    /// Stop performance monitoring
    pub fn stop_monitoring(&mut self) {
        info!("Stopping performance monitoring");
        self.monitoring = false;
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.monitoring
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        debug!("Resetting performance metrics");
        self.core_web_vitals = CoreWebVitals::new();
        self.navigation_timing = None;
        self.resource_timings.clear();
        self.js_profiles.clear();
        self.active_profile = None;
        self.memory_snapshots.clear();
        self.markers.clear();
        self.measures.clear();
    }

    /// Update Core Web Vitals
    pub fn update_core_web_vitals(&mut self, vitals: CoreWebVitals) {
        debug!("Updating Core Web Vitals: {:?}", vitals);
        self.core_web_vitals = vitals;
    }

    /// Set navigation timing
    pub fn set_navigation_timing(&mut self, timing: NavigationTiming) {
        debug!("Setting navigation timing");
        self.navigation_timing = Some(timing);
    }

    /// Add resource timing entry
    pub fn add_resource_timing(&mut self, timing: ResourceTiming) {
        debug!("Adding resource timing for: {}", timing.name);

        // Keep only the most recent entries
        if self.resource_timings.len() >= self.max_resource_entries {
            self.resource_timings.remove(0);
        }

        self.resource_timings.push(timing);
    }

    /// Start JavaScript profiling
    pub fn start_js_profile(&mut self, script_url: String) -> String {
        let id = format!("profile_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis());

        info!("Starting JavaScript profile: {} for {}", id, script_url);
        let profile = JavaScriptProfile::new(id.clone(), script_url);
        self.active_profile = Some(profile);

        id
    }

    /// Stop JavaScript profiling
    pub fn stop_js_profile(&mut self) -> Option<JavaScriptProfile> {
        if let Some(mut profile) = self.active_profile.take() {
            profile.finish();
            info!("Stopped JavaScript profile: {} ({}ms)",
                  profile.id, profile.duration);
            self.js_profiles.push(profile.clone());
            Some(profile)
        } else {
            warn!("No active JavaScript profile to stop");
            None
        }
    }

    /// Add memory snapshot
    pub fn add_memory_snapshot(&mut self, metrics: MemoryMetrics) {
        debug!("Adding memory snapshot: {} MB used",
               metrics.used_js_heap_size / 1024 / 1024);

        if metrics.is_high_memory_usage() {
            warn!("High memory usage detected: {:.1}%",
                  metrics.heap_usage_percentage());
        }

        self.memory_snapshots.push(metrics);
    }

    /// Add custom marker
    pub fn add_marker(&mut self, name: String, metadata: Option<HashMap<String, String>>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        debug!("Adding performance marker: {}", name);
        self.markers.push(PerformanceMarker {
            name,
            timestamp,
            metadata,
        });
    }

    /// Add custom measure between two markers
    pub fn add_measure(&mut self, name: String, start_mark: String, end_mark: String) -> Result<()> {
        let start = self.markers.iter()
            .find(|m| m.name == start_mark)
            .ok_or_else(|| anyhow::anyhow!("Start marker '{}' not found", start_mark))?;

        let end = self.markers.iter()
            .find(|m| m.name == end_mark)
            .ok_or_else(|| anyhow::anyhow!("End marker '{}' not found", end_mark))?;

        let duration = end.timestamp - start.timestamp;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        debug!("Adding performance measure: {} ({}ms)", name, duration);
        self.measures.push(PerformanceMeasure {
            name,
            start_mark,
            end_mark,
            duration,
            timestamp,
        });

        Ok(())
    }

    /// Get all metrics as a summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            core_web_vitals: self.core_web_vitals.clone(),
            navigation_timing: self.navigation_timing.clone(),
            resource_count: self.resource_timings.len(),
            total_resource_size: self.resource_timings.iter()
                .map(|r| r.transfer_size)
                .sum(),
            js_profile_count: self.js_profiles.len(),
            memory_snapshots: self.memory_snapshots.len(),
            latest_memory: self.memory_snapshots.last().cloned(),
            marker_count: self.markers.len(),
            measure_count: self.measures.len(),
        }
    }

    /// Get Core Web Vitals
    pub fn get_core_web_vitals(&self) -> &CoreWebVitals {
        &self.core_web_vitals
    }

    /// Get navigation timing
    pub fn get_navigation_timing(&self) -> Option<&NavigationTiming> {
        self.navigation_timing.as_ref()
    }

    /// Get all resource timings
    pub fn get_resource_timings(&self) -> &[ResourceTiming] {
        &self.resource_timings
    }

    /// Get resource timings by type
    pub fn get_resource_timings_by_type(&self, resource_type: &str) -> Vec<&ResourceTiming> {
        self.resource_timings.iter()
            .filter(|r| r.initiator_type == resource_type)
            .collect()
    }

    /// Get all JavaScript profiles
    pub fn get_js_profiles(&self) -> &[JavaScriptProfile] {
        &self.js_profiles
    }

    /// Get latest memory snapshot
    pub fn get_latest_memory(&self) -> Option<&MemoryMetrics> {
        self.memory_snapshots.last()
    }

    /// Get all memory snapshots
    pub fn get_memory_snapshots(&self) -> &[MemoryMetrics] {
        &self.memory_snapshots
    }

    /// Get all markers
    pub fn get_markers(&self) -> &[PerformanceMarker] {
        &self.markers
    }

    /// Get all measures
    pub fn get_measures(&self) -> &[PerformanceMeasure] {
        &self.measures
    }

    /// Generate JavaScript code to collect performance metrics
    pub fn generate_collection_script(&self) -> String {
        include_str!("../../templates/performance_collection.js").to_string()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance summary for quick overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub core_web_vitals: CoreWebVitals,
    pub navigation_timing: Option<NavigationTiming>,
    pub resource_count: usize,
    pub total_resource_size: u64,
    pub js_profile_count: usize,
    pub memory_snapshots: usize,
    pub latest_memory: Option<MemoryMetrics>,
    pub marker_count: usize,
    pub measure_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_web_vitals_score() {
        let mut vitals = CoreWebVitals::new();
        vitals.lcp = Some(2000.0);  // Good
        vitals.fid = Some(80.0);     // Good
        vitals.cls = Some(0.05);     // Good

        assert!(vitals.is_good());
        assert_eq!(vitals.get_score(), 100);
    }

    #[test]
    fn test_core_web_vitals_poor_score() {
        let mut vitals = CoreWebVitals::new();
        vitals.lcp = Some(5000.0);  // Poor
        vitals.fid = Some(400.0);    // Poor
        vitals.cls = Some(0.5);      // Poor

        assert!(!vitals.is_good());
        assert_eq!(vitals.get_score(), 0);
    }

    #[test]
    fn test_memory_metrics_percentage() {
        let metrics = MemoryMetrics {
            js_heap_size_limit: 1000,
            total_js_heap_size: 900,
            used_js_heap_size: 850,
            dom_node_count: 100,
            event_listener_count: 50,
            detached_node_count: 5,
            timestamp: 0,
        };

        assert_eq!(metrics.heap_usage_percentage(), 85.0);
        assert!(metrics.is_high_memory_usage());
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        assert!(!monitor.is_monitoring());

        monitor.start_monitoring();
        assert!(monitor.is_monitoring());

        monitor.add_marker("start".to_string(), None);
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.add_marker("end".to_string(), None);

        let result = monitor.add_measure(
            "test_measure".to_string(),
            "start".to_string(),
            "end".to_string()
        );

        assert!(result.is_ok());
        assert_eq!(monitor.get_measures().len(), 1);
    }

    #[test]
    fn test_js_profiling() {
        let mut monitor = PerformanceMonitor::new();

        let profile_id = monitor.start_js_profile("test.js".to_string());
        assert!(monitor.active_profile.is_some());

        std::thread::sleep(std::time::Duration::from_millis(10));

        let profile = monitor.stop_js_profile();
        assert!(profile.is_some());
        assert!(profile.unwrap().duration > 0.0);
        assert_eq!(monitor.get_js_profiles().len(), 1);
    }
}
