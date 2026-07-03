"""
flowlocal_whisper.transcriber — Audio transcription via faster-whisper.

Handles:
  - Base64-encoded f32 PCM chunk buffering
  - Sample rate conversion (any Hz → 16 kHz)
  - faster-whisper transcription with built-in Silero VAD
  - Streaming partial transcript events per segment
"""

from __future__ import annotations

import base64
import logging
import time
from collections import defaultdict
from typing import AsyncIterator, Callable, Awaitable

import numpy as np
from scipy import signal as scipy_signal

from .models import load_model

logger = logging.getLogger(__name__)

TARGET_SAMPLE_RATE = 16_000  # faster-whisper requires 16 kHz


class SessionBuffer:
    """Accumulates audio chunks for one recording session."""

    def __init__(self, session_id: str) -> None:
        self.session_id = session_id
        self.chunks: list[np.ndarray] = []
        self.start_time = time.monotonic()

    def add_chunk(self, data_b64: str, sample_rate: int) -> None:
        """Decode a base64 f32 LE audio chunk and resample to 16 kHz."""
        raw = base64.b64decode(data_b64)
        samples = np.frombuffer(raw, dtype="<f4").astype(np.float32).copy()

        if sample_rate != TARGET_SAMPLE_RATE and len(samples) > 0:
            samples = _resample(samples, sample_rate, TARGET_SAMPLE_RATE)

        self.chunks.append(samples)

    def assemble(self) -> np.ndarray:
        """Concatenate all chunks into a single float32 array."""
        if not self.chunks:
            return np.zeros(0, dtype=np.float32)
        return np.concatenate(self.chunks)

    @property
    def duration_ms(self) -> int:
        return int((time.monotonic() - self.start_time) * 1000)


class Transcriber:
    """Stateful transcriber managing per-session audio buffers."""

    def __init__(
        self,
        model_name: str = "base",
        device: str = "auto",
        compute_type: str = "auto",
        language: str | None = None,
        vad_filter: bool = True,
        vad_threshold: float = 0.5,
        silence_duration_ms: int = 500,
    ) -> None:
        self.language = language
        self.vad_filter = vad_filter
        self.vad_threshold = vad_threshold
        self.silence_duration_ms = silence_duration_ms

        # Load model synchronously at startup
        self.model = load_model(model_name, device, compute_type)
        self._sessions: dict[str, SessionBuffer] = {}

    # ── Public API ─────────────────────────────────────────────

    def add_chunk(self, session_id: str, data_b64: str, sample_rate: int) -> None:
        """Buffer an audio chunk for the given session."""
        if session_id not in self._sessions:
            self._sessions[session_id] = SessionBuffer(session_id)
        self._sessions[session_id].add_chunk(data_b64, sample_rate)

    async def finalize(
        self,
        session_id: str,
        on_partial: Callable[[str, str], Awaitable[None]] | None = None,
    ) -> tuple[str, str, int]:
        """
        Transcribe the buffered audio for `session_id`.

        Returns (text, language, duration_ms).
        `on_partial` is called with (session_id, partial_text) for each segment.
        """
        buf = self._sessions.pop(session_id, None)
        if buf is None:
            return "", "en", 0

        audio = buf.assemble()
        duration_ms = buf.duration_ms

        if len(audio) < TARGET_SAMPLE_RATE * 0.1:
            # Less than 100ms of audio — too short
            logger.warning("Session %s: audio too short (%d samples)", session_id, len(audio))
            return "", "en", duration_ms

        logger.info(
            "Transcribing session %s: %.2fs of audio",
            session_id,
            len(audio) / TARGET_SAMPLE_RATE,
        )

        vad_params = {}
        if self.vad_filter:
            vad_params = {
                "threshold": self.vad_threshold,
                "min_silence_duration_ms": self.silence_duration_ms,
            }

        segments, info = self.model.transcribe(
            audio,
            language=self.language,
            vad_filter=self.vad_filter,
            vad_parameters=vad_params if self.vad_filter else None,
            word_timestamps=False,
            condition_on_previous_text=False,
            beam_size=5,
        )

        text_parts: list[str] = []
        detected_language = info.language or "en"

        for segment in segments:
            part = segment.text.strip()
            if not part:
                continue
            text_parts.append(part)

            # Stream partial transcript back to the Rust bridge
            if on_partial:
                partial = " ".join(text_parts)
                await on_partial(session_id, partial)

        full_text = " ".join(text_parts).strip()
        logger.info(
            "Transcript: %d chars | lang=%s | duration=%dms",
            len(full_text),
            detected_language,
            duration_ms,
        )
        return full_text, detected_language, duration_ms

    def cancel(self, session_id: str) -> None:
        """Discard buffered audio for a session (e.g. if user cancelled)."""
        self._sessions.pop(session_id, None)


# ──────────────────────────────────────────────────────────────
# Resampling utility
# ──────────────────────────────────────────────────────────────

def _resample(audio: np.ndarray, from_rate: int, to_rate: int) -> np.ndarray:
    """Resample audio from `from_rate` Hz to `to_rate` Hz."""
    if from_rate == to_rate:
        return audio
    ratio = to_rate / from_rate
    new_length = max(1, int(len(audio) * ratio))
    resampled = scipy_signal.resample(audio, new_length)
    return resampled.astype(np.float32)
