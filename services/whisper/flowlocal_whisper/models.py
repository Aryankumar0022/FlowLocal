"""
flowlocal_whisper.models — Model loading and singleton caching.

The WhisperModel is expensive to load (several seconds).
This module loads it once on startup and caches it globally.
"""

from __future__ import annotations

import logging
import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from faster_whisper import WhisperModel as FWModel

logger = logging.getLogger(__name__)

_model: "FWModel | None" = None
_model_name: str = ""


def load_model(
    model_name: str = "base",
    device: str = "auto",
    compute_type: str = "auto",
) -> "FWModel":
    """Load (or return cached) faster-whisper model.

    Device selection priority:
        "auto" → CUDA if available, else CPU
        "cuda" → GPU with float16 (falls back to int8 if VRAM is low)
        "cpu"  → CPU with int8_float32 for speed
    """
    global _model, _model_name

    if _model is not None and _model_name == model_name:
        return _model

    from faster_whisper import WhisperModel

    # Resolve device
    resolved_device = device
    resolved_compute = compute_type

    if device == "auto":
        resolved_device = _detect_device()

    if compute_type == "auto":
        if resolved_device == "cuda":
            resolved_compute = "float16"
        else:
            resolved_compute = "int8"

    logger.info(
        "Loading Whisper model '%s' on %s with compute_type=%s",
        model_name,
        resolved_device,
        resolved_compute,
    )

    # Download model to ~/.cache/huggingface by default
    _model = WhisperModel(
        model_name,
        device=resolved_device,
        compute_type=resolved_compute,
        num_workers=2,
        download_root=os.environ.get("FLOWLOCAL_MODEL_CACHE"),
    )
    _model_name = model_name

    logger.info("Whisper model loaded OK")
    return _model


def _detect_device() -> str:
    """Return 'cuda' if a CUDA GPU is available, else 'cpu'."""
    try:
        import torch  # type: ignore[import]

        if torch.cuda.is_available():
            logger.info("CUDA GPU detected: %s", torch.cuda.get_device_name(0))
            return "cuda"
    except ImportError:
        pass
    try:
        import ctranslate2  # type: ignore[import]

        if "cuda" in ctranslate2.get_supported_compute_types("cuda"):
            return "cuda"
    except Exception:
        pass
    logger.info("No CUDA GPU found — using CPU")
    return "cpu"
