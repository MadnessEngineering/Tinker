# Tinker

A Madness engineered browser built for tinkerers and test enthusiasts. Tinker treats web testing as a craft, combining precision mechanics with MQTT-based event handling to create the ultimate testing workbench.

## The Blueprint

Born in a workshop of web innovation, Tinker aims to reimagine browser testing through ingenious engineering. By building testing capabilities directly into the browser's machinery, we provide craftspeople with the tools they need for reliable, observable, and controllable web testing.

## Quick Start
- 🔧 Clone the repository
- 🛠️ Run `cargo build`
- 🚀 Start with `cargo run -- --url https://example.com`

See [Getting Started Guide](docs/getting-started.md) for detailed setup.

## Workshop Tools
- 🔧 MQTT-powered Control Mechanisms
- 🛠️ Universal Workbench API as primary interface
- 📊 Built-in Diagnostic Dashboard
- 🔍 Test Blueprint Management, Pluginable
- 🔄 Customizable Session Versioning and Comparison
- 🔄 Reproducible Testing Patterns and replay tooling
- 🎯 Precision Event Engineering

## Documentation
- [Changelog](CHANGELOG.md) - Project history and updates
- [Roadmap](ROADMAP.md) - Future development plans
- [Lessons Learned](LESSONS_LEARNED.md) - Engineering insights
- [Contributing](CONTRIBUTING.md) - (TODO) Join the guild!

## AI Knowledge Graph

This section maintains a structured knowledge graph for AI assistants working on the project.

### Core Components
1. BrowserEngine {
   - type: "core_component"
   - responsibility: "Main browser orchestration"
   - key_features: [
     "WebView management",
     "Event handling",
     "Tab management",
     "Window management"
   ]
   - dependencies: [
     "wry",
     "tao",
     "tokio"
   ]
   - subcomponents: {
     "WebView Forge": [
       "Custom WebView implementation",
       "Event capture integration",
       "IPC communication"
     ],
     "Control Panel": [
       "MQTT integration",
       "Command routing",
       "State synchronization"
     ]
   }
}

2. EventSystem {
   - type: "core_component"
   - responsibility: "Event handling and distribution"
   - key_features: [
     "MQTT integration",
     "Event recording",
     "Event replay",
     "Command distribution"
   ]
   - state_management: "Uses Arc<RwLock<T>> for thread safety"
   - architecture: {
     "Event Bus": [
       "Central event routing",
       "Topic-based pub/sub",
       "Priority handling",
       "Event filtering"
     ],
     "MQTT Integration": [
       "Topic path mapping",
       "Remote control",
       "Event broadcasting",
       "Secure connections"
     ],
     "Event Monitoring": [
       "Real-time visualization",
       "History with search",
       "Performance metrics",
       "Debug logging"
     ],
     "Recording/Replay": [
       "Event capture with timing",
       "Deterministic replay",
       "Session management",
       "Export/import"
     ]
   }
   - event_types: {
     "Browser": [
       "Navigation",
       "Tab operations",
       "Window state",
       "Error conditions"
     ],
     "User": [
       "Mouse events",
       "Keyboard input",
       "Touch/gesture",
       "Form interactions"
     ],
     "System": [
       "Resource usage",
       "Network activity",
       "Plugin events",
       "Process lifecycle"
     ],
     "Custom": [
       "Test automation",
       "External integration",
       "Custom scripts",
       "Debug/profiling"
     ]
   }
}

3. StateFoundry {
   - type: "core_component"
   - responsibility: "State management and persistence"
   - subcomponents: {
     "Tab State": [
       "Active tab tracking",
       "Tab lifecycle",
       "Navigation state"
     ],
     "Window State": [
       "Size/position",
       "Focus handling",
       "Display settings"
     ],
     "Configuration": [
       "User settings",
       "Environment vars",
       "Runtime config"
     ]
   }
   - state_pattern: "Arc<RwLock<State>>"
}

4. ViewAssembly {
   - type: "core_component"
   - responsibility: "UI and layout management"
   - subcomponents: {
     "WebView Factory": [
       "View creation",
       "Content rendering",
       "Script injection"
     ],
     "Chrome Shop": [
       "Window decoration",
       "Control elements",
       "Native integration"
     ],
     "Layout Engine": [
       "Component positioning",
       "Resize handling",
       "Z-index management"
     ]
   }
}

5. InputControl {
   - type: "core_component"
   - responsibility: "Input handling and routing"
   - subcomponents: {
     "Keyboard": [
       "Keyboard event handling",
       "Text input",
       "Focus management",
       "shortcut handling"
     ],
     "Mouse": [
       "Click handling",
       "Drag operations",
       "Gesture support"
     ],
     "Clipboard": [
       "Clipboard reading",
       "Clipboard writing",
       "Clipboard history"
     ],
     "Commands": [
       "Action routing",
       "State updates",
       "Feedback handling"
     ]
   }
}

### Architecture Principles
1. ThreadSafety {
   - pattern: "Arc<RwLock<T>>"
   - usage: "All shared state"
   - rationale: "Prevent deadlocks, enable concurrent reads"
   - implementation: {
     "State Access": "Independent borrowing",
     "Channel Usage": "Dedicated per component",
     "Lock Strategy": "Minimize duration"
   }
}

2. EventDriven {
   - pattern: "MQTT-based messaging"
   - scope: "All component communication"
   - benefits: [
     "Decoupled components",
     "Reproducible interactions",
     "Testable flows",
     "Event driven architecture"
   ]
   - principles: [
     "Asynchronous processing",
     "Thread-safe handling",
     "Minimal impact",
     "Clear error paths"
   ]
}

3. ComponentDesign {
   - pattern: "Independent state managers"
   - principle: "Single responsibility"
   - communication: "Message passing"
   - state: "Independently borrowable"
   - guidelines: [
     "Consume don't duplicate",
     "Clear layer separation",
     "Standard patterns",
     "Metric collection"
   ]
}

### Integration Points
1. WebView {
   - integration_type: "Core browser component"
   - provider: "wry"
   - responsibilities: [
     "Web content rendering",
     "JavaScript execution",
     "DOM interaction"
   ]
   - js_engine: {
     "primary": "V8",
     "future": [
       "SpiderMonkey",
       "JavaScriptCore"
     ],
     "future features": [
       "Hot-swapping",
       "Performance monitoring",
       "Memory tracking"
     ]
   }
}

2. MQTT {
   - integration_type: "Event system"
   - usage: [
     "Command distribution",
     "Event recording",
     "State synchronization"
   ]
   - configuration: {
     "default_port": 1883,
     "protocol": "mqtt://"
   }
   - requirements: [
     "URL parsing with port extraction",
     "Graceful reconnection",
     "Status tracking",
     "QoS guarantees"
   ]
}

### Current Development Focus
1. EventSystem {
   - status: "in_progress"
   - priority: "high"
   - current_tasks: [
     "Keyboard/mouse event handling",
     "Window event management",
     "Event recording improvements",
   ]
   - future_tasks: [
     "Event replay improvements",
     "Event filtering",
     "Event monitoring improvements"
   ]
}

2. TabManagement {
   - status: "broken"
   - recent_improvements: [
     "Thread-safe state handling",
     "Improved error handling",
     "Better UI feedback"
   ]
   - issues: [
     "Tab switching issues",
     "State persistence issues",
     "URL management issues"
   ]
   - future_tasks: [
     "adding omnibar",
     "adding tabbar",
     "adding window chrome",
     "adding layout engine"
   ]
}

## Project Status

🚧️ Currently In The Workshop - Not ready for production use

## License

This workshop is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
