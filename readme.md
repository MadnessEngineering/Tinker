# Tinker

A Madness engineered browser built for tinkerers and test enthusiasts. Tinker treats web testing as a craft, combining precision mechanics with MQTT-based event handling to create the ultimate testing workbench.

## Quick Start
- 🔧 Clone the repository
- 🛠️ Run `cargo build`
- 🚀 Start with `cargo run -- --url https://example.com`

See [Getting Started Guide](docs/getting-started.md) for detailed setup.

## Core Features
- 🔧 MQTT-powered Control Mechanisms
- 🛠️ Universal Testing Workbench API
- 📊 Built-in Diagnostic Dashboard
- 🔍 Test Blueprint Management
- 🔄 Reproducible Testing Patterns
- 🎯 Precision Event Engineering

## Documentation
- [Changelog](CHANGELOG.md) - Project history and updates
- [Roadmap](ROADMAP.md) - Future development plans
- [Lessons Learned](LESSONS_LEARNED.md) - Engineering insights
- [Contributing](CONTRIBUTING.md) - Join the guild!

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
}

3. TabManager {
   - type: "core_component"
   - responsibility: "Tab lifecycle management"
   - key_features: [
     "Tab creation/deletion",
     "Tab switching",
     "State persistence",
     "URL management"
   ]
   - state_pattern: "Arc<RwLock<TabState>>"
}

### Architecture Principles
1. ThreadSafety {
   - pattern: "Arc<RwLock<T>>"
   - usage: "All shared state"
   - rationale: "Prevent deadlocks, enable concurrent reads"
}

2. EventDriven {
   - pattern: "MQTT-based messaging"
   - scope: "All component communication"
   - benefits: [
     "Decoupled components",
     "Reproducible interactions",
     "Testable flows"
   ]
}

3. ComponentDesign {
   - pattern: "Independent state managers"
   - principle: "Single responsibility"
   - communication: "Message passing"
   - state: "Independently borrowable"
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
}

2. MQTT {
   - integration_type: "Event system"
   - usage: [
     "Command distribution",
     "Event recording",
     "State synchronization"
   ]
   - configuration: {
     "default_port": 3003,
     "protocol": "mqtt://"
   }
}

### Current Development Focus
1. EventSystem {
   - status: "in_progress"
   - priority: "high"
   - current_tasks: [
     "Keyboard event handling",
     "Window event management",
     "Event recording improvements"
   ]
}

2. TabManagement {
   - status: "stable"
   - recent_improvements: [
     "Thread-safe state handling",
     "Improved error handling",
     "Better UI feedback"
   ]
}

## Project Status

🚧️ Currently In The Workshop - Not ready for production use

## License

This workshop is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
