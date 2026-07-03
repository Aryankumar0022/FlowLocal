// ipc/mod.rs — Public re-exports for the IPC module
pub mod bridge;
pub mod protocol;

pub use bridge::IpcBridge;
pub use protocol::{InboundMsg, OutboundMsg};
