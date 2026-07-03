// ============================================================
// context/detector.rs — Detect the currently focused application
//
// Windows: GetForegroundWindow → GetWindowThreadProcessId →
//          QueryFullProcessImageName → basename
// Linux:   xdotool getactivewindow getwindowname / getwindowfocus
// ============================================================

use anyhow::Result;
use serde::Serialize;

use super::app_map::{map_process, refine_browser_context, AppContext};

/// Information about the currently active application.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ActiveApp {
    /// OS process name (e.g. "code.exe" or "chrome")
    pub process_name: String,
    /// Window title text
    pub window_title: String,
    /// Resolved context label for prompt selection
    pub context: AppContext,
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext::Generic
    }
}

pub struct ContextDetector;

impl ContextDetector {
    /// Snapshot the active application right now.
    pub fn detect() -> ActiveApp {
        detect_impl()
    }
}

// ──────────────────────────────────────────────────────────────
// Windows implementation
// ──────────────────────────────────────────────────────────────

#[cfg(windows)]
fn detect_impl() -> ActiveApp {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND(std::ptr::null_mut()) {
            return ActiveApp::default();
        }

        // Get window title
        let title_len = GetWindowTextLengthW(hwnd);
        let mut title_buf = vec![0u16; (title_len + 1) as usize];
        GetWindowTextW(hwnd, &mut title_buf);
        let window_title = OsString::from_wide(
            &title_buf[..title_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(title_buf.len())],
        )
        .to_string_lossy()
        .to_string();

        // Get process ID
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return ActiveApp {
                window_title,
                ..Default::default()
            };
        }

        // Open process for name query
        let h_proc = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => {
                return ActiveApp {
                    window_title,
                    ..Default::default()
                }
            }
        };

        // Get full exe path, then extract basename
        let mut path_buf = vec![0u16; 1024];
        let mut size = path_buf.len() as u32;
        use windows::core::PWSTR;
        let ok = QueryFullProcessImageNameW(h_proc, PROCESS_NAME_WIN32, PWSTR(path_buf.as_mut_ptr()), &mut size);
        let _ = windows::Win32::Foundation::CloseHandle(h_proc);

        let process_name = if ok.is_ok() {
            let path_str = OsString::from_wide(&path_buf[..size as usize])
                .to_string_lossy()
                .to_string();
            std::path::Path::new(&path_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase()
        } else {
            String::new()
        };

        let mut context = map_process(&process_name);
        if context == AppContext::Browser {
            context = refine_browser_context(&window_title);
        }

        ActiveApp {
            process_name,
            window_title,
            context,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Linux implementation (requires xdotool)
// ──────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn detect_impl() -> ActiveApp {
    use std::process::Command;

    // Get focused window ID
    let wid_output = match Command::new("xdotool").arg("getactivewindow").output() {
        Ok(o) => o,
        Err(_) => return ActiveApp::default(),
    };
    let wid = String::from_utf8_lossy(&wid_output.stdout).trim().to_string();
    if wid.is_empty() {
        return ActiveApp::default();
    }

    // Get window title
    let title_output = Command::new("xdotool")
        .args(["getwindowname", &wid])
        .output()
        .ok();
    let window_title = title_output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Get PID
    let pid_output = Command::new("xdotool")
        .args(["getwindowpid", &wid])
        .output()
        .ok();
    let pid_str = pid_output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Get process name from /proc/<pid>/comm
    let process_name = if let Ok(pid) = pid_str.parse::<u32>() {
        std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .ok()
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let mut context = map_process(&process_name);
    if context == AppContext::Browser {
        context = refine_browser_context(&window_title);
    }

    ActiveApp {
        process_name,
        window_title,
        context,
    }
}

// ──────────────────────────────────────────────────────────────
// Fallback for unsupported platforms
// ──────────────────────────────────────────────────────────────

#[cfg(not(any(windows, target_os = "linux")))]
fn detect_impl() -> ActiveApp {
    ActiveApp::default()
}
