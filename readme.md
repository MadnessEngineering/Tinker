# Tinker

A Madness engineered browser built for tinkerers and test enthusiasts. Tinker treats web testing as a craft, combining precision mechanics with MQTT-based event handling to create the ultimate testing workbench.

## The Blueprint

Born in a workshop of web innovation, Tinker aims to reimagine browser testing through ingenious engineering. By building testing capabilities directly into the browser's machinery, we provide craftspeople with the tools they need for reliable, observable, and controllable web testing.

## Workshop Tools
- 🔧 MQTT-powered Control Mechanisms
- 🛠️ Universal Workbench API as primary interface
- 📊 Built-in Diagnostic Dashboard
- 🔍 Test Blueprint Management, Pluginable
- 🔄 Customizable Session Versioning and Comparison
- 🔄 Reproducible Testing Patterns and replay tooling
- 🎯 Precision Event Engineering
- 🌐 **Network Traffic Analysis & Monitoring**
- 🎨 **Visual Testing & Screenshot Comparison**
- 🧩 **DOM Element Inspector & Interaction**
- ⚡ **JavaScript Injection & Execution**

## Machine Architecture

### Core Machinery
1. Browser Engine Room
   - Custom WebView Forge ✅
   - Event Capture & Replay Mechanisms ✅
   - MQTT Control Panel Integration ✅

