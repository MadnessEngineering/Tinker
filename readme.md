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

## Quick Start
- 🔧 Clone the repository
- 🛠️ Run `cargo build`
- 🚀 Start with `cargo run -- --url https://example.com`

See [Getting Started Guide](docs/getting-started.md) for detailed setup.

## Documentation
- [Contributing](CONTRIBUTING.md) - Join the guild! Includes detailed architecture guide
- [Changelog](CHANGELOG.md) - Project history and updates
- [Roadmap](ROADMAP.md) - Future development plans
- [Lessons Learned](LESSONS_LEARNED.md) - Engineering insights

## Project Status

🚧️ Currently In The Workshop - Not ready for production use

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

## License

This workshop is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Development Tools

### Screenshot Capture Tool
Located in `tools/capture.ps1`, this PowerShell script helps capture screenshots during development for UI feedback and documentation.

#### Prerequisites
- Greenshot must be installed (can be installed via `choco install greenshot`)
- PowerShell execution policy must allow running scripts

#### Usage
```powershell
# Capture active window
.\tools\capture.ps1 -type window -name "main-window"

# Capture specific region
.\tools\capture.ps1 -type region -name "toolbar-area"

# Capture full screen
.\tools\capture.ps1 -type full -name "full-browser"
```

#### Parameters
- `-type`: Type of screenshot to capture (window, region, or full)
- `-name`: Name for the screenshot file (defaults to timestamp if not provided)

#### Output
Screenshots are saved to the `screenshots/` directory in PNG format. The directory is automatically created if it doesn't exist.

Note: The `screenshots/` directory is ignored by git to avoid committing large binary files.
