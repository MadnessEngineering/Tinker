use anyhow::{Result, anyhow};
use windows::Win32::{
    UI::{
        WindowsAndMessaging::{
            self, CreateWindowExW, DefWindowProcW, DispatchMessageW,
            GetMessageW, PostQuitMessage, RegisterClassExW, ShowWindow,
            WNDCLASSEXW, WS_OVERLAPPEDWINDOW, WS_POPUP, CW_USEDEFAULT,
            SW_SHOW, WM_DESTROY, WM_SIZE, MSG, TranslateMessage,
        },
        Controls::{
            self, TCM_INSERTITEMW, TCM_DELETEITEM, TCM_SETCURSEL,
            TCITEMW, WC_TABCONTROL,
        },
        HiDpi::{SetProcessDpiAwareness, PROCESS_DPI_AWARENESS, PROCESS_PER_MONITOR_DPI_AWARE},
    },
    Foundation::{HWND, LPARAM, WPARAM, LRESULT},
    Graphics::Gdi::UpdateWindow,
    System::LibraryLoader::GetModuleHandleW,
};
use windows::core::{PCWSTR, PWSTR, w};
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
        // Register window class
        let class_name = w!("TinkerBrowser");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WindowsAndMessaging::CS_HREDRAW | WindowsAndMessaging::CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            hInstance: unsafe { GetModuleHandleW(None)? },
            lpszClassName: class_name,
            ..Default::default()
        };

        unsafe {
            RegisterClassExW(&wc);
        }

        // Create main window
        let title = format!("{}\0", config.title);
        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                class_name,
                PCWSTR(title.encode_utf16().collect::<Vec<_>>().as_ptr()),
                if config.decorations {
                    WS_OVERLAPPEDWINDOW
                } else {
                    WS_POPUP
                },
                CW_USEDEFAULT,
                CW_USEDEFAULT,
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
                CreateWindowExW(
                    Default::default(),
                    WC_TABCONTROL,
                    None,
                    WindowsAndMessaging::WS_CHILD | WindowsAndMessaging::WS_VISIBLE,
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
                SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE)?;
            }
        }

        // Show window
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
        }

        Ok(Self {
            hwnd,
            config,
            tab_control,
        })
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