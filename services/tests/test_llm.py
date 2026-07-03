import pytest
from flowlocal_llm.cleaner import TextCleaner, _strip_preambles

def test_command_detection():
    cleaner = TextCleaner(ollama_host="http://localhost:11434", ollama_model="test")
    
    # Test standard text
    is_cmd, text = cleaner.detect_command("This is just some text.")
    assert is_cmd is None
    
    # Test built-in command prefix mapping
    is_cmd, text = cleaner.detect_command("Make it professional this is my draft")
    assert is_cmd == "rewrite_professional"
    assert text == "this is my draft"
    
    is_cmd, text = cleaner.detect_command("summarize this long text")
    assert is_cmd == "summarize"
    assert text == "this long text"
    
    # Case insensitivity
    is_cmd, text = cleaner.detect_command("TRANSLATE TO FRENCH hello world")
    assert is_cmd == "translate_french"
    assert text == "hello world"

@pytest.mark.asyncio
async def test_strip_preamble():
    text = "Here is the cleaned text:\n\nThis is the actual output."
    assert _strip_preambles(text) == "This is the actual output."
    
    # Should not strip if there is no clear preamble break
    text = "This is just one single paragraph without a double newline."
    assert _strip_preambles(text) == text
