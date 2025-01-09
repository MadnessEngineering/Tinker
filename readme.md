# Tinker

A meticulously engineered browser built for tinkerers and test enthusiasts. Tinker treats web testing as a craft, combining precision mechanics with MQTT-based event handling to create the ultimate testing workbench.

## The Blueprint

Born in a workshop of web innovation, Tinker aims to reimagine browser testing through ingenious engineering. By building testing capabilities directly into the browser's machinery, we provide craftspeople with the tools they need for reliable, observable, and controllable web testing.

## Workshop Tools

- 🔧 MQTT-powered Control Mechanisms
- 🛠️ Universal Testing Workbench API
- 📊 Built-in Diagnostic Dashboard
- 🔍 Test Blueprint Management
- 📝 Automated Workshop Logs
- 🔄 Reproducible Testing Patterns
- 🎯 Precision Event Engineering

## Machine Architecture

### Core Machinery

1. Browser Engine Room
   - Custom WebView Forge
   - Event Capture & Replay Mechanisms
   - MQTT Control Panel Integration

2. Testing Workbench
   - Remote Control Interface
   - Test Assembly Line
   - Quality Assurance Tools
   - Visual Inspection System

3. Event Workshop
   - MQTT Signal Tower
   - Event Blueprint System
   - Replay Engineering
   - Timing Calibration Tools

4. Reporting Laboratory
   - Results Fabrication
   - Blueprint Generation
   - Metrics Workshop
   - Video Engineering Station

## Recent Achievements
- ✨ Implemented robust tab management system with thread-safe state handling
- 🔄 Created event monitoring system with fixed-size circular buffer
- 🌐 Integrated WebView with IPC communication for tab control
- 🚀 Added async operations support for API server and event system
- 🛡️ Implemented comprehensive error handling throughout the system
- 🧪 Added extensive test coverage for core functionality
- 📝 Added event recording and replay functionality
- 🎯 Implemented CLI interface with version support and descriptive help
- 🔧 Fixed test suite issues and improved test reliability
- 🎨 Added custom HTML/CSS menu bar with full keyboard shortcut support
- 🔄 Added environment variable configuration support
- 🎯 Improved tab creation with better error handling and UI feedback
- 🖼️ Fixed window chrome and native controls integration
- 🏗️ Improved tab bar construction with proper encapsulation
- 🔄 Enhanced window chrome integration with native controls

## Changelog

### January 5, 2024

#### Project Initialization
- 🎨 Rebranded project to Tinker with updated vision
- 📚 Transformed README into workshop-themed documentation
- 📜 Created CODE_OF_CONDUCT.md for community guidelines
- 🔧 Updated gitignore configuration for Cargo.lock

#### Core Development
- 🏗️ Forged initial Core Browser Engine
- ⚙️ Implemented navigation controls and tab management
- 🛠️ Improved browser engine implementation
- 🔧 Fixed compilation issues and code organization

#### Testing Infrastructure
- 🧪 Forged robust test infrastructure
- ✅ Created initial CLI framework with tests
- 📝 Updated README progress on Core Engine Assembly

### January 6, 2024

#### Project Foundation
- 🎨 Renamed project from testbrowser to tinker
- 📚 Updated README with keyboard controls progress
- 🔧 Cleaned up unused imports and variables

#### Core Features
- ⌨️ Implemented keyboard shortcuts for navigation and tab management
- 🌐 Added headless mode and URL navigation support
- 🎯 Added CLI arguments support
- 🔄 Improved cleanup handling and test behavior

#### Tab System and Event Monitoring
- 📊 Implemented tab system with TabManager
  * Create, close, and switch between tabs
  * Track active tab state
  * Prevent closing last tab
  * Update tab titles and URLs
- 👀 Added EventViewer for monitoring browser events
  * Chronological event logging with timestamps
  * Fixed-size circular buffer (1000 events)
  * Event filtering and retrieval methods
  * Memory-efficient using VecDeque

#### UI Implementation
- 🎨 Added tab UI with HTML/CSS/JS implementation
- 🔧 Fixed WebView and IPC handler issues in tab UI
- ✨ Implemented tab UI commands and event handling
- 🖼️ Updated window creation and event loop handling

#### Event System
- 📡 Implemented event signal tower with MQTT integration
- 🔄 Restored API server and event system integration
- 🛡️ Added rate limiting for MQTT error logging
- 🔧 Fixed MQTT client mutable reference issues
- 🎯 Improved tab closing logic and tests

### January 7, 2024

#### Core Architecture
- 🏗️ Refactored browser engine for improved architecture
- 🔒 Enhanced thread safety with Arc<Mutex> implementation
- 📝 Added command system for browser actions
- ✨ Improved IPC message handling with proper JSON parsing

