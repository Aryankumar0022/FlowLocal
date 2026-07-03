"""
flowlocal_shared.logging — Structured logging setup.

Call setup_logging(level, service) once at process startup.
"""

from __future__ import annotations

import logging
import sys


def setup_logging(level: str = "INFO", service: str = "flowlocal") -> None:
    """Configure the root logger with a clean, structured format."""
    fmt = f"[%(asctime)s] [{service}] %(levelname)-8s %(name)s — %(message)s"
    datefmt = "%H:%M:%S"

    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(logging.Formatter(fmt=fmt, datefmt=datefmt))

    root = logging.getLogger()
    root.setLevel(getattr(logging, level.upper(), logging.INFO))
    root.handlers.clear()
    root.addHandler(handler)

    # Suppress noisy third-party loggers
    for noisy in ("httpx", "httpcore", "chromadb", "urllib3", "asyncio"):
        logging.getLogger(noisy).setLevel(logging.WARNING)
