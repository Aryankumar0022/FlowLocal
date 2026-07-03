// ============================================================
// types/index.ts — All application-wide TypeScript types
// ============================================================

// ── Settings ─────────────────────────────────────────────────

export type WhisperModel =
  | 'tiny'
  | 'tiny.en'
  | 'base'
  | 'base.en'
  | 'small'
  | 'small.en'
  | 'medium'
  | 'medium.en'
  | 'large-v3';

export type HotkeyMode = 'hold' | 'toggle';
export type CleanupAggressiveness = 'light' | 'moderate' | 'aggressive';

export interface Settings {
  // Input
  hotkey: string;
  hotkey_mode: HotkeyMode;
  audio_device: string | null;

  // Transcription
  whisper_model: WhisperModel;
  language: string;
  auto_detect_language: boolean;
  vad_enabled: boolean;
  vad_threshold: number;
  silence_duration_ms: number;

  // Cleanup
  cleanup_enabled: boolean;
  cleanup_aggressiveness: CleanupAggressiveness;
  remove_fillers: boolean;
  fix_punctuation: boolean;
  fix_capitalization: boolean;

  // Ollama
  ollama_host: string;
  ollama_model: string;
  embedding_model: string;

  // RAG / Memory
  rag_enabled: boolean;
  rag_max_results: number;

  // Context
  context_aware: boolean;
  detect_active_app: boolean;

  // Hardware
  use_gpu: boolean;

  // UI
  show_overlay: boolean;
  overlay_position: 'bottom' | 'top' | 'cursor';
  auto_start: boolean;
}

// ── Sessions / History ────────────────────────────────────────

export interface Session {
  id: string;
  created_at: number; // Unix ms
  raw_text: string;
  clean_text: string;
  app_context: string;
  window_title: string;
  language: string;
  duration_ms: number;
  word_count: number;
  char_count: number;
}

// ── Dictionary ────────────────────────────────────────────────

export interface DictionaryEntry {
  id: number;
  wrong: string;
  correct: string;
  created_at: number;
  use_count: number;
}

// ── Recording state ───────────────────────────────────────────

export type RecordingPhase = 'idle' | 'recording' | 'processing' | 'done';

export interface RecordingState {
  phase: RecordingPhase;
  sessionId: string | null;
  partialText: string;
  lastText: string;
  appContext: string;
  windowTitle: string;
  durationMs: number;
}

// ── Service health ────────────────────────────────────────────

export interface ServiceHealth {
  whisper: boolean;
  llm: boolean;
  rag: boolean;
}

// ── Events (from Rust → Frontend) ────────────────────────────

export interface RecordingStartEvent {
  session_id: string;
  app_context: string;
  window_title: string;
}

export interface RecordingStopEvent {
  session_id: string;
}

export interface TranscriptPartialEvent {
  session_id: string;
  text: string;
}

export interface TranscriptFinalEvent {
  session_id: string;
  text: string;
  language: string;
  duration_ms: number;
}

export interface TextReadyEvent {
  session_id: string;
  raw_text: string;
  clean_text: string;
  app_context: string;
  language: string;
  duration_ms: number;
  word_count: number;
}

export interface ServicesReadyEvent {
  whisper: boolean;
  llm: boolean;
  rag: boolean;
  error?: string;
}

export interface AppErrorEvent {
  code: string;
  message: string;
}
