"""
flowlocal_whisper.main — Entry point for the Whisper service.

Usage:
    flowlocal-whisper
    python -m flowlocal_whisper.main
"""

from __future__ import annotations

import asyncio
import logging
import signal
import sys

from flowlocal_shared.config import load as load_config
from flowlocal_shared.logging import setup_logging

from .server import WhisperServer
from .transcriber import Transcriber


def main() -> None:
    cfg = load_config()
    setup_logging(cfg.log_level, "whisper")
    logger = logging.getLogger(__name__)

    logger.info("FlowLocal Whisper Service starting...")
    logger.info(
        "Model: %s | Device: %s | Compute: %s",
        cfg.whisper_model,
        cfg.whisper_device,
        cfg.whisper_compute_type,
    )

    # Load the model synchronously before starting the server
    # (this avoids a cold-start delay on first transcription)
    transcriber = Transcriber(
        model_name=cfg.whisper_model,
        device=cfg.whisper_device,
        compute_type=cfg.whisper_compute_type,
        language=cfg.whisper_language,
        vad_filter=cfg.whisper_vad_filter,
        vad_threshold=cfg.whisper_vad_threshold,
        silence_duration_ms=cfg.whisper_silence_ms,
    )

    server = WhisperServer(
        transcriber=transcriber,
        host=cfg.host,
        port=cfg.whisper_port,
    )

    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    # Graceful shutdown on SIGTERM / SIGINT
    for sig in (signal.SIGTERM, signal.SIGINT):
        try:
            loop.add_signal_handler(sig, loop.stop)
        except NotImplementedError:
            # Windows doesn't support add_signal_handler for all signals
            pass

    try:
        loop.run_until_complete(server.start())
    except (asyncio.CancelledError, KeyboardInterrupt):
        logger.info("Whisper service stopping...")
    finally:
        loop.close()
        logger.info("Whisper service stopped.")


if __name__ == "__main__":
    main()
