// ============================================================
// inject/windows.rs — Text injection via Windows Clipboard API
//
// Strategy: Save clipboard → set our text → Ctrl+V → restore
// This works in every Windows application without exception,
// including VS Code, Chrome, Slack, terminals, and Electron apps.
// ============================================================

#![cfg(windows)]

use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

const CF_UNICODETEXT: u32 = 13;

/// Inject `text` into the active application using the clipboard.
pub fn inject(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    // 1. Encode text as null-terminated UTF-16
    let wide: Vec<u16> = OsStr::new(text)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let byte_size = wide.len() * 2;

    unsafe {
        // 2. Save current clipboard contents so we can restore them
        let prev_data = save_clipboard();

        // 3. Allocate global memory and fill with our text
        let h_mem: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, byte_size)
            .context("GlobalAlloc failed")?;

        let ptr = GlobalLock(h_mem) as *mut u16;
        if ptr.is_null() {
            anyhow::bail!("GlobalLock returned null");
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(h_mem).ok();

        // 4. Set clipboard data (clipboard takes ownership of h_mem)
        OpenClipboard(HWND(std::ptr::null_mut()))
            .context("OpenClipboard failed")?;
        EmptyClipboard().context("EmptyClipboard failed")?;
        SetClipboardData(CF_UNICODETEXT, HANDLE(h_mem.0 as *mut core::ffi::c_void))
            .context("SetClipboardData failed")?;
        CloseClipboard().context("CloseClipboard failed")?;

        // 5. Send Ctrl+V key sequence
        send_ctrl_v()?;

        // 6. Brief delay to ensure Ctrl+V is processed before we restore
        std::thread::sleep(std::time::Duration::from_millis(80));

        // 7. Restore previous clipboard contents
        restore_clipboard(prev_data);
    }

    Ok(())
}

/// Send a Ctrl+Down → V+Down → V+Up → Ctrl+Up sequence.
unsafe fn send_ctrl_v() -> Result<()> {
    let inputs = [
        make_key_input(VK_CONTROL, windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0)),
        make_key_input(VK_V, windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0)),
        make_key_input(VK_V, KEYEVENTF_KEYUP),
        make_key_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ];

    let sent = SendInput(
        &inputs,
        std::mem::size_of::<INPUT>() as i32,
    );

    if sent != inputs.len() as u32 {
        anyhow::bail!("SendInput only sent {}/{} events", sent, inputs.len());
    }

    Ok(())
}

fn make_key_input(
    vk: VIRTUAL_KEY,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

// ──────────────────────────────────────────────────────────────
// Clipboard save / restore
// ──────────────────────────────────────────────────────────────

struct SavedClipboard {
    data: Vec<u16>,
}

unsafe fn save_clipboard() -> Option<SavedClipboard> {
    if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
        return None;
    }

    let h = GetClipboardData(CF_UNICODETEXT).ok();
    let data = h.and_then(|handle| {
        let h_global = HGLOBAL(handle.0 as *mut core::ffi::c_void);
        let ptr = GlobalLock(h_global) as *const u16;
        if ptr.is_null() {
            return None;
        }
        // Copy the null-terminated wide string
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len + 1);
        let owned = slice.to_vec();
        GlobalUnlock(h_global).ok();
        Some(owned)
    });

    CloseClipboard().ok();
    data.map(|d| SavedClipboard { data: d })
}

unsafe fn restore_clipboard(saved: Option<SavedClipboard>) {
    let Some(s) = saved else { return };

    let byte_size = s.data.len() * 2;
    let Ok(h_mem): Result<HGLOBAL, _> = GlobalAlloc(GMEM_MOVEABLE, byte_size) else {
        return;
    };

    let ptr = GlobalLock(h_mem) as *mut u16;
    if ptr.is_null() {
        return;
    }
    std::ptr::copy_nonoverlapping(s.data.as_ptr(), ptr, s.data.len());
    GlobalUnlock(h_mem).ok();

    if OpenClipboard(HWND(std::ptr::null_mut())).is_ok() {
        EmptyClipboard().ok();
        SetClipboardData(CF_UNICODETEXT, HANDLE(h_mem.0 as *mut core::ffi::c_void)).ok();
        CloseClipboard().ok();
    }
}
