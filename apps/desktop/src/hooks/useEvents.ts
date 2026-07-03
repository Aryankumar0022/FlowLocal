// ============================================================
// hooks/useEvents.ts — Global Tauri event → Zustand bridge
//
// Call this hook once at the App root level.
// It subscribes to all Rust-emitted events and keeps the stores
// in sync without any component needing to know the event names.
// ============================================================

import { useEffect } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useRecordingStore } from '@/stores/recordingStore';
import { useHistoryStore } from '@/stores/historyStore';
import type {
  RecordingStartEvent,
  TranscriptPartialEvent,
  TextReadyEvent,
  ServicesReadyEvent,
  Session,
} from '@/types';

export function useEvents() {
  const rec = useRecordingStore();
  const hist = useHistoryStore();

  useEffect(() => {
    const unlisten: Array<Promise<UnlistenFn>> = [];

    // ── recording:start ──────────────────────────────────────
    unlisten.push(
      listen<RecordingStartEvent>('recording:start', ({ payload }) => {
        rec.setRecording(payload.session_id, payload.app_context, payload.window_title);
      })
    );

    // ── recording:processing ───────────────────────────────────────
    unlisten.push(
      listen('recording:processing', () => {
        rec.setProcessing();
      })
    );

    // ── transcript:partial ───────────────────────────────────
    unlisten.push(
      listen<TranscriptPartialEvent>('transcript:partial', ({ payload }) => {
        rec.setPartial(payload.text);
      })
    );

    // ── text:ready ───────────────────────────────────────────
    // Fired after LLM cleanup; text has been injected into the active app.
    unlisten.push(
      listen<TextReadyEvent>('text:ready', ({ payload }) => {
        rec.setDone(payload.clean_text, payload.duration_ms);

        // Prepend to history
        const newSession: Session = {
          id: payload.session_id,
          created_at: Date.now(),
          raw_text: payload.raw_text,
          clean_text: payload.clean_text,
          app_context: payload.app_context,
          window_title: '',
          language: payload.language,
          duration_ms: payload.duration_ms,
          word_count: payload.word_count,
          char_count: payload.clean_text.length,
        };
        hist.prependSession(newSession);

        // Brief "done" state, then back to idle
        setTimeout(() => rec.setIdle(), 2500);
      })
    );

    // ── text:error ───────────────────────────────────────────
    unlisten.push(
      listen('text:error', () => {
        rec.setIdle();
      })
    );

    // ── services:ready ───────────────────────────────────────
    unlisten.push(
      listen<ServicesReadyEvent>('services:ready', ({ payload }) => {
        rec.setServices({
          whisper: payload.whisper,
          llm: payload.llm,
          rag: payload.rag,
        });
      })
    );

    // ── Cleanup on unmount ────────────────────────────────────
    return () => {
      unlisten.forEach((p) => p.then((fn) => fn()));
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}
