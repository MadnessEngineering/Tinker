# Tinker

A Madness engineered browser built for tinkerers and test enthusiasts. Tinker treats web testing as a craft, combining precision mechanics with MQTT-based event handling to create the ultimate testing workbench.

## Features

### Core Features
- 🔧 MQTT-powered Control Mechanisms
- 🛠️ Universal Workbench API
- 📊 Built-in Diagnostic Dashboard
- 🔍 Test Blueprint Management
- 🔄 Session Recording and Replay
- 🎯 Precision Event Engineering

### Platform Support
- 🪟 Windows
  - Native window decorations
  - DPI awareness
  - Theme integration
  - Custom chrome
- 🍎 MacOS (Coming Soon)
  - Native controls
  - Touch bar support
  - System integrations

### JavaScript Engines
- V8 (Default)
- JavaScriptCore (Optional)
- SpiderMonkey (Optional)

## Architecture

### Core Components
1. Browser Engine
   - WebView Management
   - Tab Control System
   - Event Handling
   - State Management

2. Platform Layer
   - Abstract platform traits
   - Native window management
   - Theme integration
   - Platform-specific optimizations

3. JavaScript Integration
   - Pluggable engine system
   - Common interface
   - Memory management
   - Hook system

4. Event System
   - MQTT integration
   - Event recording
   - State synchronization
   - Diagnostic tools

## Quick Start

### Installation
```bash
# Clone the repository
git clone https://github.com/DanEdens/Tinker.git
cd Tinker

# Build with default features (V8 engine)
cargo build

# Build with all JavaScript engines
cargo build --features full

# Build with specific engine
cargo build --features javascriptcore
```

### Running
```bash
# Start with default URL
cargo run -- --url https://example.com

# Start in headless mode
cargo run -- --headless

# Start with debugging
cargo run -- --debug
```

## Configuration

### JavaScript Engine Selection
Select your preferred JavaScript engine through feature flags:
```toml
[dependencies]
tinker = { version = "0.1", features = ["v8"] }        # Use V8 (default)
tinker = { version = "0.1", features = ["full"] }      # All engines
tinker = { version = "0.1", features = ["spidermonkey"] } # SpiderMonkey only
```

### Platform-Specific Settings
Windows:
```rust
use tinker::{WindowsConfig, WindowsTheme};

let config = WindowsConfig {
    title: "My Browser".to_string(),
    theme: WindowsTheme::Dark,
    dpi_aware: true,
    ..Default::default()
};
```

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

## Development Guidelines

### Git Workflow
1. Branch Strategy
   - Use trunk-based development with feature branches
   - Branch naming: `feat/`, `fix/`, `docs/`, `refactor/` prefixes
   - Keep branches short-lived and focused

2. Commit Guidelines
   - Write detailed commit messages for future debugging reference
   - Include ticket/issue numbers when applicable
   - Use semantic commit types (feat, fix, docs, etc.)
   - Keep commits atomic and focused

3. Platform-Specific Notes
   Windows:
   - Use `git --no-pager` for commands that might trigger a pager
   - Example: `git --no-pager log`, `git --no-pager diff`
   
   Unix/Linux/MacOS:
   - Append `| cat` to commands that might trigger a pager
   - Example: `git log | cat`, `git diff | cat`

4. Command Execution
   - Avoid newlines in command strings
   - Use semicolons or && for multiple commands
   - Escape special characters appropriately
   - Set flags to prevent interactive prompts

### Issue Management
1. Creating Issues
   ```bash
   # Basic issue creation
   gh issue create --title "Issue Title" --body "Description"

   # With labels (must exist in repository)
   gh issue create --title "Issue Title" --body "Description" --label "bug" --label "help wanted"

   # From template (if available)
   gh issue create --template "bug_report.md"
   ```

2. Issue Guidelines
   - Use clear, descriptive titles
   - Include reproduction steps for bugs
   - Add relevant code snippets or error messages
   - Link related PRs or commits
   - Use markdown formatting for readability
   - Add appropriate labels once created

3. Issue Templates
   Located in `.github/ISSUE_TEMPLATE/`:
   - Bug report template
   - Feature request template
   - Documentation improvement template

4. Issue Labels
   Standard labels:
   - `bug`: Something isn't working
   - `enhancement`: New feature or request
   - `documentation`: Documentation improvements
   - `help wanted`: Extra attention needed
   - `good first issue`: Good for newcomers

### Lessons Learned
- Keep platform-specific code isolated in dedicated modules
- Use feature flags for optional functionality
- Maintain comprehensive error handling
- Document API changes and breaking changes
- Test across different platforms before merging

## Known Issues

### JavaScript Engine Integration
Currently experiencing platform-specific issues with JavaScript engine integration on Windows:

1. V8 Engine:
   - Build fails with precompiled binaries
   - Source compilation issues with build dependencies
   - Status: Investigating alternative build configurations

2. SpiderMonkey:
   - Build script issues with path handling
   - Compatibility issues with Windows paths
   - Status: Need to update build scripts for Windows paths

3. JavaScriptCore:
   - Missing pkg-config dependency on Windows
   - Requires glib system dependencies
   - Status: Investigating Windows-specific build requirements

### Workaround
Until these issues are resolved:
- JavaScript functionality is temporarily disabled on Windows
- Focus is on core browser functionality and Windows API integration
- JavaScript engine selection will be re-enabled once platform-specific issues are resolved

### Next Steps
1. Complete Windows API integration
2. Implement basic browser functionality
3. Revisit JavaScript engine integration with platform-specific builds
4. Add proper error handling for missing JavaScript functionality
