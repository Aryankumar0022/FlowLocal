# FlowLocal — Production Architecture Plan

> **Stage 1 of 8** — Architecture & System Design

---

## Overview

FlowLocal is a production-grade, fully offline, local-first voice dictation desktop application modeled after Wispr Flow. It targets Windows 11 (primary) and Linux (secondary), runs all inference locally via Ollama + faster-whisper, and is built on Tauri 2 + React + Rust + Python microservices.

---

## System Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                          FlowLocal Desktop App                        │
│                                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    Tauri 2 Shell (Rust)                          │  │
│  │                                                                   │  │
│  │  ┌──────────────────┐    ┌───────────────────────────────────┐  │  │
│  │  │  React Frontend  │◄──►│      Rust Core Backend            │  │  │
│  │  │  (TypeScript)    │    │                                   │  │  │
│  │  │                  │    │  ┌─────────────────────────────┐  │  │  │
│  │  │  • Overlay UI    │    │  │  hotkey_manager             │  │  │  │
│  │  │  • Settings      │    │  │  audio_capture              │  │  │  │
│  │  │  • Dictionary    │    │  │  text_injector              │  │  │  │
│  │  │  • Live Preview  │    │  │  ipc_bridge                 │  │  │  │
│  │  │  • History       │    │  │  context_detector           │  │  │  │
│  │  │  • System Tray   │    │  │  settings_store             │  │  │  │
│  │  └──────────────────┘    │  │  db_manager                 │  │  │  │
│  │                          │  └─────────────────────────────┘  │  │  │
│  │                          └───────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                    │                                   │
│                     Unix Socket / Named Pipe IPC                       │
│                                    │                                   │
│  ┌─────────────────────────────────▼───────────────────────────────┐  │
│  │                    Python AI Microservices                        │  │
│  │                                                                   │  │
│  │  ┌──────────────────┐    ┌──────────────────┐                   │  │
│  │  │  Whisper Service │    │   LLM Service    │                   │  │
│  │  │  (faster-whisper)│    │  (Ollama bridge) │                   │  │
│  │  │  • transcribe()  │    │  • cleanup()     │                   │  │
│  │  │  • stream()      │    │  • command()     │                   │  │
│  │  │  • VAD filter    │    │  • format()      │                   │  │
│  │  └──────────────────┘    └──────────────────┘                   │  │
│  │                                                                   │  │
│  │  ┌──────────────────┐    ┌──────────────────┐                   │  │
│  │  │   RAG Service    │    │  Dictionary Svc  │                   │  │
│  │  │  (ChromaDB)      │    │  (SQLite)        │                   │  │
│  │  │  • store()       │    │  • apply()       │                   │  │
│  │  │  • retrieve()    │    │  • learn()       │                   │  │
│  │  │  • embed()       │    │  • export()      │                   │  │
│  │  └──────────────────┘    └──────────────────┘                   │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                    │                                   │
│                            Ollama HTTP API                             │
│                          (localhost:11434)                             │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Repository Structure

