# Migration to iced Framework

## Analysis of Current Projects

### 1. GTK Implementation (Current)
**Strengths:**
- Native widget system
- Tab management implemented
- Context menu system
- Basic web functionality

**Challenges:**
- WebKit dependency issues on Windows
- Complex widget hierarchy
- Platform-specific quirks
- Heavy dependency footprint

### 2. Browser Module (Original)
**Reusable Components:**
- Tab state management (`tabs.rs`)
- Navigation logic (`navigation.rs`)
- Keyboard handling (`keyboard.rs`)
- Event system (`event_viewer.rs`)

### 3. EventGhost-Rust Project
**Learnings:**
- Clean architecture separation
- Event handling patterns
- Resource management
- Platform integration

## Migration Strategy

### Phase 1: Foundation
1. **State Management**
```rust
pub struct TinkerState {
    // Tab Management
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    
    // UI State
    theme: Theme,
    window_size: Size,
    
    // Navigation
    navigation_history: Vec<String>,
    current_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Tab Actions
    NewTab,
    CloseTab(usize),
    SwitchTab(usize),
    DuplicateTab(usize),
    
    // Navigation
    Navigate(String),
    GoBack,
    GoForward,
    Reload,
    
    // Window Actions
    Resize(Size),
    ToggleTheme,
}
```

### Phase 2: UI Components
1. **Tab Bar**
   - Custom tab widget with close button
   - Tab overflow handling
   - Drag and drop support

2. **Content Area**
   - Multiple content type support
   - Split view capability
   - Scrolling and zoom

3. **Navigation Bar**
   - URL input
   - Navigation buttons
   - Progress indicator

### Phase 3: Features
1. **Web Integration**
   - Consider alternatives:
     - System webview
     - Embedded browser engines
     - Pure HTML renderer

2. **Extensions**
   - Plugin system
   - Custom content handlers
   - Theme support

### Phase 4: Polish
1. **Performance**
   - Lazy loading
   - Resource caching
   - Memory management

2. **User Experience**
   - Keyboard shortcuts
   - Animations
   - Accessibility

## Implementation Plan

### Step 1: Basic Structure
1. Set up iced dependencies
2. Create main application state
3. Implement basic message system
4. Create window layout

### Step 2: Tab System
1. Port tab management logic
2. Create tab UI components
3. Implement tab interactions
4. Add context menu

### Step 3: Content Handling
1. Design content trait
2. Implement basic content types
3. Add web content support
4. Create split view system

### Step 4: Navigation
1. Port navigation logic
2. Create navigation UI
3. Implement history management
4. Add bookmarks

## Dependencies
```toml
[dependencies]
iced = { version = "0.10", features = ["tokio", "debug"] }
iced_native = "0.10"
iced_style = "0.9"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Next Steps
1. Create initial project structure
2. Port state management
3. Build basic UI components
4. Migrate tab system

Would you like to proceed with any particular aspect of this plan? 