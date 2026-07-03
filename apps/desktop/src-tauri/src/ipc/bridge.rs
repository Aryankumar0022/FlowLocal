// ============================================================
// ipc/bridge.rs — Rust ↔ Python service communication
//
// Transport: TCP localhost (platform-agnostic, no pywin32 dep)
//   Whisper:  127.0.0.1:7771
//   LLM:      127.0.0.1:7772
//   RAG:      127.0.0.1:7773
//
// Protocol: [u32 LE length][UTF-8 JSON payload]
// ============================================================

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

use super::protocol::{decode_payload, encode_frame, InboundMsg, OutboundMsg};
use crate::audio::AudioChunk;
use crate::settings::CleanupAggressiveness;

// ──────────────────────────────────────────────────────────────
// TCP connection wrapper
// ──────────────────────────────────────────────────────────────

struct ServiceConn {
    reader: tokio::io::ReadHalf<TcpStream>,
    writer: tokio::io::WriteHalf<TcpStream>,
}

impl ServiceConn {
    async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .with_context(|| format!("Failed to connect to AI service at {}", addr))?;

        // Disable Nagle's algorithm for minimum latency on small frames
        stream.set_nodelay(true)?;

        let (reader, writer) = tokio::io::split(stream);
        Ok(Self { reader, writer })
    }

    async fn send(&mut self, msg: &OutboundMsg) -> Result<()> {
        let frame = encode_frame(msg)?;
        self.writer.write_all(&frame).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<InboundMsg> {
        let mut len_buf = [0u8; 4];
        self.reader.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut payload = vec![0u8; len];
        self.reader.read_exact(&mut payload).await?;
        decode_payload(&payload)
    }
}

// ──────────────────────────────────────────────────────────────
// Per-service client with persistent connection + retry
// ──────────────────────────────────────────────────────────────

struct ServiceClient {
    name: &'static str,
    addr: String,
    conn: Mutex<Option<ServiceConn>>,
}

impl ServiceClient {
    fn new(name: &'static str, addr: String) -> Self {
        Self {
            name,
            addr,
            conn: Mutex::new(None),
        }
    }

    async fn ensure_connected(&self) -> Result<()> {
        let mut guard = self.conn.lock().await;
        if guard.is_none() {
            let conn = self
                .connect_with_retry(20, Duration::from_millis(500))
                .await?;
            *guard = Some(conn);
        }
        Ok(())
    }

    async fn connect_with_retry(&self, max: u32, delay: Duration) -> Result<ServiceConn> {
        for attempt in 1..=max {
            match ServiceConn::connect(&self.addr).await {
                Ok(c) => {
                    tracing::info!("Connected to {} ({})", self.name, self.addr);
                    return Ok(c);
                }
                Err(e) if attempt < max => {
                    tracing::debug!(
                        "[{}/{}] Waiting for {} service: {}",
                        attempt,
                        max,
                        self.name,
                        e
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!(
                            "{} service unavailable after {} attempts at {}",
                            self.name, max, self.addr
                        )
                    });
                }
            }
        }
        unreachable!()
    }

    /// Send a message and await one response.
    async fn request(&self, msg: &OutboundMsg) -> Result<InboundMsg> {
        self.ensure_connected().await?;
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().context("No connection")?;
        conn.send(msg).await?;
        let resp = conn.recv().await.map_err(|e| {
            tracing::warn!("{} read error — will reconnect on next call: {}", self.name, e);
            e
        })?;
        if matches!(resp, InboundMsg::ServiceError { .. }) {
            *guard = None; // force reconnect on error
        }
        Ok(resp)
    }

    /// Send only, no response expected.
    async fn send_only(&self, msg: &OutboundMsg) -> Result<()> {
        self.ensure_connected().await?;
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().context("No connection")?;
        conn.send(msg).await
    }

    /// Receive the next inbound message (no send).
    async fn recv_next(&self) -> Result<InboundMsg> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().context("No connection")?;
        conn.recv().await
    }
}

// ──────────────────────────────────────────────────────────────
// Public transcript result
// ──────────────────────────────────────────────────────────────

pub struct TranscriptResult {
    pub text: String,
    pub language: String,
    pub duration_ms: u64,
}

// ──────────────────────────────────────────────────────────────
// IpcBridge — Tauri managed state
// ──────────────────────────────────────────────────────────────

pub struct IpcBridge {
    whisper: Arc<ServiceClient>,
    llm: Arc<ServiceClient>,
    rag: Arc<ServiceClient>,
    partial_store: RwLock<std::collections::HashMap<String, String>>,
}