```
flowlocal/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── release-windows.yml
│       └── release-linux.yml
│
├── apps/
│   └── desktop/                        # Tauri 2 app root
│       ├── src-tauri/                  # Rust backend
│       │   ├── Cargo.toml
│       │   ├── tauri.conf.json
│       │   ├── build.rs
│       │   └── src/
│       │       ├── main.rs
│       │       ├── lib.rs
│       │       ├── hotkey/
│       │       │   ├── mod.rs
│       │       │   └── manager.rs
│       │       ├── audio/
│       │       │   ├── mod.rs
│       │       │   └── capture.rs
│       │       ├── inject/
│       │       │   ├── mod.rs
│       │       │   ├── windows.rs
│       │       │   └── linux.rs
│       │       ├── context/
│       │       │   ├── mod.rs
│       │       │   ├── detector.rs
│       │       │   └── app_map.rs
│       │       ├── ipc/
│       │       │   ├── mod.rs
│       │       │   ├── bridge.rs
│       │       │   └── protocol.rs
│       │       ├── db/
│       │       │   ├── mod.rs
│       │       │   ├── manager.rs
│       │       │   └── migrations/
│       │       │       ├── 001_initial.sql
│       │       │       ├── 002_dictionary.sql
│       │       │       ├── 003_history.sql
│       │       │       └── 004_memory.sql
│       │       ├── settings/
│       │       │   ├── mod.rs
│       │       │   └── store.rs
│       │       ├── tray/
│       │       │   ├── mod.rs
│       │       │   └── menu.rs
│       │       └── commands/           # Tauri IPC commands
│       │           ├── mod.rs
│       │           ├── audio_commands.rs
│       │           ├── dict_commands.rs
│       │           ├── settings_commands.rs
│       │           └── history_commands.rs
│       │
│       └── src/                        # React frontend
│           ├── index.html
│           ├── main.tsx
│           ├── App.tsx
│           ├── vite.config.ts
│           ├── tsconfig.json
│           ├── components/
│           │   ├── Overlay/
│           │   │   ├── Overlay.tsx
│           │   │   ├── Waveform.tsx
│           │   │   └── LiveTranscript.tsx
│           │   ├── Settings/
│           │   │   ├── Settings.tsx
│           │   │   ├── ModelSettings.tsx
│           │   │   ├── HotkeySettings.tsx
│           │   │   ├── DictionarySettings.tsx
│           │   │   └── LanguageSettings.tsx
│           │   ├── History/
│           │   │   ├── History.tsx
│           │   │   └── HistoryItem.tsx
│           │   └── shared/
│           │       ├── Button.tsx
│           │       ├── Input.tsx
│           │       └── Toggle.tsx
│           ├── hooks/
│           │   ├── useHotkey.ts
│           │   ├── useRecording.ts
│           │   ├── useSettings.ts
│           │   └── useHistory.ts
│           ├── stores/
│           │   ├── recording.store.ts
│           │   ├── settings.store.ts
│           │   └── history.store.ts
│           ├── services/
│           │   ├── tauri.service.ts
│           │   └── events.service.ts
│           └── styles/
│               ├── globals.css
│               ├── overlay.css
│               └── settings.css
│
├── services/
│   ├── whisper/                        # faster-whisper microservice
│   │   ├── pyproject.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.py
│   │       ├── server.py
│   │       ├── transcriber.py
│   │       ├── vad.py
│   │       └── models.py
│   │
│   ├── llm/                            # Ollama LLM microservice
│   │   ├── pyproject.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.py
│   │       ├── server.py
│   │       ├── cleaner.py
│   │       ├── commander.py
│   │       ├── formatter.py
│   │       └── prompts/
│   │           ├── cleanup.txt
│   │           ├── command.txt
│   │           └── context/
│   │               ├── code.txt
│   │               ├── email.txt
│   │               ├── notes.txt
│   │               ├── chat.txt
│   │               └── terminal.txt
│   │
│   ├── rag/                            # ChromaDB RAG microservice
│   │   ├── pyproject.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.py
│   │       ├── server.py
│   │       ├── store.py
│   │       └── embedder.py
│   │
│   └── shared/                         # Shared Python utilities
│       ├── pyproject.toml
│       └── src/
│           ├── ipc.py
│           ├── logging.py
│           └── config.py
│
├── data/
│   ├── models/                         # Whisper model cache
│   ├── chroma/                         # ChromaDB vector store
│   └── db/                             # SQLite databases
│
├── scripts/
│   ├── install.ps1                     # Windows install script
│   ├── install.sh                      # Linux install script
│   ├── dev.ps1                         # Windows dev runner
│   ├── dev.sh                          # Linux dev runner
│   ├── build.ps1                       # Windows build
│   ├── build.sh                        # Linux build
│   ├── setup-ollama.ps1
│   └── setup-ollama.sh
│
├── docker/
│   ├── docker-compose.yml
│   ├── docker-compose.dev.yml
│   └── Dockerfile.services
│
├── tests/
│   ├── unit/
│   │   ├── rust/
│   │   └── python/
│   ├── integration/
│   └── e2e/
│
├── docs/
│   ├── architecture.md
│   ├── development.md
│   ├── deployment.md
│   └── api.md
│
├── .env.example
├── .gitignore
├── README.md
└── CONTRIBUTING.md
```

---

## Data Flow Diagrams

### Primary Flow: Voice → Text → Inject

