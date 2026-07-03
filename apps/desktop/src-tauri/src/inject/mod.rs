// inject/mod.rs — Text injection: platform dispatch
pub mod linux;
pub mod windows;

use anyhow::Result;

/// Inject `text` into the currently focused application.
/// Dispatches to the platform-specific implementation.
pub fn inject_text(text: &str) -> Result<()> {
    #[cfg(windows)]
    {
        windows::inject(text)
    }
    #[cfg(target_os = "linux")]
    {
        linux::inject(text)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        anyhow::bail!("Text injection not supported on this platform");
    }
}
