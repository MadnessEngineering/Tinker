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
   - Remote Control Interface ✅
   - Test Assembly Line ✅
   - Quality Assurance Tools ✅
   - Visual Inspection System ✅

3. Event Workshop
   - MQTT Signal Tower ✅
   - Event Blueprint System ✅
   - Replay Engineering ✅
   - Timing Calibration Tools

4. **Advanced Testing Laboratory** 🆕
   - DOM Element Inspector ✅
   - JavaScript Injection Engine ✅
   - Network Traffic Monitor ✅
   - Visual Testing Suite ✅
   - Performance Analyzer 🚧
   - Debug Tools & Breakpoints 🚧

## Quick Start
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

### Real-time Control
- `WS /ws` - WebSocket for real-time events and control

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

See [MCP Server Documentation](docs/mcp-server.md) for complete details.

## Documentation
- [Contributing](CONTRIBUTING.md) - Join the guild! Includes detailed architecture guide
- [Changelog](CHANGELOG.md) - Project history and updates
- [Roadmap](ROADMAP.md) - Future development plans
- [Lessons Learned](LESSONS_LEARNED.md) - Engineering insights

## Project Status

🎉 **WORLD-CLASS BROWSER TESTING PLATFORM** - Major phases complete!

### ✅ Phase 1: Foundation (COMPLETE)
- **Core Engine**: Window & WebView creation with proper bounds
- **Tab Management**: Complete system with thread-safe state handling  
- **MQTT Events**: Full implementation with reconnection logic
- **CLI Interface**: Comprehensive argument parsing and configuration
- **Build System**: Compiles successfully and starts properly

### ✅ Phase 2: Visual Testing (COMPLETE)
- **Screenshot Capture**: Multi-format support (PNG, JPEG, WebP)
- **Visual Comparison**: Pixel-level difference analysis
- **Baseline Testing**: Create and compare against visual baselines
- **Image Processing**: Full pipeline for visual regression testing

### ✅ Phase 3: Advanced DOM & Network (COMPLETE)
- **DOM Element Inspector**: CSS/XPath/text selector support
- **Element Interaction**: Click, type, hover, scroll, drag operations
- **JavaScript Injection**: Full script execution capabilities
- **Wait Conditions**: Smart waiting for dynamic content
- **Network Monitoring**: Real-time request/response tracking
- **HAR Export**: Industry-standard network analysis format
- **Performance Statistics**: Request timing and analysis

### 🚧 Phase 4: Performance & Debugging (IN PROGRESS)
- **Performance Metrics**: Page load timing, resource analysis
- **Memory Profiling**: JavaScript heap and DOM analysis
- **Step-through Debugging**: Breakpoints and code inspection
- **CPU Performance**: JavaScript execution profiling

### 🔧 Quick Start Status
- `cargo build` ✅ Works perfectly
- `cargo run -- --url https://example.com` ✅ **BROWSER FULLY FUNCTIONAL!**
- `cargo run -- --api` ✅ **REST API SERVER READY!**
- Window creation ✅ Works with proper chrome
- Tab management ✅ Works with visual tab bar
- MQTT events ✅ Works with full event publishing
- WebView integration ✅ Both content view and tab bar working
- DOM inspection ✅ **Complete automation capabilities**
- Network monitoring ✅ **Real-time traffic analysis**
- Visual testing ✅ **Screenshot & comparison engine**

## Testing Scripts

Use the included Python test scripts to verify functionality:

```bash
# Test DOM inspection and JavaScript injection
python test_dom_simple.py

# Test network monitoring features  
python test_network_monitoring.py

# Test visual testing capabilities
python test_visual.py
```

## What's Next

### Immediate Priorities (Phase 4)
1. **Performance Analyzer**: JavaScript execution profiling, memory usage tracking
2. **Debug Tools**: Step-through debugging, breakpoint management
3. **Advanced Metrics**: Core Web Vitals, custom performance markers

### Future Enhancements
- Browser DevTools integration
- Multi-browser support (Firefox, Safari engines)
- Cloud testing infrastructure
- Plugin system for custom testing tools

## Capabilities Summary

🎯 **Element Automation**: Find, interact, and wait for DOM elements
🌐 **Network Analysis**: Monitor, filter, and export network traffic  
📸 **Visual Testing**: Screenshot capture and visual regression testing
⚡ **JavaScript Control**: Execute custom scripts and monitor execution
🔄 **Event Streaming**: Real-time MQTT event publishing
🌍 **Web Automation**: Complete browser control via REST API
📊 **Performance Monitoring**: Request timing and resource analysis
🔧 **Debugging Tools**: Element highlighting and inspection

**Tinker is now a production-ready browser testing platform!**

## License

This workshop is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.