```
User Presses Hotkey
       │
       ▼
[Rust] hotkey_manager::on_press()
       │
       ├──► Emit Tauri event: "recording:start"
       │         │
       │         ▼
       │    [React] Overlay appears + waveform starts
       │
       ▼
[Rust] audio::capture::start_stream()
       │ (WASAPI on Windows / ALSA on Linux)
       │
       ▼ PCM chunks (16kHz, mono, f32)
       │
       ▼
[Rust] ipc::bridge::send_audio_chunk(chunk)
       │ (Unix socket / named pipe)
       │
       ▼
[Python] whisper/vad.py::process_chunk()
       │
       ├──► If speech detected → buffer
       ├──► If silence threshold → flush
       │
       ▼
[Python] whisper/transcriber.py::transcribe(buffer)
       │ (faster-whisper, CTranslate2, GPU if available)
       │
       ▼ raw_text: "uh hello um i wanted to..."
       │
       ▼
[Python] rag/store.py::retrieve_context(raw_text)
       │ (similar past corrections from ChromaDB)
       │
       ▼
[Python] llm/cleaner.py::clean(raw_text, context)
       │ (Ollama: qwen3:4b)
       │
       ▼ cleaned_text: "Hello, I wanted to..."
       │
       ▼
[Python] llm/formatter.py::format(cleaned_text, app_context)
       │ (context-aware formatting)
       │
       ▼ final_text
       │
       ▼
[Rust] ipc::bridge::on_text_ready(final_text)
       │
       ├──► Emit Tauri event: "text:ready" → React shows preview
       │
       ▼
[Rust] inject::text_injector::insert(final_text)
       │ (SendInput / xdotool)
       │
       ▼
Text appears in active application ✓
       │
       ▼
[Rust] db::manager::save_session(raw, cleaned, app, ts)
[Python] rag/store.py::store_correction(raw, cleaned)
```

### Command Mode Flow

```
User says: "rewrite professionally [text]"
       │
       ▼
[Python] whisper::transcribe()
       │
       ▼ raw: "rewrite professionally hello can we talk"
       │
       ▼
[Python] commander.py::detect_command(raw)
       │
       ├── command: "rewrite_professional"
       └── content: "hello can we talk"
       │
       ▼
[Python] commander.py::execute(command, content)
       │ (Ollama prompt template)
       │
       ▼ "Hello, could we schedule a conversation?"
       │
       ▼
[Rust] inject::insert(result)
```

---

## IPC Protocol Design

### Rust ↔ Python Communication

**Transport**: Named pipe (Windows) / Unix domain socket (Linux)

**Message format**: Length-prefixed JSON frames

```
┌─────────────────────┬──────────────────────────┐
│  4 bytes (u32 LE)   │  N bytes (UTF-8 JSON)    │
│  payload length     │  payload                 │
└─────────────────────┴──────────────────────────┘
```

**Message types** (Rust → Python):

```json
{ "type": "audio_chunk",   "data": "<base64 PCM>",  "seq": 42 }
{ "type": "audio_end",     "session_id": "uuid" }
{ "type": "ping" }
{ "type": "set_context",   "app": "vscode", "window_title": "..." }
{ "type": "execute_cmd",   "command": "rewrite_professional", "text": "..." }
```

**Message types** (Python → Rust):

```json
{ "type": "transcript_partial", "text": "hello world",  "session_id": "uuid" }
{ "type": "transcript_final",   "text": "Hello, world.", "session_id": "uuid" }
{ "type": "text_ready",         "text": "...",           "session_id": "uuid" }
{ "type": "error",              "message": "...",        "code": 500 }
{ "type": "pong" }
```

### Tauri IPC Commands (Rust → React)

```typescript
// Tauri commands exposed to frontend
invoke("start_recording")
invoke("stop_recording")
invoke("get_settings") → Settings
invoke("save_settings", { settings: Settings })
invoke("get_history", { limit: number, offset: number }) → Session[]
invoke("get_dictionary") → DictionaryEntry[]
invoke("add_dictionary_entry", { wrong: string, correct: string })
invoke("delete_dictionary_entry", { id: number })
invoke("get_ai_status") → { ollama: bool, whisper: bool, rag: bool }

// Tauri events emitted to frontend
listen("recording:start")
listen("recording:stop")
listen("transcript:partial", { text: string })
listen("transcript:final",   { text: string })
listen("text:injected",      { text: string, app: string })
listen("error",              { message: string, code: number })
```

---

## Database Schemas

### SQLite (`flowlocal.db`)

