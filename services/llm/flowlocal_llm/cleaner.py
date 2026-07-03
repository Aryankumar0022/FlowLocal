"""
flowlocal_llm.cleaner — Text cleanup via Ollama.

Builds context-aware prompts using Jinja2 templates,
calls the Ollama /api/generate endpoint, and returns
the cleaned text.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path

import httpx
from jinja2 import Environment, FileSystemLoader, select_autoescape

logger = logging.getLogger(__name__)

PROMPTS_DIR = Path(__file__).parent / "prompts"

# Voice command prefixes that trigger command mode instead of cleanup
COMMAND_PREFIXES: dict[str, str] = {
    "rewrite professionally": "rewrite_professional",
    "rewrite this professionally": "rewrite_professional",
    "make it professional": "rewrite_professional",
    "make professional": "rewrite_professional",
    "rewrite casually": "rewrite_casual",
    "summarize": "summarize",
    "summarize this": "summarize",
    "create summary": "summarize",
    "bullet points": "bullet_points",
    "make bullet points": "bullet_points",
    "convert to bullets": "bullet_points",
    "expand": "expand",
    "expand this": "expand",
    "elaborate": "expand",
    "make shorter": "shorten",
    "make it shorter": "shorten",
    "shorten this": "shorten",
    "be concise": "shorten",
    "fix grammar": "fix_grammar",
    "fix the grammar": "fix_grammar",
    "generate commit message": "generate_commit",
    "write commit message": "generate_commit",
    "generate documentation": "generate_docs",
    "write docs": "generate_docs",
    "translate to hindi": "translate_hindi",
    "translate to english": "translate_english",
    "translate to spanish": "translate_spanish",
    "translate to french": "translate_french",
}


class TextCleaner:
    def __init__(
        self,
        ollama_host: str,
        ollama_model: str,
        temperature: float = 0.1,
        max_tokens: int = 2048,
        timeout: float = 60.0,
    ) -> None:
        self.ollama_host = ollama_host.rstrip("/")
        self.ollama_model = ollama_model
        self.temperature = temperature
        self.max_tokens = max_tokens
        self.timeout = timeout

        self._jinja = Environment(
            loader=FileSystemLoader(str(PROMPTS_DIR)),
            autoescape=select_autoescape([]),
            trim_blocks=True,
            lstrip_blocks=True,
        )

    # ── Public API ─────────────────────────────────────────────

    def detect_command(self, text: str) -> tuple[str | None, str]:
        """Check if text starts with a known command phrase.

        Returns (command_name, remaining_text) or (None, original_text).
        """
        normalized = re.sub(r"\s+", " ", text.lower()).strip()
        for prefix, command in COMMAND_PREFIXES.items():
            if normalized.startswith(prefix):
                remaining = text[len(prefix):].strip(" ,.:;-")
                return command, remaining
        return None, text

    async def clean(
        self,
        raw_text: str,
        app_context: str = "generic",
        language: str = "en",
        aggressiveness: str = "moderate",
        rag_context: list[str] | None = None,
        dictionary_terms: list[list[str]] | None = None,
        remove_fillers: bool = True,
        fix_punctuation: bool = True,
        fix_capitalization: bool = True,
    ) -> str:
        """Clean transcription text using the Ollama LLM."""
        if not raw_text.strip():
            return raw_text

        # Check for inline command
        command, content = self.detect_command(raw_text)
        if command and content:
            return await self.execute_command(command, content, language)

        template = self._jinja.get_template("cleanup.txt")
        prompt = template.render(
            raw_text=raw_text,
            app_context=app_context,
            language=language,
            aggressiveness=aggressiveness,
            rag_context=rag_context or [],
            dictionary_terms=dictionary_terms or [],
            remove_fillers=remove_fillers,
            fix_punctuation=fix_punctuation,
            fix_capitalization=fix_capitalization,
        )

        logger.debug("Cleanup prompt length: %d chars", len(prompt))
        result = await self._call_ollama(prompt)

        # Safety: if result is way longer than input, something went wrong
        if len(result) > len(raw_text) * 3:
            logger.warning("LLM output suspiciously long — using raw text")
            return raw_text

        return result

    async def execute_command(
        self,
        command: str,
        text: str,
        language: str = "en",
    ) -> str:
        """Execute a named transformation command on text."""
        if not text.strip():
            return text

        template = self._jinja.get_template("command.txt")
        prompt = template.render(command=command, text=text, language=language)

        logger.info("Executing command: %s (%d chars)", command, len(text))
        return await self._call_ollama(prompt)

    # ── Ollama API ─────────────────────────────────────────────

    async def _call_ollama(self, prompt: str) -> str:
        """Call the Ollama /api/generate endpoint and return the response text."""
        url = f"{self.ollama_host}/api/generate"
        payload = {
            "model": self.ollama_model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": self.temperature,
                "num_predict": self.max_tokens,
                "top_p": 0.9,
                "repeat_penalty": 1.1,
            },
        }

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            try:
                resp = await client.post(url, json=payload)
                resp.raise_for_status()
                data = resp.json()
                text = data.get("response", "").strip()

                # Strip common LLM preambles
                text = _strip_preambles(text)

                logger.debug("Ollama response: %d chars", len(text))
                return text

            except httpx.TimeoutException:
                logger.error("Ollama request timed out after %.1fs", self.timeout)
                raise
            except httpx.HTTPStatusError as e:
                logger.error("Ollama HTTP error: %s", e)
                raise
            except Exception as e:
                logger.exception("Ollama call failed: %s", e)
                raise

    async def health_check(self) -> bool:
        """Return True if Ollama is reachable and the model is available."""
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                resp = await client.get(f"{self.ollama_host}/api/tags")
                resp.raise_for_status()
                models = [m["name"] for m in resp.json().get("models", [])]
                available = any(
                    m.startswith(self.ollama_model.split(":")[0]) for m in models
                )
                if not available:
                    logger.warning(
                        "Model '%s' not found. Available: %s",
                        self.ollama_model,
                        models,
                    )
                return available
        except Exception as e:
            logger.error("Ollama health check failed: %s", e)
            return False


def _strip_preambles(text: str) -> str:
    """Remove common LLM preambles like 'Here is the cleaned text:' etc."""
    patterns = [
        r"^(?:here is|here's|the cleaned|cleaned text|output|result)[^\n]*:\s*",
        r"^```[^\n]*\n",
        r"\n```\s*$",
    ]
    for pattern in patterns:
        text = re.sub(pattern, "", text, flags=re.IGNORECASE).strip()
    return text
