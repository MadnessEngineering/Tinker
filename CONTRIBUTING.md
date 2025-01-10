# Contributing to Tinker

Welcome to the Tinker Workshop! Whether you're a human craftsperson or an AI assistant, this guide will help you understand how to contribute to the project effectively.

## For Human Contributors

### Getting Started
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Message Guidelines
- Use semantic commit messages:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation changes
  - `test:` for test-related changes
  - `refactor:` for code refactoring
  - `style:` for formatting changes
  - `chore:` for maintenance tasks
- Use the present tense for commit messages

### Code Style
- Follow Rust style guidelines
- Use meaningful variable names
- Document public APIs
- Write tests for new features
- Keep functions focused and small

### Pull Request Process
1. Update documentation as needed
2. Add tests for new features
3. Update the CHANGELOG.md
4. Get review from maintainers

## For AI Assistants

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

### AI Assistant Guidelines
1. Code Changes {
   - Always use available tool functions
   - Follow semantic commit messages
   - Update tests for changes
   - Document new functionality
   - Handle errors gracefully wi
}

2. Documentation {
   - Keep knowledge graph updated
   - Document architectural decisions
   - Maintain clear component boundaries
   - Update relevant docs for changes
}

3. Testing {
   - Write unit tests for new features
   - Update integration tests
   - Document test scenarios
   - Handle edge cases
}
