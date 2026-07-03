"""
flowlocal_rag.main — Entry point for the RAG service.
"""

from __future__ import annotations

import asyncio
import logging
import signal

from flowlocal_shared.config import load as load_config
from flowlocal_shared.logging import setup_logging

from .embedder import Embedder
from .server import RagServer
from .store import CorrectionStore


def main() -> None:
    cfg = load_config()
    setup_logging(cfg.log_level, "rag")
    logger = logging.getLogger(__name__)

    logger.info("FlowLocal RAG Service starting...")
    logger.info(
        "ChromaDB path: %s | Embedding model: %s | Ollama: %s",
        cfg.chroma_path,
        cfg.embedding_model,
        cfg.ollama_host,
    )

    embedder = Embedder(
        ollama_host=cfg.ollama_host,
        model=cfg.embedding_model,
    )

    store = CorrectionStore(
        chroma_path=cfg.chroma_path,
        embedder=embedder,
    )

    # Initialize ChromaDB synchronously
    store.initialize()

    server = RagServer(
        store=store,
        host=cfg.host,
        port=cfg.rag_port,
    )

    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    for sig in (signal.SIGTERM, signal.SIGINT):
        try:
            loop.add_signal_handler(sig, loop.stop)
        except NotImplementedError:
            pass

    async def _startup():
        # Verify embedder is reachable
        ok = await embedder.health_check()
        if not ok:
            logger.warning(
                "Ollama embedding model '%s' not ready — will retry per request",
                cfg.embedding_model,
            )
        await server.start()

    try:
        loop.run_until_complete(_startup())
    except (asyncio.CancelledError, KeyboardInterrupt):
        logger.info("RAG service stopping...")
    finally:
        loop.close()
        logger.info("RAG service stopped.")


if __name__ == "__main__":
    main()
