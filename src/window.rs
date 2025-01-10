use std::env;
use anyhow::Error;
use wry::{WebView, WebViewBuilder};

#[cfg(target_os = "windows")]
use {
    windows::Win32::Foundation::HWND,
    webview2_com::WebView2Environment,
};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSWindow, NSWindowStyleMask};

pub struct BrowserWindow {
    #[cfg(target_os = "macos")]
    window: NSWindow,
    #[cfg(target_os = "windows")]
    window: HWND,
    webview: WebView,
}

impl BrowserWindow {
    pub fn new() -> Result<Self, Error> {
        #[cfg(target_os = "windows")]
        let window = {
            let env = WebView2Environment::builder().build()?;
            let window = tao::window::WindowBuilder::new()
                .with_title("Browser")
                .with_inner_size(tao::dpi::LogicalSize::new(800, 600))
                .build(&tao::event_loop::EventLoop::new()?)?;
            window.hwnd()
        };

        #[cfg(target_os = "macos")]
        let window = unsafe {
            let window = NSWindow::alloc(cocoa::base::nil).initWithContentRect_styleMask_backing_defer_(
                cocoa::foundation::NSRect::new(
                    cocoa::foundation::NSPoint::new(0., 0.),
                    cocoa::foundation::NSSize::new(800., 600.),
                ),
                NSWindowStyleMask::NSTitledWindowMask,
                cocoa::appkit::NSBackingStoreType::NSBackingStoreBuffered,
                false,
            );
            window.center();
            window
        };

        let webview = WebViewBuilder::new(window)?
            .with_transparent(true)
            .with_initialization_script(include_str!("../templates/init.js"))
            .build()?;

        Ok(Self { window, webview })
    }
} 