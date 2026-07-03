"""
flowlocal_shared.config — Configuration from environment variables.

Services read their config from environment variables, which are set
by the Tauri parent process or dev scripts. All values have sensible
defaults so services can run standalone during development.
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field


@dataclass
class Config:
    # ── Ports ──────────────────────────────────────────────────
    whisper_port: int = 7771
    llm_port: int = 7772
    rag_port: int = 7773
    host: str = "127.0.0.1"

    # ── Whisper ────────────────────────────────────────────────
    whisper_model: str = "base"
    whisper_device: str = "auto"          # "auto" | "cuda" | "cpu"
    whisper_compute_type: str = "auto"    # "auto" | "float16" | "int8"
    whisper_language: str | None = None   # None = auto-detect
    whisper_vad_filter: bool = True
    whisper_vad_threshold: float = 0.5
    whisper_silence_ms: int = 500

    # ── Ollama ─────────────────────────────────────────────────
    ollama_host: str = "http://localhost:11434"
    ollama_model: str = "qwen3:4b"
    ollama_timeout: float = 60.0
    ollama_temperature: float = 0.1
    ollama_max_tokens: int = 2048

    # ── Embeddings ─────────────────────────────────────────────
    embedding_model: str = "nomic-embed-text"

    # ── RAG ────────────────────────────────────────────────────
    chroma_path: str = "./data/chroma"
    chroma_collection: str = "corrections"
    rag_max_results: int = 5

    # ── Logging ────────────────────────────────────────────────
    log_level: str = "INFO"


def load() -> Config:
    """Load config from environment variables, falling back to defaults."""
    c = Config()

    def _int(key: str, default: int) -> int:
        return int(os.environ.get(key, default))

    def _float(key: str, default: float) -> float:
        return float(os.environ.get(key, default))

    def _bool(key: str, default: bool) -> bool:
        v = os.environ.get(key, "").lower()
        if v in ("1", "true", "yes"):
            return True
        if v in ("0", "false", "no"):
            return False
        return default

    def _str(key: str, default: str) -> str:
        return os.environ.get(key, default)

    def _opt_str(key: str) -> str | None:
        return os.environ.get(key) or None

    c.whisper_port = _int("FLOWLOCAL_WHISPER_PORT", c.whisper_port)
    c.llm_port = _int("FLOWLOCAL_LLM_PORT", c.llm_port)
    c.rag_port = _int("FLOWLOCAL_RAG_PORT", c.rag_port)
    c.host = _str("FLOWLOCAL_HOST", c.host)

    c.whisper_model = _str("FLOWLOCAL_WHISPER_MODEL", c.whisper_model)
    c.whisper_device = _str("FLOWLOCAL_WHISPER_DEVICE", c.whisper_device)
    c.whisper_compute_type = _str("FLOWLOCAL_WHISPER_COMPUTE_TYPE", c.whisper_compute_type)
    c.whisper_language = _opt_str("FLOWLOCAL_WHISPER_LANGUAGE")
    c.whisper_vad_filter = _bool("FLOWLOCAL_WHISPER_VAD", c.whisper_vad_filter)
    c.whisper_vad_threshold = _float("FLOWLOCAL_WHISPER_VAD_THRESHOLD", c.whisper_vad_threshold)
    c.whisper_silence_ms = _int("FLOWLOCAL_WHISPER_SILENCE_MS", c.whisper_silence_ms)

    c.ollama_host = _str("FLOWLOCAL_OLLAMA_HOST", c.ollama_host)
    c.ollama_model = _str("FLOWLOCAL_OLLAMA_MODEL", c.ollama_model)
    c.ollama_timeout = _float("FLOWLOCAL_OLLAMA_TIMEOUT", c.ollama_timeout)
    c.ollama_temperature = _float("FLOWLOCAL_OLLAMA_TEMPERATURE", c.ollama_temperature)
    c.ollama_max_tokens = _int("FLOWLOCAL_OLLAMA_MAX_TOKENS", c.ollama_max_tokens)

    c.embedding_model = _str("FLOWLOCAL_EMBEDDING_MODEL", c.embedding_model)

    c.chroma_path = _str("FLOWLOCAL_CHROMA_PATH", c.chroma_path)
    c.rag_max_results = _int("FLOWLOCAL_RAG_MAX_RESULTS", c.rag_max_results)

    c.log_level = _str("FLOWLOCAL_LOG_LEVEL", c.log_level)

    return c