impl IpcBridge {
    pub fn new() -> Self {
        Self {
            whisper: Arc::new(ServiceClient::new("whisper", "127.0.0.1:7771".to_string())),
            llm: Arc::new(ServiceClient::new("llm", "127.0.0.1:7772".to_string())),
            rag: Arc::new(ServiceClient::new("rag", "127.0.0.1:7773".to_string())),
            partial_store: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub async fn health_check(&self) -> (bool, bool, bool) {
        let w = self.ping(&self.whisper).await;
        let l = self.ping(&self.llm).await;
        let r = self.ping(&self.rag).await;
        (w, l, r)
    }

    async fn ping(&self, client: &ServiceClient) -> bool {
        matches!(
            client.request(&OutboundMsg::Ping).await,
            Ok(InboundMsg::Pong { .. })
        )
    }

    // ── Whisper ───────────────────────────────────────────────

    pub async fn send_audio_chunk(&self, session_id: &str, chunk: &AudioChunk) -> Result<()> {
        let raw: Vec<u8> = chunk
            .samples
            .iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();
        let data = B64.encode(&raw);

        self.whisper
            .send_only(&OutboundMsg::AudioChunk {
                session_id: session_id.to_string(),
                seq: chunk.seq,
                data,
                sample_rate: chunk.sample_rate,
            })
            .await
    }

    pub async fn send_audio_end(&self, session_id: &str) -> Result<()> {
        self.whisper
            .send_only(&OutboundMsg::AudioEnd {
                session_id: session_id.to_string(),
            })
            .await
    }

    pub async fn try_recv_partial(&self, session_id: &str) -> Option<String> {
        self.partial_store.read().await.get(session_id).cloned()
    }

    pub async fn await_transcript(&self, session_id: &str) -> Result<TranscriptResult> {
        loop {
            let msg = tokio::time::timeout(
                Duration::from_secs(120),
                self.whisper.recv_next(),
            )
            .await
            .context("Whisper timed out (120s)")??;

            match msg {
                InboundMsg::TranscriptPartial {
                    text,
                    session_id: sid,
                } if sid == session_id => {
                    self.partial_store
                        .write()
                        .await
                        .insert(session_id.to_string(), text);
                }
                InboundMsg::TranscriptFinal {
                    text,
                    language,
                    duration_ms,
                    session_id: sid,
                } if sid == session_id => {
                    self.partial_store.write().await.remove(session_id);
                    return Ok(TranscriptResult { text, language, duration_ms });
                }
                InboundMsg::ServiceError { message, .. } => {
                    anyhow::bail!("Whisper error: {}", message);
                }
                _ => {} // messages for other sessions
            }
        }
    }

    // ── LLM ───────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub async fn clean_text(
        &self,
        session_id: &str,
        raw_text: &str,
        app_context: &str,
        language: &str,
        aggressiveness: &CleanupAggressiveness,
        rag_context: Vec<String>,
        dictionary_terms: Vec<[String; 2]>,
        remove_fillers: bool,
        fix_punctuation: bool,
        fix_capitalization: bool,
    ) -> Result<String> {
        let agg = match aggressiveness {
            CleanupAggressiveness::Light => "light",
            CleanupAggressiveness::Moderate => "moderate",
            CleanupAggressiveness::Aggressive => "aggressive",
        };

        let msg = OutboundMsg::CleanText {
            session_id: session_id.to_string(),
            raw_text: raw_text.to_string(),
            app_context: app_context.to_string(),
            language: language.to_string(),
            aggressiveness: agg.to_string(),
            rag_context,
            dictionary_terms,
            remove_fillers,
            fix_punctuation,
            fix_capitalization,
        };

        match self.llm.request(&msg).await? {
            InboundMsg::TextReady { text, .. } => Ok(text),
            InboundMsg::ServiceError { message, .. } => {
                anyhow::bail!("LLM error: {}", message)
            }
            other => anyhow::bail!("Unexpected LLM response: {:?}", other),
        }
    }

    pub async fn execute_command(
        &self,
        session_id: &str,
        command: &str,
        text: &str,
        language: &str,
    ) -> Result<String> {
        let msg = OutboundMsg::ExecuteCommand {
            session_id: session_id.to_string(),
            command: command.to_string(),
            text: text.to_string(),
            language: language.to_string(),
        };
        match self.llm.request(&msg).await? {
            InboundMsg::CommandResult { text, .. } => Ok(text),
            InboundMsg::ServiceError { message, .. } => {
                anyhow::bail!("LLM command error: {}", message)
            }
            other => anyhow::bail!("Unexpected LLM response: {:?}", other),
        }
    }

    // ── RAG ───────────────────────────────────────────────────

    pub async fn retrieve_context(
        &self,
        session_id: &str,
        query_text: &str,
        max_results: u32,
    ) -> Result<Vec<String>> {
        let msg = OutboundMsg::RetrieveContext {
            session_id: session_id.to_string(),
            query_text: query_text.to_string(),
            max_results,
        };
        match self.rag.request(&msg).await? {
            InboundMsg::ContextReady { segments, .. } => Ok(segments),
            InboundMsg::ServiceError { message, .. } => {
                anyhow::bail!("RAG error: {}", message)
            }
            other => anyhow::bail!("Unexpected RAG response: {:?}", other),
        }
    }

    pub async fn store_correction(
        &self,
        session_id: &str,
        raw_text: &str,
        clean_text: &str,
        app_context: &str,
        language: &str,
    ) -> Result<()> {
        let msg = OutboundMsg::StoreCorrection {
            session_id: session_id.to_string(),
            raw_text: raw_text.to_string(),
            clean_text: clean_text.to_string(),
            app_context: app_context.to_string(),
            language: language.to_string(),
        };
        let rag = self.rag.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = rag.send_only(&msg).await {
                tracing::warn!("RAG store failed: {}", e);
            }
        });
        Ok(())
    }

    pub async fn shutdown(&self) {
        for client in [&self.whisper, &self.llm, &self.rag] {
            let _ = client.send_only(&OutboundMsg::Shutdown).await;
        }
    }
}
