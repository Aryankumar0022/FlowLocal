# FlowLocal — Contributing Guide

Thank you for contributing to FlowLocal!

## Development Setup

See [README.md](README.md) for prerequisites and install steps.

## Architecture Overview

- **`apps/desktop/src-tauri/`** — Rust backend (audio capture, text injection, IPC, DB, hotkeys)
- **`apps/desktop/src/`** — React frontend (settings UI, overlay, history)
- **`services/whisper/`** — faster-whisper transcription Python microservice
- **`services/llm/`** — Ollama LLM cleanup Python microservice
- **`services/rag/`** — ChromaDB RAG Python microservice
- **`services/shared/`** — Shared Python utilities (IPC protocol, logging)

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Turkish language support
fix: prevent duplicate text injection on rapid keypress
perf: pre-warm Whisper model on startup
docs: update architecture diagram
test: add VAD silence detection unit tests
chore: bump faster-whisper to 1.1.0
```

## Branching

- `main` — stable, release-tagged
- `develop` — active development
- `feature/<name>` — feature branches
- `fix/<name>` — bug fix branches

## Pull Requests

1. Fork and create a branch from `develop`
2. Write tests for new functionality
3. Ensure `cargo clippy`, `cargo fmt`, `npm run lint`, and `ruff check` all pass
4. Submit PR against `develop`

## Code Style

**Rust**: `cargo fmt` + `clippy` (no warnings)

**TypeScript**: ESLint strict + Prettier

**Python**: Ruff (line length 100), type annotations required

## Testing

```bash
# Rust
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml

# Frontend
cd apps/desktop/src && npm test

# Python (per service)
cd services/whisper && python -m pytest
cd services/llm    && python -m pytest
cd services/rag    && python -m pytest
```

## License

By contributing, you agree your contributions will be licensed under the MIT License.
