"""
flowlocal_whisper.server — TCP server that handles the IPC protocol.

Lifecycle:
  1. Rust sends AudioChunk messages (one per ~100ms of audio)
  2. Transcriber buffers the PCM samples
  3. Rust sends AudioEnd → server transcribes in a thread pool
  4. Partial transcripts sent per segment as they stream
  5. Final TranscriptFinal sent when done

Multiple concurrent sessions are supported.
"""

from __future__ import annotations

import asyncio
import logging
from concurrent.futures import ThreadPoolExecutor

from flowlocal_shared.ipc import make_error, make_pong, make_ready, read_messages, send_message

from .transcriber import Transcriber

logger = logging.getLogger(__name__)

VERSION = "0.1.0"
SERVICE_NAME = "whisper"

# Thread pool for running faster-whisper (CPU/GPU bound)
_EXECUTOR = ThreadPoolExecutor(max_workers=1, thread_name_prefix="whisper-infer")


class WhisperServer:
    def __init__(self, transcriber: Transcriber, host: str, port: int) -> None:
        self.transcriber = transcriber
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
        logger.info("Whisper service listening on %s", addrs)
        logger.info("Whisper service READY OK")

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
            logger.exception("Unhandled error for client %s: %s", addr, exc)
        finally:
            try:
                writer.close()
            except Exception:
                pass
            logger.info("Client disconnected: %s", addr)

    async def _dispatch(self, msg: dict, writer: asyncio.StreamWriter) -> None:
        msg_type = msg.get("type", "")
        session_id = msg.get("session_id")

        try:
            if msg_type == "ping":
                await send_message(writer, make_pong(SERVICE_NAME))

            elif msg_type == "audio_chunk":
                # Buffer the chunk — no response
                self.transcriber.add_chunk(
                    session_id=session_id,
                    data_b64=msg["data"],
                    sample_rate=int(msg.get("sample_rate", 44100)),
                )

            elif msg_type == "audio_end":
                await self._transcribe_and_reply(session_id, writer)

            elif msg_type == "shutdown":
                logger.info("Shutdown requested by client")
                raise asyncio.CancelledError

            else:
                logger.warning("Unknown message type: %s", msg_type)
                await send_message(
                    writer,
                    make_error(session_id, 400, f"Unknown message type: {msg_type}"),
                )

        except asyncio.CancelledError:
            raise
        except Exception as exc:
            logger.exception("Error dispatching '%s': %s", msg_type, exc)
            try:
                await send_message(writer, make_error(session_id, 500, str(exc)))
            except Exception:
                pass

    async def _transcribe_and_reply(
        self, session_id: str, writer: asyncio.StreamWriter
    ) -> None:
        """Run transcription and stream results back to the Rust bridge."""
        loop = asyncio.get_running_loop()

        # Partial transcript callback — runs in the inference thread
        # We schedule it on the event loop so we can do async I/O
        async def _send_partial(sid: str, text: str) -> None:
            try:
                await send_message(
                    writer,
                    {"type": "transcript_partial", "session_id": sid, "text": text},
                )
            except Exception as e:
                logger.warning("Failed to send partial transcript: %s", e)

        # Run finalize() in the thread pool (it's CPU-bound)
        # We pass a sync callback that schedules the async send on the loop
        def _sync_on_partial(sid: str, text: str) -> None:
            asyncio.run_coroutine_threadsafe(_send_partial(sid, text), loop)

        try:
            text, language, duration_ms = await loop.run_in_executor(
                _EXECUTOR,
                lambda: _finalize_sync(self.transcriber, session_id, _sync_on_partial),
            )
        except Exception as e:
            logger.exception("Transcription failed for session %s: %s", session_id, e)
            await send_message(writer, make_error(session_id, 500, f"Transcription error: {e}"))
            return

        await send_message(
            writer,
            {
                "type": "transcript_final",
                "session_id": session_id,
                "text": text,
                "language": language,
                "duration_ms": duration_ms,
            },
        )
        logger.info(
            "Session %s done: '%s...' (%dms)",
            session_id,
            text[:60],
            duration_ms,
        )


def _finalize_sync(
    transcriber: Transcriber,
    session_id: str,
    on_partial,
) -> tuple[str, str, int]:
    """Synchronous wrapper for Transcriber.finalize, called in a thread pool."""
    import asyncio as _asyncio

    # Create a temporary event loop for this thread
    inner_loop = _asyncio.new_event_loop()

    async def _run():
        async def _partial_cb(sid: str, text: str) -> None:
            on_partial(sid, text)

        return await transcriber.finalize(session_id, on_partial=_partial_cb)

    try:
        return inner_loop.run_until_complete(_run())
    finally:
        inner_loop.close()
