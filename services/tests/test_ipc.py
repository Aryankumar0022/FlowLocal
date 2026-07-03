import pytest
import asyncio
import io
import json
from typing import cast
from flowlocal_shared.ipc import read_message, send_message

@pytest.mark.asyncio
async def test_send_message():
    # We will simulate a writer stream
    class MockWriter:
        def __init__(self):
            self.buffer = bytearray()
        def write(self, data: bytes):
            self.buffer.extend(data)
        async def drain(self):
            pass

    writer = MockWriter()
    msg = {"type": "test", "payload": {"foo": "bar"}}
    await send_message(cast(asyncio.StreamWriter, writer), msg)
    
    # 4 bytes for length (little-endian), followed by JSON bytes
    assert len(writer.buffer) > 4
    length = int.from_bytes(writer.buffer[:4], byteorder='little')
    payload = writer.buffer[4:].decode('utf-8')
    assert length == len(payload)
    
    decoded = json.loads(payload)
    assert decoded["type"] == "test"
    assert decoded["payload"]["foo"] == "bar"

@pytest.mark.asyncio
async def test_read_message():
    class MockReader:
        def __init__(self, data: bytes):
            self.data = data
            self.pos = 0
            
        async def readexactly(self, n: int) -> bytes:
            if self.pos + n > len(self.data):
                raise asyncio.IncompleteReadError(self.data[self.pos:], n)
            chunk = self.data[self.pos:self.pos+n]
            self.pos += n
            return chunk

    msg = {"action": "transcribe"}
    payload = json.dumps(msg).encode('utf-8')
    length = len(payload).to_bytes(4, byteorder='little')
    
    reader = MockReader(length + payload)
    
    result = await read_message(cast(asyncio.StreamReader, reader))
    assert result == msg