#### Tab Management System
- 🎯 Implemented interactive tab management
- ➕ Added tab creation via '+' button
- ❌ Added tab closing with '×' button
- 🔄 Implemented tab switching with visual feedback
- 📝 Added tab title and URL update functionality
- 🔧 Fixed WebView initialization and tab UI handling

#### Testing and Documentation
- 🧪 Added tests for recording and replay features
- 📚 Updated README with current progress
- 🔍 Added detailed debug logging for tab operations

### January 8, 2024

#### Event System Improvements
- 🔄 Restored and enhanced MQTT event system functionality
- 📝 Improved event recording and replay with better save path handling
- 🛡️ Enhanced error handling in event system
- 🔧 Fixed event system initialization in BrowserEngine

#### UI and Window Management
- 🖼️ Fixed window layout and chrome view positioning
- 🎨 Improved tab bar visibility and WebView positioning
- 🏗️ Separated tab bar and content WebViews
- 🎯 Added proper window resize handling
- 🔧 Fixed WebView rendering issues with proper background colors
- 🚀 Improved tab switching and creation mechanics

#### Tab Management
- ✨ Enhanced tab management with better error handling
- 🛡️ Made TabBar fields public for better integration
- 🔄 Improved tab creation with UI feedback
- 🎯 Added get_active_tab_mut method to TabManager

#### Configuration and Environment
- 🔧 Moved DEFAULT_URL to environment variable
- 📝 Updated README with new lessons learned
- 🛡️ Added comprehensive event system tests

#### Code Quality and Documentation
- 🧹 Cleaned up menu-related templates and code
- 📚 Added JavaScript Engine integration plan
- 🔧 Fixed CLI tests with better version support and help text
- 📝 Updated documentation with lessons learned about tab bar construction

## Lessons Learned

### AI Pair Programming
- Test coverage drives AI decision making - untested features may be removed during refactoring
- Using git history in conversations helps AI understand and restore previously working code
- Maintaining a "Lessons Learned" section helps keep important details in focus during iterations
- When dealing with complex UI elements, instruct AI to comment out code instead of deleting
- Clean commits with clear messages improve AI's ability to reference past solutions
- Complex UI features require comprehensive test coverage to prevent unintended removal
- Git squashing can improve the usefulness of commit history in AI conversations

### WebView Management
- WebView instances should be owned by their respective tabs
- Strong references to parent windows must be maintained
- Proper display parameters are crucial for preventing frame issues
- IPC handlers need careful error handling and type checking
- Event handling should be bi-directional (UI ↔ Backend)
- UI state should be explicitly synced after backend operations
- Error handling should include user feedback
- WebView creation should be atomic (succeed completely or fail safely)
- Environment variables should be used for configurable defaults
- Tab bar construction should use proper encapsulation and initialization
- Window chrome integration requires careful coordination with native controls
- WebViews need explicit z-index management to prevent overlap issues
- Tab switching must properly handle WebView visibility and focus
- Content WebViews should be positioned relative to tab bar height
- Each tab's WebView should maintain its own state independently
- WebView cleanup must be explicit to prevent memory leaks
- Tab creation and closing operations need proper state synchronization
- Window resize events must update all WebView bounds correctly

### Menu System Design
- Native menu APIs can be inconsistent across platforms
- HTML/CSS menus provide better control and consistency
- WebView-based UI components need proper z-index management
- Keyboard shortcuts should be handled at both UI and system levels

### Event System Design
- Events should have clear success/failure feedback
- UI state must be kept in sync with backend state
- Error handling should be comprehensive and user-friendly
- Event handlers should be properly scoped and cleaned up
- Event propagation should be predictable and traceable

### JavaScript Engine Integration
- 🎯 Primary Focus: V8 Engine Integration
  - Industry standard JavaScript engine
  - Powers Chrome and Edge browsers
  - Extensive tooling and debugging capabilities

- 🔄 Future Engine Support:
  - SpiderMonkey (Firefox)
  - JavaScriptCore (Safari)

- 🏗️ Architecture Design:
  - Common interface layer for all engines
  - Engine-specific implementations isolated in modules
  - Unified manager for engine operations
  - Hot-swapping capability for testing different engines

- 🧪 Testing Considerations:
  - Performance benchmarking between engines
  - Compatibility testing across engines
  - Memory usage monitoring
  - Script execution timing analysis

- 📊 Metrics Collection:
  - Script execution time
  - Memory consumption
  - Garbage collection patterns
  - Error handling differences

- 🔍 Development Approach:
  1. Implement V8 integration first
  2. Add engine selection configuration
  3. Build performance monitoring
  4. Add remaining engines as needed
  5. Implement comparison tooling

## Engineering Roadmap

### Phase 1: Foundation Works
- [x] Core Engine Assembly
  - [x] Forge Wry WebView Components
  - [x] Engineer Navigation Controls
  - [x] Construct Tab Management
  - [x] Design Window Framework
  - [x] Wire Keyboard Controls

