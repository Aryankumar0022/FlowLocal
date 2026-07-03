"""
flowlocal_rag.store — ChromaDB vector store for past corrections.

Each stored document represents a raw→clean correction pair.
On retrieval, we return the `clean_text` of the most similar past sessions
so the LLM can use them as few-shot examples.
"""

from __future__ import annotations

import logging
import os
from pathlib import Path

logger = logging.getLogger(__name__)

COLLECTION_NAME = "corrections"


class CorrectionStore:
    def __init__(
        self,
        chroma_path: str = "./data/chroma",
        embedder=None,
    ) -> None:
        self.chroma_path = chroma_path
        self.embedder = embedder
        self._collection = None

    def initialize(self) -> None:
        """Connect to (or create) the ChromaDB persistent store."""
        import chromadb

        Path(self.chroma_path).mkdir(parents=True, exist_ok=True)

        client = chromadb.PersistentClient(path=self.chroma_path)
        self._collection = client.get_or_create_collection(
            name=COLLECTION_NAME,
            metadata={"hnsw:space": "cosine"},
        )
        count = self._collection.count()
        logger.info(
            "ChromaDB initialized at '%s': %d corrections stored",
            self.chroma_path,
            count,
        )

    async def store(
        self,
        session_id: str,
        raw_text: str,
        clean_text: str,
        app_context: str = "generic",
        language: str = "en",
    ) -> None:
        """Store a raw→clean correction pair as a vector document."""
        if not raw_text.strip() or not clean_text.strip():
            return

        # We embed the raw text so we can retrieve by similar raw queries later
        try:
            embedding = await self.embedder.embed(raw_text)
        except Exception as e:
            logger.error("Failed to embed text for RAG storage: %s", e)
            return

        if not embedding:
            return

        metadata = {
            "raw_text": raw_text[:1000],  # ChromaDB metadata value limit
            "clean_text": clean_text[:1000],
            "app_context": app_context,
            "language": language,
        }

        try:
            # Upsert by session_id (idempotent)
            self._collection.upsert(
                ids=[session_id],
                embeddings=[embedding],
                documents=[raw_text],
                metadatas=[metadata],
            )
            logger.debug("Stored correction for session %s", session_id)
        except Exception as e:
            logger.error("ChromaDB upsert failed: %s", e)

    async def retrieve(
        self,
        query_text: str,
        max_results: int = 5,
        min_similarity: float = 0.7,
    ) -> list[str]:
        """Return the `clean_text` of the most similar past corrections.

        Filters out results below `min_similarity` to avoid low-quality context.
        """
        if not query_text.strip():
            return []

        count = self._collection.count()
        if count == 0:
            return []

        try:
            embedding = await self.embedder.embed(query_text)
        except Exception as e:
            logger.error("Failed to embed query for RAG retrieval: %s", e)
            return []

        if not embedding:
            return []

        n = min(max_results, count)
        try:
            results = self._collection.query(
                query_embeddings=[embedding],
                n_results=n,
                include=["metadatas", "distances"],
            )
        except Exception as e:
            logger.error("ChromaDB query failed: %s", e)
            return []

        segments: list[str] = []
        metadatas = results.get("metadatas", [[]])[0]
        distances = results.get("distances", [[]])[0]

        for meta, dist in zip(metadatas, distances):
            # ChromaDB cosine distance: 0 = identical, 2 = opposite
            # Convert to similarity: 1 - dist/2
            similarity = 1.0 - (dist / 2.0)
            if similarity < min_similarity:
                continue
            clean = meta.get("clean_text", "")
            if clean:
                segments.append(clean)

        logger.debug(
            "RAG retrieved %d/%d segments (similarity >= %.2f)",
            len(segments),
            n,
            min_similarity,
        )
        return segments
