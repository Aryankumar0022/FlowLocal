"""
flowlocal_llm.server — TCP server for the LLM cleanup service.
"""

from __future__ import annotations

import asyncio
import logging
import sys
from pathlib import Path

_SHARED_DIR = Path(__file__).resolve().parents[2] / "shared"
if _SHARED_DIR.exists() and str(_SHARED_DIR) not in sys.path:
    sys.path.insert(0, str(_SHARED_DIR))

# pyrefly: ignore [missing-import]
from flowlocal_shared.ipc import make_error, make_pong, make_ready, read_messages, send_message

from .cleaner import TextCleaner

logger = logging.getLogger(__name__)

VERSION = "0.1.0"
SERVICE_NAME = "llm"


class LlmServer:
    def __init__(self, cleaner: TextCleaner, host: str, port: int) -> None:
        self.cleaner = cleaner
        self.host = host
        self.port = port

    async def start(self) -> None:
        server = await asyncio.start_server(
            self._handle_client,
            self.host,
            self.port,
            reuse_address=True,
        )
        addrs = [str(s.getsockname()) for s in server.sockets]
        logger.info("LLM service listening on %s", addrs)
        logger.info("LLM service READY OK")

        async with server:
            await server.serve_forever()

    async def _handle_client(
        self,
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter,
    ) -> None:
        addr = writer.get_extra_info("peername", "unknown")
        logger.info("Client connected: %s", addr)
        await send_message(writer, make_ready(SERVICE_NAME, VERSION))

        try:
            async for msg in read_messages(reader):
                await self._dispatch(msg, writer)
        except Exception as exc:
            logger.exception("Client error: %s", exc)
        finally:
            writer.close()
            logger.info("Client disconnected: %s", addr)

    async def _dispatch(
        self,
        msg: dict,
        writer: asyncio.StreamWriter,
    ) -> None:
        msg_type = msg.get("type", "")
        session_id = msg.get("session_id")

        try:
            if msg_type == "ping":
                await send_message(writer, make_pong(SERVICE_NAME))

            elif msg_type == "clean_text":
                text = await self.cleaner.clean(
                    raw_text=msg["raw_text"],
                    app_context=msg.get("app_context", "generic"),
                    language=msg.get("language", "en"),
                    aggressiveness=msg.get("aggressiveness", "moderate"),
                    rag_context=msg.get("rag_context", []),
                    dictionary_terms=msg.get("dictionary_terms", []),
                    remove_fillers=msg.get("remove_fillers", True),
                    fix_punctuation=msg.get("fix_punctuation", True),
                    fix_capitalization=msg.get("fix_capitalization", True),
                )
                await send_message(
                    writer,
                    {"type": "text_ready", "session_id": session_id, "text": text},
                )

            elif msg_type == "execute_command":
                text = await self.cleaner.execute_command(
                    command=msg["command"],
                    text=msg["text"],
                    language=msg.get("language", "en"),
                )
                await send_message(
                    writer,
                    {
                        "type": "command_result",
                        "session_id": session_id,
                        "command": msg["command"],
                        "text": text,
                    },
                )

            elif msg_type == "shutdown":
                logger.info("Shutdown requested")
                raise asyncio.CancelledError

            else:
                await send_message(
                    writer,
                    make_error(session_id, 400, f"Unknown message: {msg_type}"),
                )

        except asyncio.CancelledError:
            raise
        except Exception as exc:
            logger.exception("Error handling '%s': %s", msg_type, exc)
            await send_message(writer, make_error(session_id, 500, str(exc)))