2. Testing Workbench
   - Remote Control Interface ✅ (REST, WebSocket, MQTT, MCP)
   - Test Assembly Line 🚧 (recording works; test generation doesn't)
   - Quality Assurance Tools ✅
   - Visual Inspection System ✅

3. Event Workshop
   - MQTT Signal Tower ✅
   - Event Blueprint System ✅
   - Replay Engineering ✅ (seek, step, speed, loop)
   - Timing Calibration Tools ✅

4. **Advanced Testing Laboratory** 🆕
   - DOM Element Inspector ✅
   - JavaScript Injection Engine ✅
   - Network Traffic Monitor ✅
   - Visual Testing Suite ✅
   - Console Monitor ✅
   - Performance Analyzer ✅ (Core Web Vitals, memory, JS profiling)
   - Step Debugging ✅ (via replay); Breakpoints ❌

## Quick Start

> **First build?** Install the native dependencies first — GTK and WebKitGTK
> headers on Linux. They aren't in `Cargo.toml` and a cold clone fails without
> them. See [Getting Started](docs/getting-started.md).

- 🔧 Clone the repository
- 🛠️ Run `cargo build`
- 🚀 Start with `cargo run -- --url https://example.com`
- 🌐 Start with API: `cargo run -- --api` (runs on http://127.0.0.1:3003)
- 🤖 Start with MCP: `cargo run -- --mcp` (Model Context Protocol for AI agents)

See [Getting Started Guide](docs/getting-started.md) for detailed setup.

## API Endpoints

Tinker provides a comprehensive REST API for automation and testing:

### Core Browser Control
- `POST /api/navigate` - Navigate to URL
- `POST /api/tabs` - Create new tab
- `DELETE /api/tabs/{id}` - Close tab
- `POST /api/tabs/{id}/activate` - Switch to tab

### Visual Testing
- `POST /api/screenshot` - Capture screenshot
- `POST /api/visual/baseline` - Create visual baseline
- `POST /api/visual/test` - Run visual regression test

### DOM Inspection & Interaction
- `POST /api/element/find` - Find elements with CSS/XPath/text selectors
- `POST /api/element/interact` - Click, type, hover, scroll elements
- `POST /api/element/highlight` - Highlight elements for debugging
- `POST /api/element/wait` - Wait for element conditions
- `POST /api/javascript/execute` - Execute JavaScript in page
- `GET /api/page/info` - Extract page information

### Network Monitoring
- `POST /api/network/start` - Start network monitoring
- `POST /api/network/stop` - Stop network monitoring
- `GET /api/network/stats` - Get network statistics
- `GET /api/network/export` - Export HAR file
- `POST /api/network/filter` - Add network filters
- `POST /api/network/clear-filters` - Clear all filters

### Performance
- `POST /api/performance/start` - Start performance monitoring
- `POST /api/performance/stop` - Stop performance monitoring
- `GET /api/performance/metrics` - Collect current metrics
- `GET /api/performance/core-web-vitals` - LCP, FID, CLS, INP, TTFB, FCP
- `GET /api/performance/memory` - Heap, DOM nodes, listeners
- `GET /api/performance/summary` - Aggregate summary
- `POST /api/performance/profiling/start` - Start JS profiling
- `POST /api/performance/profiling/stop` - Stop JS profiling
- `POST /api/performance/marker` - Add a custom marker
- `POST /api/performance/measure` - Measure between two markers

### Console
- `POST /api/console/start` - Start console monitoring
- `POST /api/console/stop` - Stop console monitoring
- `GET /api/console/logs` - Retrieve captured logs
- `POST /api/console/clear` - Clear the log buffer
- `POST /api/console/filter` - Filter by level

### Recording & Playback
- `POST /api/recording/start` - Start recording a session
- `POST /api/recording/stop` - Stop recording
- `POST /api/recording/pause` / `resume` - Pause and resume
- `POST /api/recording/save` / `load` - Persist and restore recordings
- `POST /api/recording/assertion` - Attach an expected-state assertion
- `POST /api/recording/snapshots` - Enable periodic snapshots
- `POST /api/playback/start` / `stop` / `pause` / `resume` - Playback control
- `POST /api/playback/seek` - Seek to a timestamp
- `POST /api/playback/step/forward` / `backward` - Step through events
- `POST /api/playback/speed` - Set playback speed
- `POST /api/playback/loop` - Toggle looping
- `GET /api/playback/state` - Current playback state

### Real-time Control
- `WS /ws` - WebSocket for real-time events and control
- `GET /health` - Health check
- `GET /api/info` - Browser and capability info

## MCP Server (AI Agent Control)

Tinker includes a Model Context Protocol (MCP) server that allows AI agents like Claude to directly control the browser. The MCP server implements JSON-RPC 2.0 over stdio.

### Quick Start

```bash
# Start in MCP mode
cargo run -- --mcp --url https://example.com

# Or combine with API server
cargo run -- --mcp --api --url https://example.com
```

### Claude Desktop Integration

Add to your Claude Desktop configuration:

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

Then ask Claude to control the browser:
- "Navigate to rust-lang.org and take a screenshot"
- "Find the search button and click it"
- "Execute JavaScript to get the page title"

### Available MCP Tools

- **Navigation**: navigate, create_tab, close_tab, switch_tab
- **Visual Testing**: take_screenshot, create_visual_baseline, run_visual_test
- **DOM Interaction**: find_element, click_element, type_text
- **JavaScript**: execute_javascript, get_page_info
- **Network**: start_network_monitoring, stop_network_monitoring, get_network_stats, export_network_har
- **Console**: start_console_monitoring, stop_console_monitoring, get_console_logs, clear_console_logs
- **Performance**: start_performance_monitoring, stop_performance_monitoring, get_core_web_vitals, get_memory_metrics, get_performance_summary

See [MCP Server Documentation](docs/mcp-server.md) for complete details.

## Documentation
- [Getting Started](docs/getting-started.md) - Install, build, run
- [Contributing](CONTRIBUTING.md) - Join the guild! Includes detailed architecture guide
- [Changelog](CHANGELOG.md) - Project history and updates
- [Roadmap](ROADMAP.md) - Future development plans
- [Lessons Learned](LESSONS_LEARNED.md) - Engineering insights

## Project Status

Tinker works and is useful, with the caveats below. Status here is kept
honest against the code — see the [roadmap](ROADMAP.md) for a per-module
breakdown citing implementing files and test counts.

**Verified**: August 22, 2026 · ~70 browser commands · `cargo test` → 164 passed, 3 ignored

### What works

- **Core engine** — window and WebView creation, tabs, navigation with
  per-tab history, session persistence across launches
- **Control surfaces** — MQTT event tower, REST API, WebSocket at `/ws`,
  and an MCP server for AI agents
- **DOM automation** — CSS/XPath/text selectors, click/type/hover/scroll,
  wait conditions, element highlighting, JavaScript execution
- **Visual testing** — screenshot capture, baselines, pixel-level diffing
- **Network monitoring** — request/response capture, filters, HAR export
- **Console monitoring** — capture and filtering by level
- **Performance** — Core Web Vitals, navigation and resource timing,
  memory metrics, JavaScript profiling, custom marks and measures
- **Recording and replay** — record sessions, replay with seek, step
  forward/back, speed control, and looping

### Known gaps

- **No CI.** The suite passes but nothing runs it automatically, on any
  platform. This is the next thing being fixed.
- **Cross-platform is unproven.** Development has been macOS-centric. Windows
  and Linux builds are expected to work but aren't regularly exercised.
- **Recordings can't become tests.** You can record and replay a session, but
  not generate a committable test from it.
- **No reporting.** Results surface as logs and API responses; there's no
  report or export format.
- **Keyboard input isn't exposed.** `browser/keyboard.rs` handles shortcuts
  internally but isn't reachable over the API or MCP, so keyboard-driven
  testing (tab order, accessibility) isn't scriptable yet.
- **Recording/replay isn't exposed over MCP.** Reachable over REST only, so an
  agent can't record or replay its own session.
- **Assertions are minimal.** Recordings can store expected state, but there's
  no authoring UX and no pass/fail surfacing.

### Getting it running

`cargo build` needs system libraries that aren't in `Cargo.toml` — GTK and
WebKitGTK development headers on Linux. A cold clone fails without them, in a
dependency, with an error that doesn't name the fix. See
[Getting Started](docs/getting-started.md).

## Testing

```bash
cargo test
```

The Rust suites need no running browser. For manual smoke tests against a live
instance, see [`tests/integration/README.md`](tests/integration/README.md).

## License

This workshop is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.