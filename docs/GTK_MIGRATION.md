# GTK Migration Plan for Tinker

## Overview
This document outlines the plan to migrate Tinker from wry WebView to GTK, incorporating lessons learned from EventGhost-Rust implementation.

## Architecture Changes

### 1. Window Management
- [ ] Replace `wry::WebView` with `gtk::ApplicationWindow`
- [ ] Implement window chrome using GTK widgets instead of HTML/CSS
- [ ] Migrate window configuration from wry-specific to GTK-specific settings
- [ ] Update platform-specific window handling (MacOS, Windows, Linux)

### 2. Tab System
- [ ] Replace WebView-based tab bar with `gtk::Notebook`
- [ ] Implement tab management using GTK's native tab container
- [ ] Create custom tab widgets for better control and styling
- [ ] Migrate tab state management to work with GTK widgets

### 3. Content Area
- [ ] Replace WebView content with appropriate GTK widgets
- [ ] Implement content area using `gtk::Box` or `gtk::Paned` for split views
- [ ] Add scrollable containers where needed
- [ ] Implement proper widget hierarchy for content organization

### 4. Event System
- [ ] Update event handling to use GTK's signal system
- [ ] Migrate browser events to GTK events
- [ ] Implement proper event propagation through widget hierarchy
- [ ] Update keyboard shortcut handling for GTK

## File Changes Required

### Core Changes
1. `src/browser/mod.rs`:
   - [ ] Replace WebView initialization with GTK setup
   - [ ] Update window creation and management
   - [ ] Modify event loop for GTK integration
   - [ ] Update content view handling

2. `src/platform/*.rs`:
   - [ ] Update platform-specific code for GTK
   - [ ] Remove WebView-specific implementations
   - [ ] Add GTK-specific window handling
   - [ ] Update theme handling for GTK

3. `src/browser/tab_ui.rs`:
   - [ ] Replace WebView-based tab UI with GTK widgets
   - [ ] Implement GTK-native tab controls
   - [ ] Update tab styling and behavior

4. `src/browser/tabs.rs`:
   - [ ] Update tab management for GTK widgets
   - [ ] Modify tab state handling
   - [ ] Implement GTK-specific tab operations

### New Files Needed
1. `src/widgets/`:
   - [ ] Create custom GTK widgets
   - [ ] Implement widget styling
   - [ ] Add widget behavior handlers

2. `src/ui/`:
   - [ ] Add UI layout management
   - [ ] Implement theme handling
   - [ ] Create widget factories

### Files to Remove
- [ ] `src/templates/window_chrome.html`
- [ ] `src/templates/window_chrome.js`
- [ ] `src/templates/tab_bar.html`
- [ ] `src/templates/tab_bar.js`

## Implementation Strategy

### Phase 1: Foundation
1. Set up GTK dependencies
2. Create basic window framework
3. Implement basic widget structure
4. Set up event system

### Phase 2: Core Features
1. Implement tab management
2. Add content area handling
3. Create custom widgets
4. Set up theme system

### Phase 3: Polish
1. Add animations and transitions
2. Implement proper styling
3. Add platform-specific optimizations
4. Implement accessibility features

## Testing Strategy

### Unit Tests
- [ ] Widget behavior tests
- [ ] Event handling tests
- [ ] State management tests
- [ ] Theme system tests

### Integration Tests
- [ ] Window management tests
- [ ] Tab system tests
- [ ] Content area tests
- [ ] Full application flow tests

## Dependencies to Add
```toml
[dependencies]
gtk = { version = "0.18", features = ["v4_12"] }
gtk4 = "0.18"
gio = "0.18"
glib = "0.18"
```

## Lessons Applied from EventGhost

### 1. Simplicity Over Complexity
- Keep widget hierarchy simple and logical
- Use direct event handling
- Maintain clear widget lifecycles

### 2. Resource Management
- Implement proper cleanup for GTK widgets
- Handle window and widget destruction properly
- Manage system resources efficiently

### 3. Cross-Platform Considerations
- Handle platform-specific GTK features appropriately
- Implement proper DPI scaling
- Support system themes
- Ensure consistent behavior across platforms

## Performance Considerations
1. Lazy widget loading
2. Efficient event handling
3. Resource pooling where appropriate
4. Proper cleanup and disposal

## Accessibility
1. Implement ARIA labels
2. Add keyboard navigation
3. Support screen readers
4. Handle high-contrast themes 