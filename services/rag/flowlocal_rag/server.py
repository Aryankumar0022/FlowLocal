"""
flowlocal_rag.server — TCP server for the RAG service.
"""

from __future__ import annotations

import asyncio
import logging

from flowlocal_shared.ipc import make_error, make_pong, make_ready, read_messages, send_message

from .store import CorrectionStore

logger = logging.getLogger(__name__)

VERSION = "0.1.0"
SERVICE_NAME = "rag"


class RagServer:
    def __init__(self, store: CorrectionStore, host: str, port: int) -> None:
        self.store = store
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
        logger.info("RAG service listening on %s", addrs)
        logger.info("RAG service READY OK")

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

            elif msg_type == "store_correction":
                # Fire-and-forget: Rust does not wait for a response
                asyncio.create_task(
                    self.store.store(
                        session_id=session_id,
                        raw_text=msg["raw_text"],
                        clean_text=msg["clean_text"],
                        app_context=msg.get("app_context", "generic"),
                        language=msg.get("language", "en"),
                    )
                )
                # No reply needed

            elif msg_type == "retrieve_context":
                segments = await self.store.retrieve(
                    query_text=msg["query_text"],
                    max_results=int(msg.get("max_results", 5)),
                )
                await send_message(
                    writer,
                    {
                        "type": "context_ready",
                        "session_id": session_id,
                        "segments": segments,
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
