"""
flowlocal_llm.main — Entry point for the LLM service.
"""

from __future__ import annotations

import asyncio
import logging
import signal

from flowlocal_shared.config import load as load_config
from flowlocal_shared.logging import setup_logging

from .cleaner import TextCleaner
from .server import LlmServer


def main() -> None:
    cfg = load_config()
    setup_logging(cfg.log_level, "llm")
    logger = logging.getLogger(__name__)

    logger.info("FlowLocal LLM Service starting...")
    logger.info(
        "Ollama: %s | Model: %s",
        cfg.ollama_host,
        cfg.ollama_model,
    )

    cleaner = TextCleaner(
        ollama_host=cfg.ollama_host,
        ollama_model=cfg.ollama_model,
        temperature=cfg.ollama_temperature,
        max_tokens=cfg.ollama_max_tokens,
        timeout=cfg.ollama_timeout,
    )

    server = LlmServer(
        cleaner=cleaner,
        host=cfg.host,
        port=cfg.llm_port,
    )

    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    for sig in (signal.SIGTERM, signal.SIGINT):
        try:
            loop.add_signal_handler(sig, loop.stop)
        except NotImplementedError:
            pass

    # Warm-up Ollama health check
    async def _startup():
        ok = await cleaner.health_check()
        if not ok:
            logger.warning(
                "Ollama model '%s' not ready — service will retry per request",
                cfg.ollama_model,
            )
        await server.start()

    try:
        loop.run_until_complete(_startup())
    except (asyncio.CancelledError, KeyboardInterrupt):
        logger.info("LLM service stopping...")
    finally:
        loop.close()
        logger.info("LLM service stopped.")


if __name__ == "__main__":
    main()