```sql
-- Migration 001: core schema
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,          -- UUID
    created_at  INTEGER NOT NULL,          -- Unix ms
    raw_text    TEXT NOT NULL,
    clean_text  TEXT NOT NULL,
    app_context TEXT NOT NULL DEFAULT '',
    window_title TEXT NOT NULL DEFAULT '',
    language    TEXT NOT NULL DEFAULT 'en',
    duration_ms INTEGER NOT NULL DEFAULT 0,
    model_used  TEXT NOT NULL DEFAULT ''
);

CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- Migration 002: dictionary
CREATE TABLE dictionary (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    wrong       TEXT NOT NULL,
    correct     TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    use_count   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(wrong)
);

CREATE INDEX idx_dictionary_wrong ON dictionary(wrong);

-- Migration 003: history metadata
CREATE TABLE history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    event_type  TEXT NOT NULL,   -- 'dictation' | 'command'
    command     TEXT,
    inserted_at INTEGER NOT NULL
);

-- Migration 004: memory / snippets
CREATE TABLE snippets (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    trigger     TEXT NOT NULL UNIQUE,
    expansion   TEXT NOT NULL,
    use_count   INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL
);

CREATE TABLE writing_patterns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern     TEXT NOT NULL,
    frequency   INTEGER NOT NULL DEFAULT 1,
    last_seen   INTEGER NOT NULL
);
```

### Settings Schema (JSON stored in `settings` table)

```typescript
interface Settings {
  // Hotkey
  hotkey: string;                       // default: "ctrl+space"
  hotkey_mode: "hold" | "toggle";      // default: "hold"

  // Models
  whisper_model: "tiny" | "base" | "small" | "medium" | "large-v3";
  ollama_model: string;                 // default: "qwen3:4b"
  ollama_host: string;                  // default: "http://localhost:11434"
  embedding_model: string;             // default: "nomic-embed-text"

  // Language
  language: string;                     // ISO 639-1, default: "en"
  auto_detect_language: boolean;

  // AI Cleanup
  cleanup_enabled: boolean;
  cleanup_aggressiveness: "light" | "moderate" | "aggressive";
  remove_fillers: boolean;
  fix_punctuation: boolean;
  fix_capitalization: boolean;

  // VAD
  vad_enabled: boolean;
  vad_threshold: number;               // 0.0–1.0, default: 0.5
  silence_duration_ms: number;         // default: 1500

  // Context
  context_aware: boolean;
  detect_active_app: boolean;

  // GPU
  use_gpu: boolean;
  gpu_device_index: number;            // default: 0

  // RAG
  rag_enabled: boolean;
  rag_max_results: number;             // default: 5

  // UI
  show_overlay: boolean;
  overlay_position: "top" | "bottom" | "cursor";
  theme: "dark" | "light" | "system";

  // Misc
  auto_start: boolean;
  telemetry: false;                    // always false
}
```

### ChromaDB Collections

```python
# Collection: "corrections"
# Stores past (raw → clean) pairs for RAG context
{
  "id": "uuid",
  "embedding": [...],           # nomic-embed-text of raw_text
  "document": "raw_text",
  "metadata": {
    "clean_text": "...",
    "app_context": "vscode",
    "language": "en",
    "timestamp": 1720000000
  }
}

# Collection: "vocabulary"
# Stores domain-specific terms for retrieval during cleanup
{
  "id": "uuid",
  "embedding": [...],
  "document": "term",
  "metadata": {
    "definition": "...",
    "category": "tech|personal|domain",
    "frequency": 5
  }
}
```

---

## Module Boundaries & Responsibilities

### Rust Modules

| Module | Responsibility | Key Crates |
|---|---|---|
| `hotkey::manager` | Register/unregister global hotkeys, emit events | `global-hotkey`, `tauri-plugin-global-shortcut` |
| `audio::capture` | Capture microphone PCM via WASAPI/ALSA | `cpal` |
| `inject::windows` | SendInput API for text injection | `windows-sys` |
| `inject::linux` | xdotool/ydotool via subprocess | `std::process` |
| `context::detector` | Get active window title/process | `windows-sys`, `x11rb` |
| `context::app_map` | Map process name → context label | pure Rust |
| `ipc::bridge` | Manage named pipe / Unix socket to Python | `tokio`, `serde_json` |
| `db::manager` | SQLite connection pool, migrations | `sqlx` |
| `settings::store` | Read/write settings from SQLite | `serde`, `sqlx` |
| `tray::menu` | System tray icon and menu | `tauri` |

