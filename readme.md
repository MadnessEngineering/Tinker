# Tinker

A Madness engineered browser built for tinkerers and test enthusiasts. Tinker treats web testing as a craft, combining precision mechanics with MQTT-based event handling to create the ultimate testing workbench.

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

### Component Workshop Design (New Architecture)

1. State Foundry (`StateManager`)
   - Tab State Forge
   - Window State Workshop
   - Configuration Anvil
   - Settings Storage

2. View Assembly Line (`ViewManager`)
   - WebView Factory
   - Window Chrome Shop
   - Tab UI Workshop
   - Layout Engineering

3. Event Signal Tower (`EventManager`)
   - MQTT Control Station
   - Event Recording Press
   - Replay Engineering Shop
   - Command Distribution Network

4. Input Control Station (`InputManager`)
   - Keyboard Command Factory
   - Mouse Event Workshop
   - Shortcut Engineering
   - Focus Control Unit

5. Navigation Bridge (`NavigationManager`)
   - URL Processing Plant
   - History Assembly Line
   - Redirect Control Unit
   - Security Checkpoint

Each component will:
- Be wrapped in Arc<RwLock<_>> for thread-safe access
- Have its own dedicated channel for command processing
- Maintain independent state that can be borrowed without blocking others
- Communicate through message passing rather than direct mutation


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

### Event System Architecture
- 🎯 Primary Goals:
  - Real-time event monitoring and visualization
  - MQTT-based remote control and automation
  - Event recording and replay for testing
  - Distributed system integration capabilities

- 🏗️ Core Components:
  1. Event Bus
     - Central event routing and distribution
     - Topic-based publish/subscribe system
     - Priority-based event handling
     - Event filtering and transformation

  2. MQTT Integration
     - Topic path mapping for browser events
     - Remote control command handling
     - Event broadcasting to external systems
     - Secure connection management

  3. Event Monitoring
     - Real-time event visualization
     - Event history with search/filter
     - Performance metrics collection
     - Debug logging integration

  4. Recording/Replay
     - Event capture with timing information
     - Deterministic replay capabilities
     - Session management and storage
     - Export/import functionality

- 🔄 Event Types:
  1. Browser Events
     - Navigation (URL changes, redirects)
     - Tab operations (create, close, switch)
     - Window state changes
     - Error conditions

  2. User Interactions
     - Mouse events (clicks, movement)
     - Keyboard input
     - Touch/gesture events
     - Form interactions

  3. System Events
     - Resource usage (memory, CPU)
     - Network activity
     - Plugin/extension events
     - Process lifecycle events

  4. Custom Events
     - Test automation commands
     - External system integration
     - Custom script events
     - Debug/profiling events

- 🛡️ Design Principles:
  - Asynchronous event processing
  - Thread-safe event handling
  - Minimal performance impact
  - Extensible event types
  - Reliable delivery guarantees
  - Clear error handling
  - Comprehensive monitoring
  - Secure event transmission


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
- Connection status messages help track client connectivity
- Status messages should include client ID, timestamp, and broker details
- Use QoS::AtLeastOnce for important status messages
- Always ensure MQTT URLs have the mqtt:// scheme prefix
- Provide fallback to localhost if URL parsing fails

### Component Integration Strategy
- 🔄 Event System as Core Infrastructure
  - Acts as central nervous system for all components
  - Provides foundational event types and processing
  - Other components build on top rather than replacing
  - Single source of truth for event data

- 🎯 Component Responsibilities
  1. Event System (Core Layer)
     - Event definition and transport
     - Basic recording and replay
     - Core monitoring capabilities
     - Performance metric collection

  2. Recording Workshop (Feature Layer)
     - Advanced recording scenarios
     - Test case generation
     - Scenario management
     - Editing and composition

  3. Debug Laboratory (Tool Layer)
     - Event visualization and analysis
     - Advanced debugging features
     - Network and DOM inspection
     - Performance profiling

  4. External Tool Bench (Integration Layer)
     - Protocol adaptation to MQTT
     - External tool coordination
     - CI/CD integration
     - Framework compatibility

- 🛠️ Integration Guidelines
  - Components should consume rather than duplicate
  - Use event system for all internal communication
  - Maintain clear layer separation
  - Follow established event patterns
  - Share common metric collection
  - Standardize on MQTT for external interfaces

### Code Organization
- Avoid circular dependencies by keeping related types in the same module
- Place commands and events together when they share common data structures
- Use public helper methods to encapsulate common state checks
- Keep template files in a dedicated directory with clear naming
- Maintain clear separation between UI and business logic

### Error Handling
- Convert WebView errors to String for consistent error handling
- Use map_err for error type conversion instead of custom match blocks
- Release locks before performing operations that might need them
- Handle all potential error cases in tab management
- Provide clear error messages for debugging
- Check lock acquisition success before using Mutex guards

### Thread Safety
- Use Arc<Mutex<T>> for shared state between threads
- Release locks as soon as possible to prevent deadlocks
- Clone command channels before moving into closures
- Ensure proper lock cleanup in error cases
- Use dedicated methods for checking state to avoid lock contention
