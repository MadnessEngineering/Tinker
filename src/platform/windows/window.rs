use std::error::Error;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM, LRESULT, GetLastError};
use windows::Win32::UI::WindowsAndMessaging::{
    WNDCLASSEXW, RegisterClassExW, CreateWindowExW, DefWindowProcW,
    PostQuitMessage, GetMessageW, TranslateMessage, DispatchMessageW,
    CS_HREDRAW, CS_VREDRAW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, MSG,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE,
};
use windows::Win32::UI::Controls::{
    TCITEMW, TCM_INSERTITEMW, TCM_DELETEITEM, TCM_SETCURSEL,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
use windows::core::{PCWSTR, PWSTR, w};
use anyhow::{Result, anyhow};
use tracing::debug;
use super::config::WindowsConfig;
use crate::platform::PlatformManager;

pub struct WindowsManager {
    hwnd: HWND,
    config: WindowsConfig,
    tab_control: Option<HWND>,
}

impl WindowsManager {
    pub fn new(config: WindowsConfig) -> Result<Self> {
        unsafe {
            // Set DPI awareness if requested
            if config.dpi_aware {
                if let Err(e) = SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) {
                    debug!("Failed to set DPI awareness: {:?}", e);
                }
            }

            // Register window class
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                hInstance: GetModuleHandleW(None)?,
                lpszClassName: w!("TinkerWindow"),
                ..Default::default()
            };

            if RegisterClassExW(&wc) == 0 {
                let error = GetLastError();
                return Err(anyhow!("Failed to register window class: {:?}", error));
            }

            // Create window
            let hwnd = CreateWindowExW(
                Default::default(),
                w!("TinkerWindow"),
                w!("Tinker Browser"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                800,
                600,
                None,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            if hwnd == HWND(0) {
                let error = GetLastError();
                return Err(anyhow!("Failed to create window: {:?}", error));
            }

            Ok(Self {
                hwnd,
                config,
                tab_control: None,
            })
        }
    }

    pub fn add_tab(&self, title: &str) -> Result<()> {
        if let Some(tab_control) = self.tab_control {
            let title = format!("{}\0", title);
            let mut item = TCITEMW {
                mask: Controls::TCIF_TEXT,
                pszText: PWSTR(title.encode_utf16().collect::<Vec<_>>().as_ptr() as *mut _),
                ..Default::default()
            };

            let result = unsafe {
                WindowsAndMessaging::SendMessageW(
                    tab_control,
                    TCM_INSERTITEMW,
                    WPARAM(0),
                    LPARAM(&mut item as *mut _ as isize),
                )
            };

            if result.0 == -1 {
                Err(anyhow!("Failed to add tab"))
            } else {
                Ok(())
            }
        } else {
            Ok(()) // No tab control, silently succeed
        }
    }

    pub fn remove_tab(&self, index: usize) -> Result<()> {
        if let Some(tab_control) = self.tab_control {
            let result = unsafe {
                WindowsAndMessaging::SendMessageW(
                    tab_control,
                    TCM_DELETEITEM,
                    WPARAM(index),
                    LPARAM(0),
                )
            };

            if result.0 == 0 {
                Err(anyhow!("Failed to remove tab"))
            } else {
                Ok(())
            }
        } else {
            Ok(()) // No tab control, silently succeed
        }
    }

    pub fn set_active_tab(&self, index: usize) -> Result<()> {
        if let Some(tab_control) = self.tab_control {
            let result = unsafe {
                WindowsAndMessaging::SendMessageW(
                    tab_control,
                    TCM_SETCURSEL,
                    WPARAM(index),
                    LPARAM(0),
                )
            };

            if result.0 == -1 {
                Err(anyhow!("Failed to set active tab"))
            } else {
                Ok(())
            }
        } else {
            Ok(()) // No tab control, silently succeed
        }
    }

    extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DESTROY => {
                unsafe {
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                // Handle window resize
                let width = (lparam.0 & 0xFFFF) as i32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
                debug!("Window resized to {}x{}", width, height);
                LRESULT(0)
            }
            _ => unsafe {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }
}

impl PlatformManager for WindowsManager {
    fn new(config: impl Into<String>) -> Result<Self> {
        let config = WindowsConfig {
            title: config.into(),
            width: 800,
            height: 600,
            decorations: true,
            dpi_aware: true,
        };
        Self::new(config)
    }

    fn run(&self) -> Result<()> {
        let mut msg = MSG::default();
        
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }
} 