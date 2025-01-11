use anyhow::{Result, anyhow};
use windows::Win32::{
    UI::{
        WindowsAndMessaging as win32_window,
        Controls as win32_controls,
        Shell as win32_shell,
        HiDpi as win32_dpi,
        Input::KeyboardAndMouse as win32_input,
    },
    Foundation::{HWND, LPARAM, WPARAM, LRESULT, RECT},
    Graphics::Gdi::{HBRUSH, HDC},
};
use tracing::{debug, error};
use super::{WindowsConfig, WindowsTheme};
use std::sync::Arc;

pub struct WindowsManager {
    hwnd: HWND,
    config: WindowsConfig,
    tab_control: Option<HWND>,
}

impl WindowsManager {
    pub fn new(config: WindowsConfig) -> Result<Self> {
        // Register window class
        let class_name = "TinkerBrowser";
        let wc = win32_window::WNDCLASSEXW {
            cbSize: std::mem::size_of::<win32_window::WNDCLASSEXW>() as u32,
            style: win32_window::CS_HREDRAW | win32_window::CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            hInstance: unsafe { win32_window::GetModuleHandleW(None)? },
            lpszClassName: class_name.into(),
            ..Default::default()
        };

        unsafe {
            win32_window::RegisterClassExW(&wc);
        }

        // Create main window
        let hwnd = unsafe {
            win32_window::CreateWindowExW(
                win32_window::WINDOW_EX_STYLE::default(),
                class_name,
                &config.title,
                if config.decorations {
                    win32_window::WS_OVERLAPPEDWINDOW
                } else {
                    win32_window::WS_POPUP
                },
                win32_window::CW_USEDEFAULT,
                win32_window::CW_USEDEFAULT,
                config.width as i32,
                config.height as i32,
                None,
                None,
                wc.hInstance,
                None,
            )
        };

        if hwnd.0 == 0 {
            return Err(anyhow!("Failed to create window"));
        }

        // Create tab control if decorations are enabled
        let tab_control = if config.decorations {
            Some(unsafe {
                win32_controls::CreateWindowExW(
                    win32_window::WINDOW_EX_STYLE::default(),
                    win32_controls::WC_TABCONTROL,
                    None,
                    win32_window::WS_CHILD | win32_window::WS_VISIBLE,
                    0,
                    0,
                    config.width as i32,
                    30,
                    hwnd,
                    None,
                    wc.hInstance,
                    None,
                )
            })
        } else {
            None
        };

        // Set DPI awareness
        if config.dpi_aware {
            unsafe {
                win32_dpi::SetProcessDpiAwareness(win32_dpi::PROCESS_PER_MONITOR_DPI_AWARE)?;
            }
        }

        // Show window
        unsafe {
            win32_window::ShowWindow(hwnd, win32_window::SW_SHOW);
            win32_window::UpdateWindow(hwnd);
        }

        Ok(Self {
            hwnd,
            config,
            tab_control,
        })
    }

    pub fn add_tab(&self, title: &str) -> Result<()> {
        if let Some(tab_control) = self.tab_control {
            let mut item = win32_controls::TCITEMW {
                mask: win32_controls::TCIF_TEXT,
                pszText: title.into(),
                ..Default::default()
            };

            let result = unsafe {
                win32_controls::SendMessageW(
                    tab_control,
                    win32_controls::TCM_INSERTITEMW,
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
                win32_controls::SendMessageW(
                    tab_control,
                    win32_controls::TCM_DELETEITEM,
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
                win32_controls::SendMessageW(
                    tab_control,
                    win32_controls::TCM_SETCURSEL,
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
            win32_window::WM_DESTROY => {
                unsafe {
                    win32_window::PostQuitMessage(0);
                }
                LRESULT(0)
            }
            win32_window::WM_SIZE => {
                // Handle window resize
                let width = win32_window::LOWORD(lparam.0 as u32) as i32;
                let height = win32_window::HIWORD(lparam.0 as u32) as i32;
                debug!("Window resized to {}x{}", width, height);
                LRESULT(0)
            }
            _ => unsafe {
                win32_window::DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut msg = win32_window::MSG::default();
        
        unsafe {
            while win32_window::GetMessageW(&mut msg, None, 0, 0).into() {
                win32_window::TranslateMessage(&msg);
                win32_window::DispatchMessageW(&msg);
            }
        }

        Ok(())
    }
} 