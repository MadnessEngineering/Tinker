### Cross-Platform Considerations

#### Windows
- Use WebView2 (modern, Chromium-based)
- Native window decorations through Windows API
- Better integration with system theme

#### macOS
- WKWebView for content
- Native window decorations through Cocoa
- Better integration with macOS conventions

#### Shared Components
- Common IPC layer
- Shared tab management logic
- Platform-agnostic UI components 