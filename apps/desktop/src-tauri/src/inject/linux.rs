// ============================================================
// inject/linux.rs — Text injection via xdotool type
//
// Uses `xdotool type --clearmodifiers --delay 0 -- <text>`
// which works in X11 applications including VS Code, Chrome,
// Slack, and most terminals.
//
// For Wayland, ydotool is used as a fallback.
// ============================================================

#![cfg(target_os = "linux")]

use anyhow::{Context, Result};
use std::process::Command;

/// Inject text into the active application.
pub fn inject(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    // Try xdotool first (X11), then ydotool (Wayland)
    if is_available("xdotool") {
        inject_xdotool(text)
    } else if is_available("ydotool") {
        inject_ydotool(text)
    } else {
        anyhow::bail!(
            "No text injection tool found. Install xdotool (X11) or ydotool (Wayland):\n  \
             sudo apt install xdotool\n  \
             sudo apt install ydotool"
        )
    }
}

fn inject_xdotool(text: &str) -> Result<()> {
    // `xdotool type` is the most reliable method:
    //   --clearmodifiers: release Shift/Ctrl etc. held by user
    //   --delay 0: no inter-character delay (maximum speed)
    //   -- text:  double-dash prevents interpreting text as flags
    let output = Command::new("xdotool")
        .args(["type", "--clearmodifiers", "--delay", "0", "--", text])
        .output()
        .context("Failed to execute xdotool")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("xdotool type failed: {}", stderr);
    }

    Ok(())
}

fn inject_ydotool(text: &str) -> Result<()> {
    let output = Command::new("ydotool")
        .args(["type", "--", text])
        .output()
        .context("Failed to execute ydotool")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ydotool type failed: {}", stderr);
    }

    Ok(())
}

fn is_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