- [x] Event Engineering Station
  - [x] Construct MQTT Signal Tower
  - [x] Design Event Blueprints
  - [x] Engineer Data Patterns
  - [x] Build Capture Mechanisms
  - [x] Craft Replay Tools
  - [x] Install Signal Filters

- [ ] Control Panel Construction
  - [x] Forge API Control Points
  - [ ] Install WebSocket Machinery
  - [x] Build Security Mechanisms
  - [x] Draft Technical Schematics
  - [x] Craft Control Libraries
  - [x] Implement Environment Configuration

### Phase 2: Testing Machinery
- [ ] Quality Control Station
  - [x] Assemble Test Runner
  - [x] Craft Assertion Tools
  - [x] Design Test Blueprints
  - [ ] Build Parallel Testing Rig
  - [ ] Engineer Test Isolation Chamber

- [ ] Visual Inspection Tools
  - [ ] Construct Screenshot Apparatus
  - [ ] Build Comparison Engine
  - [ ] Create Difference Detector
  - [ ] Craft Multi-Scale Viewer
  - [ ] Engineer Element Inspector

- [x] State Engineering
  - [x] Build State Snapshot Tools
  - [x] Craft Restoration Machinery
  - [x] Engineer Data Storage
  - [x] Construct Cache Controls
  - [x] Design Session Workshop
  - [x] Implement Error Recovery

### Phase 3: Advanced Machinery
- [ ] Recording Workshop
  - [ ] Engineer Event Recorder
  - [ ] Build Test Generator
  - [ ] Craft Playback Controls
  - [ ] Design Scenario Branching
  - [ ] Construct Editing Station

- [ ] Debug Laboratory
  - [ ] Build Step Mechanism
  - [ ] Install Breakpoint System
  - [ ] Craft Network Inspector
  - [ ] Engineer DOM Workshop
  - [ ] Install Logging Station
  - [ ] Build Performance Analyzer

- [ ] Report Engineering
  - [ ] Craft Result Templates
  - [ ] Build Video Recorder
  - [ ] Engineer Metrics Station
  - [ ] Design Custom Blueprints
  - [ ] Construct Export Tools
  - [ ] Build Analytics Workshop

- [ ] JavaScript Engine Workshop
  - [ ] Design Engine Interface Layer
  - [ ] Implement V8 Integration
  - [ ] Engineer SpiderMonkey Support
  - [ ] Craft JavaScriptCore Bridge
  - [ ] Build Engine Performance Analyzer
  - [ ] Create Engine Switching Mechanism
  - [ ] Design Concurrent Testing Tools

### Phase 4: Integration Workshop
- [ ] External Tool Bench
  - [ ] Craft Selenium Bridge
  - [ ] Engineer Playwright Connection
  - [ ] Build CI/CD Tooling
  - [ ] Design Plugin Workshop
  - [ ] Craft Framework Adapters

- [ ] Performance Laboratory
  - [ ] Build Metrics Workshop
  - [ ] Craft Load Testing Rig
  - [ ] Engineer Baseline Tools
  - [ ] Design Regression Detector
  - [ ] Build Resource Monitor

- [ ] Security Workshop
  - [ ] Craft Security Scanner
  - [ ] Build SSL/TLS Validator
  - [ ] Engineer Policy Tester
  - [ ] Design XSS Detection Tools
  - [ ] Build CORS Test Station

### Phase 5: Workshop Management
- [ ] Technical Documentation
  - [ ] Draft API Schematics
  - [ ] Create Workshop Manuals
  - [ ] Design Test Blueprints
  - [ ] Write Troubleshooting Guides
  - [ ] Craft Integration Manuals

- [ ] Distribution Workshop
  - [ ] Build Package Assembly
  - [ ] Engineer Update System
  - [ ] Craft Crash Reporter
  - [ ] Design Health Monitor
  - [ ] Build Recovery Tools

- [ ] Craftsperson's Guild
  - [ ] Build Blueprint Exchange
  - [ ] Create Tool Repository
  - [ ] Draft Guild Documentation
  - [ ] Engineer Feedback Loop
  - [ ] Build Guild Hall

## Getting Started

(Workshop manual coming soon)

## Join the Guild

We welcome fellow craftspeople! See our [Guild Guidelines](CONTRIBUTING.md) for details.

## Workshop License

This workshop is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Project Status

🚧️ Currently In The Workshop - Not ready for production use

### Lessons Learned

#### MQTT Event System
- MQTT broker URL must be properly parsed to extract host and port
- Port should be taken from the URL or default to 1883
- Connection errors should be handled gracefully with reconnection attempts
- Event system should maintain its own state for reconnection handling
- Broker URL should be stored for debugging and reconnection purposes
