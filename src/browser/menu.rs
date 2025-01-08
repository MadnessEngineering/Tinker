use tao::menu::{MenuBar, MenuId, MenuItem};
use tao::window::Window;
use std::sync::mpsc::Sender;
use tracing::debug;

#[derive(Debug)]
pub enum MenuCommand {
    NewTab,
    NewWindow,
    CloseTab,
    CloseWindow,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleDevTools,
    StartRecording,
    StopRecording,
    RunTests,
    ViewTestReport,
    SwitchEngine(JsEngine),
    OpenJsConsole,
    OpenPerformanceMonitor,
    OpenDocumentation,
    ReportIssue,
    About,
}

#[derive(Debug)]
pub enum JsEngine {
    V8,
    SpiderMonkey,
    JavaScriptCore,
}

pub fn create_application_menu(window: &Window, command_tx: Sender<MenuCommand>) -> MenuBar {
    let mut menu_bar = MenuBar::new();
    
    // File Menu
    let mut file_menu = MenuBar::new();
    let tx = command_tx.clone();
    file_menu.add_item(MenuId::new("new_tab"), "New Tab")
        .set_accelerator("CmdOrCtrl+T")
        .set_action(move || {
            debug!("Menu action: New Tab");
            let _ = tx.send(MenuCommand::NewTab);
        });

    let tx = command_tx.clone();
    file_menu.add_item(MenuId::new("new_window"), "New Window")
        .set_accelerator("CmdOrCtrl+N")
        .set_action(move || {
            debug!("Menu action: New Window");
            let _ = tx.send(MenuCommand::NewWindow);
        });

    file_menu.add_native_item(MenuItem::Separator);

    let tx = command_tx.clone();
    file_menu.add_item(MenuId::new("close_tab"), "Close Tab")
        .set_accelerator("CmdOrCtrl+W")
        .set_action(move || {
            debug!("Menu action: Close Tab");
            let _ = tx.send(MenuCommand::CloseTab);
        });

    let tx = command_tx.clone();
    file_menu.add_item(MenuId::new("close_window"), "Close Window")
        .set_accelerator("CmdOrCtrl+Shift+W")
        .set_action(move || {
            debug!("Menu action: Close Window");
            let _ = tx.send(MenuCommand::CloseWindow);
        });

    menu_bar.add_submenu("File", true, file_menu);

    // Edit Menu
    let mut edit_menu = MenuBar::new();
    edit_menu.add_native_item(MenuItem::Undo);
    edit_menu.add_native_item(MenuItem::Redo);
    edit_menu.add_native_item(MenuItem::Separator);
    edit_menu.add_native_item(MenuItem::Cut);
    edit_menu.add_native_item(MenuItem::Copy);
    edit_menu.add_native_item(MenuItem::Paste);
    edit_menu.add_native_item(MenuItem::SelectAll);
    menu_bar.add_submenu("Edit", true, edit_menu);

    // View Menu
    let mut view_menu = MenuBar::new();
    let tx = command_tx.clone();
    view_menu.add_item(MenuId::new("zoom_in"), "Zoom In")
        .set_accelerator("CmdOrCtrl+Plus")
        .set_action(move || {
            debug!("Menu action: Zoom In");
            let _ = tx.send(MenuCommand::ZoomIn);
        });

    let tx = command_tx.clone();
    view_menu.add_item(MenuId::new("zoom_out"), "Zoom Out")
        .set_accelerator("CmdOrCtrl+Minus")
        .set_action(move || {
            debug!("Menu action: Zoom Out");
            let _ = tx.send(MenuCommand::ZoomOut);
        });

    let tx = command_tx.clone();
    view_menu.add_item(MenuId::new("zoom_reset"), "Reset Zoom")
        .set_accelerator("CmdOrCtrl+0")
        .set_action(move || {
            debug!("Menu action: Reset Zoom");
            let _ = tx.send(MenuCommand::ZoomReset);
        });

    view_menu.add_native_item(MenuItem::Separator);

    let tx = command_tx.clone();
    view_menu.add_item(MenuId::new("toggle_devtools"), "Toggle Developer Tools")
        .set_accelerator("CmdOrCtrl+Shift+I")
        .set_action(move || {
            debug!("Menu action: Toggle DevTools");
            let _ = tx.send(MenuCommand::ToggleDevTools);
        });

    menu_bar.add_submenu("View", true, view_menu);

    // Testing Menu
    let mut testing_menu = MenuBar::new();
    let tx = command_tx.clone();
    testing_menu.add_item(MenuId::new("record_test"), "Start Recording Test")
        .set_action(move || {
            debug!("Menu action: Start Recording Test");
            let _ = tx.send(MenuCommand::StartRecording);
        });

    let tx = command_tx.clone();
    testing_menu.add_item(MenuId::new("stop_recording"), "Stop Recording")
        .set_action(move || {
            debug!("Menu action: Stop Recording");
            let _ = tx.send(MenuCommand::StopRecording);
        });

    testing_menu.add_native_item(MenuItem::Separator);

    let tx = command_tx.clone();
    testing_menu.add_item(MenuId::new("run_tests"), "Run Tests")
        .set_action(move || {
            debug!("Menu action: Run Tests");
            let _ = tx.send(MenuCommand::RunTests);
        });

    let tx = command_tx.clone();
    testing_menu.add_item(MenuId::new("test_report"), "View Test Report")
        .set_action(move || {
            debug!("Menu action: View Test Report");
            let _ = tx.send(MenuCommand::ViewTestReport);
        });

    menu_bar.add_submenu("Testing", true, testing_menu);

    // JavaScript Menu
    let mut js_menu = MenuBar::new();
    let tx = command_tx.clone();
    js_menu.add_item(MenuId::new("engine_v8"), "V8 Engine")
        .set_action(move || {
            debug!("Menu action: Switch to V8");
            let _ = tx.send(MenuCommand::SwitchEngine(JsEngine::V8));
        });

    let tx = command_tx.clone();
    js_menu.add_item(MenuId::new("engine_spidermonkey"), "SpiderMonkey Engine")
        .set_action(move || {
            debug!("Menu action: Switch to SpiderMonkey");
            let _ = tx.send(MenuCommand::SwitchEngine(JsEngine::SpiderMonkey));
        });

    let tx = command_tx.clone();
    js_menu.add_item(MenuId::new("engine_javascriptcore"), "JavaScriptCore Engine")
        .set_action(move || {
            debug!("Menu action: Switch to JavaScriptCore");
            let _ = tx.send(MenuCommand::SwitchEngine(JsEngine::JavaScriptCore));
        });

    js_menu.add_native_item(MenuItem::Separator);

    let tx = command_tx.clone();
    js_menu.add_item(MenuId::new("js_console"), "JavaScript Console")
        .set_action(move || {
            debug!("Menu action: Open JS Console");
            let _ = tx.send(MenuCommand::OpenJsConsole);
        });

    let tx = command_tx.clone();
    js_menu.add_item(MenuId::new("js_performance"), "Performance Monitor")
        .set_action(move || {
            debug!("Menu action: Open Performance Monitor");
            let _ = tx.send(MenuCommand::OpenPerformanceMonitor);
        });

    menu_bar.add_submenu("JavaScript", true, js_menu);

    // Help Menu
    let mut help_menu = MenuBar::new();
    let tx = command_tx.clone();
    help_menu.add_item(MenuId::new("documentation"), "Documentation")
        .set_action(move || {
            debug!("Menu action: Open Documentation");
            let _ = tx.send(MenuCommand::OpenDocumentation);
        });

    let tx = command_tx.clone();
    help_menu.add_item(MenuId::new("report_issue"), "Report Issue")
        .set_action(move || {
            debug!("Menu action: Report Issue");
            let _ = tx.send(MenuCommand::ReportIssue);
        });

    help_menu.add_native_item(MenuItem::Separator);

    let tx = command_tx.clone();
    help_menu.add_item(MenuId::new("about"), "About Tinker")
        .set_action(move || {
            debug!("Menu action: About");
            let _ = tx.send(MenuCommand::About);
        });

    menu_bar.add_submenu("Help", true, help_menu);

    menu_bar
} 
