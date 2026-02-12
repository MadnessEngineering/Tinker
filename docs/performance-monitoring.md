# Performance Monitoring Guide

## Overview

Tinker's Performance Monitoring Laboratory provides comprehensive tools for measuring and analyzing web application performance. This guide covers Core Web Vitals, navigation timing, resource analysis, JavaScript profiling, and memory tracking.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core Web Vitals](#core-web-vitals)
3. [Navigation Timing](#navigation-timing)
4. [Resource Timing](#resource-timing)
5. [Memory Profiling](#memory-profiling)
6. [JavaScript Profiling](#javascript-profiling)
7. [Custom Markers & Measures](#custom-markers--measures)
8. [REST API](#rest-api)
9. [MCP Integration](#mcp-integration)
10. [Best Practices](#best-practices)

---

## Quick Start

### Starting Performance Monitoring

```bash
# Start the browser with API server
cargo run -- --api --url https://example.com
```

### Basic Usage via REST API

```bash
# Start monitoring
curl -X POST http://localhost:3003/api/performance/start

# Get Core Web Vitals
curl http://localhost:3003/api/performance/core-web-vitals

# Get memory metrics
curl http://localhost:3003/api/performance/memory

# Stop monitoring
curl -X POST http://localhost:3003/api/performance/stop
```

---

## Core Web Vitals

Core Web Vitals are Google's standardized metrics for measuring user experience.

### Metrics

| Metric | Description | Good | Needs Improvement | Poor |
|--------|-------------|------|-------------------|------|
| **LCP** | Largest Contentful Paint - Loading performance | < 2.5s | 2.5s - 4s | > 4s |
| **FID** | First Input Delay - Interactivity | < 100ms | 100ms - 300ms | > 300ms |
| **CLS** | Cumulative Layout Shift - Visual stability | < 0.1 | 0.1 - 0.25 | > 0.25 |
| **INP** | Interaction to Next Paint - Responsiveness | < 200ms | 200ms - 500ms | > 500ms |
| **TTFB** | Time to First Byte - Server response | < 800ms | 800ms - 1.8s | > 1.8s |
| **FCP** | First Contentful Paint | < 1.8s | 1.8s - 3s | > 3s |

### Getting Core Web Vitals

```bash
curl http://localhost:3003/api/performance/core-web-vitals
```

Response:
```json
{
  "success": true,
  "data": {
    "lcp": 2100.5,
    "fid": 85.2,
    "cls": 0.08,
    "inp": 180.0,
    "ttfb": 650.0,
    "fcp": 1600.0
  }
}
```

### Interpreting Results

The system automatically calculates a **0-100 score** based on thresholds:

- **100**: All metrics in "good" range
- **50**: Metrics in "needs improvement" range
- **0**: Metrics in "poor" range

### Example: Analyzing Core Web Vitals

```python
import requests

response = requests.get('http://localhost:3003/api/performance/core-web-vitals')
vitals = response.json()['data']

# Check if site meets "good" thresholds
if (vitals['lcp'] < 2500 and
    vitals['fid'] < 100 and
    vitals['cls'] < 0.1):
    print("✅ Site passes Core Web Vitals!")
else:
    print("❌ Site needs optimization")

    if vitals['lcp'] >= 2500:
        print(f"  - LCP too slow: {vitals['lcp']}ms")
    if vitals['fid'] >= 100:
        print(f"  - FID too high: {vitals['fid']}ms")
    if vitals['cls'] >= 0.1:
        print(f"  - CLS too high: {vitals['cls']}")
```

---

## Navigation Timing

Navigation Timing provides detailed breakdown of page load phases.

### Timeline Breakdown

```
Navigation Start
    ↓
DNS Lookup (10ms)
    ↓
TCP Connection (20ms)
    ↓
TLS Handshake (30ms) [if HTTPS]
    ↓
Request Sent (1ms)
    ↓
Waiting (TTFB) (100ms)
    ↓
Response Download (50ms)
    ↓
DOM Processing (150ms)
    ↓
Load Complete (320ms total)
```

### Collecting Navigation Timing

```bash
curl http://localhost:3003/api/performance/metrics
```

Response includes:
```json
{
  "navigation_timing": {
    "navigation_start": 0,
    "domain_lookup_start": 10,
    "domain_lookup_end": 20,
    "connect_start": 20,
    "connect_end": 40,
    "secure_connection_start": 40,
    "request_start": 70,
    "response_start": 170,
    "response_end": 220,
    "dom_interactive": 370,
    "dom_content_loaded_event_start": 420,
    "dom_complete": 470,
    "load_event_end": 500
  }
}
```

### Calculating Key Metrics

```python
def analyze_navigation_timing(timing):
    """Calculate key performance metrics from navigation timing"""

    metrics = {}

    # DNS lookup time
    metrics['dns_time'] = timing['domain_lookup_end'] - timing['domain_lookup_start']

    # TCP connection time
    metrics['tcp_time'] = timing['connect_end'] - timing['connect_start']

    # Time to First Byte
    metrics['ttfb'] = timing['response_start'] - timing['request_start']

    # Response download time
    metrics['download_time'] = timing['response_end'] - timing['response_start']

    # DOM processing time
    metrics['dom_processing'] = timing['dom_complete'] - timing['dom_interactive']

    # Total load time
    metrics['total_load'] = timing['load_event_end'] - timing['navigation_start']

    return metrics

# Usage
response = requests.get('http://localhost:3003/api/performance/metrics')
nav_timing = response.json()['data']['navigation_timing']
metrics = analyze_navigation_timing(nav_timing)

print(f"DNS Lookup: {metrics['dns_time']}ms")
print(f"TCP Connection: {metrics['tcp_time']}ms")
print(f"TTFB: {metrics['ttfb']}ms")
print(f"Download: {metrics['download_time']}ms")
print(f"Total Load: {metrics['total_load']}ms")
```

---

## Resource Timing

Resource Timing tracks individual resource loads (scripts, stylesheets, images, fonts).

### Collecting Resource Timing

```bash
curl http://localhost:3003/api/performance/metrics
```

Response includes:
```json
{
  "resource_timings": [
    {
      "name": "https://example.com/app.js",
      "initiator_type": "script",
      "start_time": 100.5,
      "duration": 250.3,
      "dns_time": 5.0,
      "tcp_time": 20.0,
      "ttfb": 70.0,
      "download_time": 150.0,
      "transfer_size": 50000,
      "encoded_body_size": 48000,
      "decoded_body_size": 150000,
      "from_cache": false
    }
  ]
}
```

### Resource Types

- `script` - JavaScript files
- `stylesheet` - CSS files
- `image` - Images (PNG, JPEG, etc.)
- `font` - Web fonts
- `xmlhttprequest` - AJAX requests
- `fetch` - Fetch API calls
- `document` - HTML documents

### Analyzing Resource Performance

```python
def analyze_resources(resources):
    """Analyze resource loading performance"""

    by_type = {}
    slow_resources = []
    cached_count = 0

    for resource in resources:
        # Group by type
        res_type = resource['initiator_type']
        if res_type not in by_type:
            by_type[res_type] = []
        by_type[res_type].append(resource)

        # Find slow resources (> 1 second)
        if resource['duration'] > 1000:
            slow_resources.append({
                'name': resource['name'],
                'duration': resource['duration'],
                'size': resource['transfer_size']
            })

        # Count cached resources
        if resource['from_cache']:
            cached_count += 1

    # Calculate statistics
    total_size = sum(r['transfer_size'] for r in resources)
    total_time = sum(r['duration'] for r in resources)

    print(f"Total Resources: {len(resources)}")
    print(f"Total Size: {total_size / 1024:.2f} KB")
    print(f"Cached: {cached_count} ({cached_count/len(resources)*100:.1f}%)")
    print(f"\nBy Type:")
    for res_type, items in by_type.items():
        print(f"  {res_type}: {len(items)}")

    if slow_resources:
        print(f"\n⚠️  Slow Resources (> 1s):")
        for resource in slow_resources:
            print(f"  - {resource['name']}: {resource['duration']}ms ({resource['size']/1024:.1f}KB)")
```

---

## Memory Profiling

Track JavaScript heap usage, DOM nodes, and detect memory leaks.

### Getting Memory Metrics

```bash
curl http://localhost:3003/api/performance/memory
```

Response:
```json
{
  "js_heap_size_limit": 2197815296,
  "total_js_heap_size": 50000000,
  "used_js_heap_size": 30000000,
  "dom_node_count": 500,
  "event_listener_count": 120,
  "detached_node_count": 5,
  "timestamp": 1234567890
}
```

### Memory Metrics Explained

| Metric | Description |
|--------|-------------|
| `js_heap_size_limit` | Maximum heap size available |
| `total_js_heap_size` | Total allocated heap |
| `used_js_heap_size` | Currently used heap |
| `dom_node_count` | Number of DOM nodes |
| `event_listener_count` | Number of event listeners |
| `detached_node_count` | Detached nodes (potential leak) |

### Detecting Memory Issues

```python
def check_memory_health(memory):
    """Check for memory-related issues"""

    issues = []

    # Calculate heap usage percentage
    heap_usage = (memory['used_js_heap_size'] / memory['js_heap_size_limit']) * 100

    if heap_usage > 80:
        issues.append(f"⚠️  High heap usage: {heap_usage:.1f}%")

    # Check for detached nodes (memory leaks)
    if memory['detached_node_count'] > 50:
        issues.append(f"⚠️  High detached nodes: {memory['detached_node_count']}")

    # Check for excessive event listeners
    listeners_per_node = memory['event_listener_count'] / max(memory['dom_node_count'], 1)
    if listeners_per_node > 5:
        issues.append(f"⚠️  Too many listeners per node: {listeners_per_node:.1f}")

    if issues:
        print("Memory Issues Detected:")
        for issue in issues:
            print(f"  {issue}")
        return False
    else:
        print(f"✅ Memory healthy: {heap_usage:.1f}% used")
        return True
```

### Memory Leak Detection Pattern

```python
import time

def detect_memory_leak(url, duration_seconds=60, check_interval=10):
    """Monitor memory over time to detect leaks"""

    snapshots = []
    start_time = time.time()

    while time.time() - start_time < duration_seconds:
        response = requests.get('http://localhost:3003/api/performance/memory')
        memory = response.json()['data']

        snapshots.append({
            'timestamp': time.time(),
            'used_heap': memory['used_js_heap_size'],
            'dom_nodes': memory['dom_node_count'],
            'detached_nodes': memory['detached_node_count']
        })

        time.sleep(check_interval)

    # Analyze trend
    heap_growth = snapshots[-1]['used_heap'] - snapshots[0]['used_heap']
    node_growth = snapshots[-1]['dom_nodes'] - snapshots[0]['dom_nodes']

    if heap_growth > 50_000_000:  # 50MB growth
        print(f"⚠️  Potential memory leak detected!")
        print(f"  Heap grew by {heap_growth / 1024 / 1024:.2f}MB")
        print(f"  DOM nodes grew by {node_growth}")
    else:
        print("✅ No memory leak detected")
```

---

## JavaScript Profiling

Profile JavaScript execution to identify performance bottlenecks.

### Starting a Profile

```bash
curl -X POST http://localhost:3003/api/performance/profiling/start \
  -H "Content-Type: application/json" \
  -d '{"script_url": "https://example.com/app.js"}'
```

### Stopping a Profile

```bash
curl -X POST http://localhost:3003/api/performance/profiling/stop
```

Response:
```json
{
  "id": "profile_1234567890",
  "script_url": "https://example.com/app.js",
  "start_time": 1000,
  "end_time": 1500,
  "duration": 500.0,
  "functions": [
    {
      "name": "processData",
      "location": "app.js:45",
      "self_time": 50.0,
      "total_time": 150.0,
      "call_count": 10
    }
  ],
  "call_count": 0
}
```

### Analyzing Profile Results

```python
def analyze_profile(profile):
    """Analyze JavaScript profile for bottlenecks"""

    print(f"Profile: {profile['script_url']}")
    print(f"Duration: {profile['duration']}ms")
    print(f"\nFunction Breakdown:")

    # Sort by total time
    functions = sorted(profile['functions'],
                      key=lambda f: f['total_time'],
                      reverse=True)

    for func in functions[:10]:  # Top 10
        print(f"\n  {func['name']} ({func['location']})")
        print(f"    Total: {func['total_time']}ms")
        print(f"    Self: {func['self_time']}ms")
        print(f"    Calls: {func['call_count']}")

        # Calculate percentage of total execution
        pct = (func['total_time'] / profile['duration']) * 100
        print(f"    {pct:.1f}% of total execution")

        # Warn if function is a bottleneck
        if pct > 20:
            print(f"    ⚠️  BOTTLENECK - Consider optimizing")
```

---

## Custom Markers & Measures

Add custom timing markers and measures to track specific operations.

### Adding a Marker

```bash
curl -X POST http://localhost:3003/api/performance/marker \
  -H "Content-Type: application/json" \
  -d '{
    "name": "user_action_start",
    "metadata": {
      "action": "button_click",
      "element_id": "submit_button"
    }
  }'
```

### Adding a Measure

```bash
# Mark start
curl -X POST http://localhost:3003/api/performance/marker \
  -d '{"name": "api_call_start"}'

# ... do work ...

# Mark end
curl -X POST http://localhost:3003/api/performance/marker \
  -d '{"name": "api_call_end"}'

# Create measure
curl -X POST http://localhost:3003/api/performance/measure \
  -H "Content-Type: application/json" \
  -d '{
    "name": "api_call_duration",
    "start_mark": "api_call_start",
    "end_mark": "api_call_end"
  }'
```

### Use Cases

**User Interaction Tracking:**
```python
# Mark user action
requests.post('http://localhost:3003/api/performance/marker',
             json={'name': 'search_initiated'})

# Perform search...

requests.post('http://localhost:3003/api/performance/marker',
             json={'name': 'search_complete'})

requests.post('http://localhost:3003/api/performance/measure',
             json={
                 'name': 'search_duration',
                 'start_mark': 'search_initiated',
                 'end_mark': 'search_complete'
             })
```

**API Call Tracking:**
```python
requests.post('http://localhost:3003/api/performance/marker',
             json={'name': 'api_start', 'metadata': {'endpoint': '/api/users'}})

# Make API call...

requests.post('http://localhost:3003/api/performance/marker',
             json={'name': 'api_end'})

requests.post('http://localhost:3003/api/performance/measure',
             json={
                 'name': 'api_latency',
                 'start_mark': 'api_start',
                 'end_mark': 'api_end'
             })
```

---

## REST API

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/performance/start` | POST | Start monitoring |
| `/api/performance/stop` | POST | Stop monitoring |
| `/api/performance/metrics` | GET | Get all metrics |
| `/api/performance/core-web-vitals` | GET | Get Core Web Vitals |
| `/api/performance/memory` | GET | Get memory metrics |
| `/api/performance/summary` | GET | Get performance summary |
| `/api/performance/profiling/start` | POST | Start JS profiling |
| `/api/performance/profiling/stop` | POST | Stop JS profiling |
| `/api/performance/marker` | POST | Add performance marker |
| `/api/performance/measure` | POST | Add performance measure |

### Complete Example

```python
import requests
import time

BASE_URL = 'http://localhost:3003/api/performance'

# 1. Start monitoring
requests.post(f'{BASE_URL}/start')

# 2. Wait for page to load
time.sleep(5)

# 3. Collect Core Web Vitals
vitals = requests.get(f'{BASE_URL}/core-web-vitals').json()['data']
print(f"LCP: {vitals['lcp']}ms")
print(f"FID: {vitals['fid']}ms")
print(f"CLS: {vitals['cls']}")

# 4. Check memory
memory = requests.get(f'{BASE_URL}/memory').json()['data']
heap_pct = (memory['used_js_heap_size'] / memory['js_heap_size_limit']) * 100
print(f"Memory Usage: {heap_pct:.1f}%")

# 5. Get full summary
summary = requests.get(f'{BASE_URL}/summary').json()['data']
print(f"\nPerformance Summary:")
print(f"  Resources Loaded: {summary['resource_count']}")
print(f"  Total Size: {summary['total_resource_size'] / 1024:.2f} KB")

# 6. Stop monitoring
requests.post(f'{BASE_URL}/stop')
```

---

## MCP Integration

The MCP server includes performance monitoring tools for AI agents.

### Available Tools

- `start_performance_monitoring` - Start monitoring
- `stop_performance_monitoring` - Stop monitoring
- `get_core_web_vitals` - Get Core Web Vitals
- `get_memory_metrics` - Get memory usage
- `get_performance_summary` - Get full summary
- `start_js_profiling` - Start JS profiling
- `stop_js_profiling` - Stop JS profiling
- `add_performance_marker` - Add custom marker
- `add_performance_measure` - Add custom measure

### Example MCP Usage

```bash
# Start browser with MCP
cargo run -- --mcp --url https://example.com
```

Then in Claude Desktop, you can ask:
- "Start performance monitoring and analyze the Core Web Vitals"
- "Check the memory usage of this page"
- "Profile the JavaScript execution"

---

## Best Practices

### 1. Establish Baselines

```python
def establish_baseline(url, runs=5):
    """Run multiple tests to establish performance baseline"""

    results = []

    for i in range(runs):
        # Navigate to URL
        requests.post('http://localhost:3003/api/navigate', json={'url': url})
        time.sleep(3)

        # Collect metrics
        vitals = requests.get('http://localhost:3003/api/performance/core-web-vitals').json()['data']
        results.append(vitals)

    # Calculate averages
    baseline = {
        'lcp': sum(r['lcp'] for r in results) / runs,
        'fid': sum(r['fid'] for r in results) / runs,
        'cls': sum(r['cls'] for r in results) / runs
    }

    return baseline
```

### 2. Monitor Continuously

Set up automated monitoring:

```python
import schedule

def check_performance():
    vitals = requests.get('http://localhost:3003/api/performance/core-web-vitals').json()['data']
    memory = requests.get('http://localhost:3003/api/performance/memory').json()['data']

    # Log to monitoring system
    log_metrics({
        'lcp': vitals['lcp'],
        'memory_usage': memory['used_js_heap_size']
    })

# Check every 5 minutes
schedule.every(5).minutes.do(check_performance)
```

### 3. Test Different Scenarios

```python
scenarios = [
    {'name': 'Desktop', 'viewport': '1920x1080', 'throttle': '4g'},
    {'name': 'Mobile', 'viewport': '375x667', 'throttle': '3g'},
    {'name': 'Slow Connection', 'viewport': '1920x1080', 'throttle': '2g'}
]

for scenario in scenarios:
    print(f"\nTesting {scenario['name']}...")
    # Configure and test...
```

### 4. Set Performance Budgets

```python
PERFORMANCE_BUDGET = {
    'lcp': 2500,  # ms
    'fid': 100,   # ms
    'cls': 0.1,
    'total_js_size': 300_000,  # bytes
    'total_css_size': 100_000   # bytes
}

def check_budget(vitals, resources):
    """Check if performance is within budget"""

    violations = []

    if vitals['lcp'] > PERFORMANCE_BUDGET['lcp']:
        violations.append(f"LCP: {vitals['lcp']}ms > {PERFORMANCE_BUDGET['lcp']}ms")

    js_size = sum(r['transfer_size'] for r in resources if r['initiator_type'] == 'script')
    if js_size > PERFORMANCE_BUDGET['total_js_size']:
        violations.append(f"JS Size: {js_size} > {PERFORMANCE_BUDGET['total_js_size']}")

    if violations:
        print("❌ Performance budget violated:")
        for v in violations:
            print(f"  - {v}")
        return False

    print("✅ Performance within budget")
    return True
```

### 5. Correlate with User Experience

```python
def performance_score_to_grade(score):
    """Convert performance score to letter grade"""
    if score >= 90:
        return 'A'
    elif score >= 75:
        return 'B'
    elif score >= 60:
        return 'C'
    elif score >= 50:
        return 'D'
    else:
        return 'F'

# Get vitals and calculate score
vitals = requests.get('http://localhost:3003/api/performance/core-web-vitals').json()['data']
# Calculate score based on your criteria
score = calculate_score(vitals)
grade = performance_score_to_grade(score)

print(f"Performance Grade: {grade} ({score}/100)")
```

---

## Troubleshooting

### High LCP

**Causes:**
- Large images not optimized
- Render-blocking CSS/JS
- Slow server response

**Solutions:**
- Optimize images (WebP, lazy loading)
- Inline critical CSS
- Use CDN for static assets

### High FID

**Causes:**
- Long-running JavaScript
- Heavy event handlers
- Main thread blocking

**Solutions:**
- Split code into smaller chunks
- Use Web Workers for heavy computation
- Debounce/throttle event handlers

### High CLS

**Causes:**
- Images without dimensions
- Dynamic content insertion
- Web fonts causing reflow

**Solutions:**
- Set explicit width/height on images
- Reserve space for dynamic content
- Use font-display: swap

### High Memory Usage

**Causes:**
- Memory leaks (detached nodes)
- Too many event listeners
- Large data structures in memory

**Solutions:**
- Remove event listeners when done
- Clean up DOM nodes properly
- Use WeakMap/WeakSet for caches

---

## Additional Resources

- [Web Vitals Documentation](https://web.dev/vitals/)
- [Navigation Timing API](https://www.w3.org/TR/navigation-timing-2/)
- [Resource Timing API](https://www.w3.org/TR/resource-timing-2/)
- [Performance API](https://developer.mozilla.org/en-US/docs/Web/API/Performance)
- [Chrome DevTools Performance](https://developer.chrome.com/docs/devtools/performance/)

---

## Summary

Tinker's Performance Monitoring provides:

✅ **Core Web Vitals** - Industry-standard metrics
✅ **Navigation Timing** - Detailed load breakdown
✅ **Resource Timing** - Per-resource analysis
✅ **Memory Profiling** - Leak detection
✅ **JS Profiling** - Bottleneck identification
✅ **Custom Markers** - Application-specific tracking
✅ **REST API** - Easy integration
✅ **MCP Support** - AI agent control

Use these tools to build faster, more reliable web applications!
