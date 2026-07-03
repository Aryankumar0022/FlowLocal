// ============================================================
// ipc/protocol.rs — Wire message types for Rust ↔ Python IPC
// Length-prefixed JSON frames: [u32 LE length][UTF-8 JSON payload]
// ============================================================

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────
// Outbound: Rust → Python
// ──────────────────────────────────────────────────────────────

/// Every message sent from the Rust backend to a Python service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutboundMsg {
    /// Raw PCM audio chunk (base64-encoded f32 LE samples at `sample_rate` Hz, mono)
    AudioChunk {
        session_id: String,
        /// Sequential chunk counter for ordering / loss detection
        seq: u32,
        /// Base64-encoded raw f32 LE PCM bytes
        data: String,
        sample_rate: u32,
    },

    /// Signals that audio recording has ended for this session.
    AudioEnd {
        session_id: String,
    },

    /// Ask the LLM service to clean up raw transcribed text.
    CleanText {
        session_id: String,
        raw_text: String,
        /// E.g. "vscode" | "slack" | "email" | "generic"
        app_context: String,
        /// ISO 639-1 language code, e.g. "en"
        language: String,
        /// "light" | "moderate" | "aggressive"
        aggressiveness: String,
        /// Retrieved RAG context snippets (past corrections)
        rag_context: Vec<String>,
        /// Personal dictionary: [(wrong, correct), ...]
        dictionary_terms: Vec<[String; 2]>,
        remove_fillers: bool,
        fix_punctuation: bool,
        fix_capitalization: bool,
    },

    /// Ask the LLM service to execute a voice command on text.
    ExecuteCommand {
        session_id: String,
        /// E.g. "rewrite_professional" | "summarize" | "bullet_points" | "translate_hindi"
        command: String,
        text: String,
        language: String,
    },

    /// Store a raw→clean correction pair in ChromaDB for future RAG retrieval.
    StoreCorrection {
        session_id: String,
        raw_text: String,
        clean_text: String,
        app_context: String,
        language: String,
    },

    /// Retrieve the most similar past corrections for the given query.
    RetrieveContext {
        session_id: String,
        query_text: String,
        max_results: u32,
    },

    /// Health-check ping.
    Ping,

    /// Ask the service to flush state and exit cleanly.
    Shutdown,
}

// ──────────────────────────────────────────────────────────────
// Inbound: Python → Rust
// ──────────────────────────────────────────────────────────────

/// Every message received by the Rust backend from a Python service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InboundMsg {
    /// Streaming partial transcript (sent while audio is still flowing).
    TranscriptPartial {
        session_id: String,
        text: String,
    },

    /// Final transcript after VAD silence or explicit AudioEnd.
    TranscriptFinal {
        session_id: String,
        text: String,
        /// Detected ISO 639-1 language code
        language: String,
        /// Actual audio duration processed, in milliseconds
        duration_ms: u64,
    },

    /// Cleaned / formatted text ready for injection.
    TextReady {
        session_id: String,
        text: String,
    },

    /// Retrieved RAG context segments.
    ContextReady {
        session_id: String,
        segments: Vec<String>,
    },

    /// Result of an AI command execution.
    CommandResult {
        session_id: String,
        command: String,
        text: String,
    },

    /// Sent by a service once it has fully initialized and is ready.
    Ready {
        service: String,
        version: String,
    },

    /// Response to a Ping message.
    Pong {
        service: String,
    },

    /// Error from a Python service.
    ServiceError {
        session_id: Option<String>,
        code: u32,
        message: String,
    },
}

// ──────────────────────────────────────────────────────────────
// Helper: frame serialization
// ──────────────────────────────────────────────────────────────

/// Serialize a message as a length-prefixed frame: [u32 LE][JSON bytes]
pub fn encode_frame(msg: &OutboundMsg) -> anyhow::Result<Vec<u8>> {
    let payload = serde_json::to_vec(msg)?;
    let len = payload.len() as u32;
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Deserialize an `InboundMsg` from a raw JSON byte slice (already length-stripped).
pub fn decode_payload(bytes: &[u8]) -> anyhow::Result<InboundMsg> {
    Ok(serde_json::from_slice(bytes)?)
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_frame_ping() {
        let msg = OutboundMsg::Ping;
        let frame = encode_frame(&msg).expect("Failed to encode");
        
        assert!(frame.len() > 4);
        let len_bytes: [u8; 4] = frame[0..4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes) as usize;
        assert_eq!(len, frame.len() - 4);

        let json_str = std::str::from_utf8(&frame[4..]).unwrap();
        assert_eq!(json_str, r#"{"type":"ping"}"#);
    }

    #[test]
    fn test_decode_payload_pong() {
        let json = r#"{"type":"pong","service":"whisper"}"#;
        let msg = decode_payload(json.as_bytes()).expect("Failed to decode");
        
        match msg {
            InboundMsg::Pong { service } => {
                assert_eq!(service, "whisper");
            }
            _ => panic!("Expected Pong message"),
        }
    }
}

