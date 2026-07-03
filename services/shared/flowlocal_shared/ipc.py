"""
flowlocal_shared.ipc — Length-prefixed JSON IPC protocol

Wire format: [u32 LE length][UTF-8 JSON payload]
Matches exactly the Rust IpcBridge frame format.
"""

from __future__ import annotations

import asyncio
import json
import struct
from typing import Any


async def read_message(reader: asyncio.StreamReader) -> dict[str, Any]:
    """Read one length-prefixed JSON message from the stream.

    Returns the decoded dict, or raises if the connection is closed.
    """
    # Read 4-byte LE length prefix
    raw_len = await reader.readexactly(4)
    (length,) = struct.unpack("<I", raw_len)

    # Read payload
    payload = await reader.readexactly(length)
    return json.loads(payload.decode("utf-8"))


async def send_message(writer: asyncio.StreamWriter, msg: dict[str, Any]) -> None:
    """Encode and write one length-prefixed JSON message."""
    payload = json.dumps(msg, ensure_ascii=False).encode("utf-8")
    length_prefix = struct.pack("<I", len(payload))
    writer.write(length_prefix + payload)
    await writer.drain()


async def read_messages(reader: asyncio.StreamReader):
    """Async generator that yields decoded messages until connection closes."""
    try:
        while True:
            yield await read_message(reader)
    except (asyncio.IncompleteReadError, ConnectionResetError):
        return


def make_error(session_id: str | None, code: int, message: str) -> dict:
    return {
        "type": "service_error",
        "session_id": session_id,
        "code": code,
        "message": message,
    }


def make_pong(service: str) -> dict:
    return {"type": "pong", "service": service}


def make_ready(service: str, version: str) -> dict:
    return {"type": "ready", "service": service, "version": version}