### Python Services

| Service | Port/Socket | Responsibility | Key Libraries |
|---|---|---|---|
| `whisper` | socket | Audio → raw transcript | `faster-whisper`, `torch`, `silero-vad` |
| `llm` | socket | Raw text → cleaned/formatted/commanded | `ollama`, `httpx` |
| `rag` | socket | Store/retrieve correction vectors | `chromadb`, `sentence-transformers` |

---

## Sequence Diagram: Full Dictation Session

```
React   Rust(Tauri)   Rust(Audio)   Python(Whisper)   Python(LLM)   Python(RAG)   Ollama
  │          │              │                │               │             │            │
  │  keydown │              │                │               │             │            │
  │─────────►│              │                │               │             │            │
  │          │ start_stream │                │               │             │            │
  │          │─────────────►│                │               │             │            │
  │          │              │ PCM chunks     │               │             │            │
  │          │              │───────────────►│               │             │            │
  │          │              │                │ partial text  │             │            │
  │          │              │                │──────────────►│             │            │
  │ partial  │              │                │               │             │            │
  │◄─────────│              │                │               │             │            │
  │          │              │                │               │             │            │
  │  keyup   │              │                │               │             │            │
  │─────────►│              │                │               │             │            │
  │          │ stop_stream  │                │               │             │            │
  │          │─────────────►│                │               │             │            │
  │          │              │ audio_end      │               │             │            │
  │          │              │───────────────►│               │             │            │
  │          │              │                │ final raw     │             │            │
  │          │              │                │──────────────►│             │            │
  │          │              │                │               │ retrieve()  │            │
  │          │              │                │               │────────────►│            │
  │          │              │                │               │   context   │            │
  │          │              │                │               │◄────────────│            │
  │          │              │                │               │ clean()     │            │
  │          │              │                │               │────────────────────────►│
  │          │              │                │               │   cleaned   │            │
  │          │              │                │               │◄────────────────────────│
  │          │              │                │               │ store()     │            │
  │          │              │                │               │────────────►│            │
  │          │ text_ready   │                │               │             │            │
  │          │◄─────────────────────────────────────────────│             │            │
  │ preview  │              │                │               │             │            │
  │◄─────────│              │                │               │             │            │
  │          │ inject()     │                │               │             │            │
  │          │──────────────────────────────────────────────►             │            │
  │          │              │                │               │             │            │
[text inserted into active app]
```

---

## Python Service IPC Design

Each Python service runs as a child process spawned by Rust on startup. They communicate via:

- **Windows**: Named pipes (`\\.\pipe\flowlocal-whisper`, etc.)
- **Linux**: Unix domain sockets (`/tmp/flowlocal-*.sock`)

Services expose a simple JSON-RPC-style framing (no HTTP overhead):

```
[u32 length][JSON payload]
```

All services support a **health check** message and **graceful shutdown** via SIGTERM.

**Process lifecycle**:
```
Tauri app start
    │
    ├── spawn Python whisper service
    ├── spawn Python llm service
    ├── spawn Python rag service
    │
    ├── wait for health checks (max 30s)
    │
    └── ready → emit app:ready to frontend

Tauri app exit
    │
    ├── send shutdown to all services
    └── wait for cleanup (max 5s) → force kill
```

---

## AI Prompt Templates

### Cleanup Prompt (moderate aggressiveness)

```
You are a transcription cleanup assistant. Your ONLY job is to clean up speech-to-text output.

Rules:
1. Remove filler words: uh, um, like, you know, basically, literally, so, actually, right, okay (when used as filler)
2. Fix punctuation and capitalization
3. Fix grammar minimally — preserve the speaker's meaning exactly
4. Do NOT add information that wasn't said
5. Do NOT summarize — output the full cleaned text
6. Do NOT add explanations

Context: {app_context}
Language: {language}
Similar corrections from memory: {rag_context}
Dictionary: {dictionary_terms}

Raw transcription:
{raw_text}

Cleaned text:
```

### Context Prompts

