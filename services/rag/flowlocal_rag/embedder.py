"""
flowlocal_rag.embedder — Generate text embeddings via Ollama.

Uses the /api/embeddings endpoint with the configured embedding model
(default: nomic-embed-text which produces 768-dim vectors).
"""

from __future__ import annotations

import logging
from functools import lru_cache

import httpx

logger = logging.getLogger(__name__)


class Embedder:
    def __init__(
        self,
        ollama_host: str,
        model: str = "nomic-embed-text",
        timeout: float = 30.0,
    ) -> None:
        self.ollama_host = ollama_host.rstrip("/")
        self.model = model
        self.timeout = timeout

    async def embed(self, text: str) -> list[float]:
        """Generate an embedding vector for the given text."""
        if not text.strip():
            return []

        url = f"{self.ollama_host}/api/embeddings"
        payload = {"model": self.model, "prompt": text}

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            resp = await client.post(url, json=payload)
            resp.raise_for_status()
            data = resp.json()
            embedding = data.get("embedding", [])

        if not embedding:
            raise ValueError(f"Empty embedding returned for model '{self.model}'")

        logger.debug("Embedding: %d dims for %d chars", len(embedding), len(text))
        return embedding

    async def embed_batch(self, texts: list[str]) -> list[list[float]]:
        """Embed multiple texts. Returns one vector per text."""
        results = []
        for text in texts:
            vec = await self.embed(text)
            results.append(vec)
        return results

    async def health_check(self) -> bool:
        """Return True if the embedding model is available."""
        try:
            await self.embed("test")
            return True
        except Exception as e:
            logger.warning("Embedding health check failed: %s", e)
            return False