**VS Code / Cursor**:
```
You are cleaning dictation for a code editor context.
- Preserve technical terms exactly
- Do not reformat code snippets
- Keep variable names, function names, URLs as-is
- Apply light cleanup only
```

**Email**:
```
You are cleaning dictation for email composition.
- Use professional punctuation
- Structure as clear paragraphs
- Fix grammar for professional communication
- Keep a natural but polished tone
```

---

## Performance Architecture

### Latency Budget (target: <500ms end-to-end)

```
Audio capture stop signal         0ms
↓
VAD silence detection            ~50ms
↓
PCM → Whisper model               
  - tiny.en:                    ~80ms  (GPU)
  - base.en:                   ~150ms  (GPU)
  - small.en:                  ~300ms  (GPU)
↓
IPC Rust → Python (socket)        ~2ms
↓
Dictionary replacement            ~1ms
↓
RAG retrieval (ChromaDB)         ~30ms
↓
Ollama LLM cleanup (qwen3:4b)   ~200ms  (GPU, cached)
↓
IPC Python → Rust (socket)        ~2ms
↓
Text injection (SendInput)        ~5ms
─────────────────────────────────────
TOTAL (tiny.en + GPU):          ~370ms ✓
TOTAL (small.en + GPU):         ~590ms (acceptable)
TOTAL (CPU only, base.en):      ~800ms (degraded mode)
```

### Optimization Strategies

1. **Model pre-warming**: Load Whisper + Ollama models on startup, keep in memory
2. **Audio streaming**: Pipe PCM chunks in real-time, don't wait for recording end
3. **Parallel pipeline**: Start VAD during recording, don't wait for keyup
4. **RAG async**: Fire ChromaDB retrieval immediately after transcription starts
5. **Connection pooling**: Reuse socket connections between Rust and Python
6. **LLM streaming**: Stream Ollama response, inject text as it arrives

---

## Open Questions

> [!IMPORTANT]
> **Q1 — Hotkey mode**: Should the default hotkey mode be **hold-to-record** (like PTT) or **toggle** (press once to start, press again to stop)? The plan implements both; which should be default?

> [!IMPORTANT]
> **Q2 — Whisper model default**: `tiny.en` is fastest but least accurate; `base` is a good balance; `small` is most accurate in reasonable time. Which should be the default out-of-box model?

> [!IMPORTANT]
> **Q3 — Python service packaging**: Should the Python services be packaged as a standalone executable (via PyInstaller/Nuitka) for distribution, or run as a managed Python venv (simpler for dev, requires Python installed)?

> [!IMPORTANT]
> **Q4 — Overlay position**: Should the live-transcript overlay appear near the cursor position, or always in a fixed corner of the screen?

> [!IMPORTANT]
> **Q5 — Audio input device**: Should the app always use the system default microphone, or provide device selection in settings?

---

## Verification Plan

After each stage:

| Stage | Verification Method |
|---|---|
| 1 — Architecture | Document review (this plan) |
| 2 — Repo structure | `tree` output matches spec; all files exist |
| 3 — Rust backend | `cargo build` compiles; hotkey triggers; audio captures; inject works |
| 4 — Python AI | Health checks pass; whisper transcribes test WAV; LLM cleans sample text |
| 5 — Frontend | UI renders; overlay shows; settings persist |
| 6 — Integration | Full dictation E2E: speak → text injected in Notepad |
| 7 — Tests | All unit + integration tests pass |
| 8 — Packaging | Windows installer produces working `.exe`; Linux `.AppImage` works |

---

## Technology Justification

| Choice | Reason |
|---|---|
| **Tauri 2** | Smallest binary, native OS APIs, proper system tray, global hotkey support, IPC to Rust |
| **faster-whisper** | 4x faster than openai-whisper, CTranslate2 backend, CUDA + CPU support |
| **Silero VAD** | <1MB model, real-time, runs on CPU, best open-source VAD |
| **Ollama** | Simplest local LLM runner, model management built-in, HTTP API |
| **qwen3:4b** | Best quality/speed ratio at 4B params, excellent for text cleanup |
| **ChromaDB** | Easiest local vector DB, Python-native, no server required |
| **nomic-embed-text** | Best open-source embedding model via Ollama, free |
| **SQLite + sqlx** | Zero-config, ACID, async Rust support |
| **cpal** | Cross-platform audio, WASAPI on Windows, ALSA on Linux |

