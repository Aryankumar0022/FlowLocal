// ============================================================
// stores/recordingStore.ts — Zustand store for live recording state
// ============================================================

import { create } from 'zustand';
import type { RecordingPhase, ServiceHealth } from '@/types';

interface RecordingStore {
  phase: RecordingPhase;
  sessionId: string | null;
  partialText: string;
  lastText: string;
  appContext: string;
  windowTitle: string;
  durationMs: number;
  services: ServiceHealth;
  // Actions
  setRecording: (sessionId: string, appContext: string, windowTitle: string) => void;
  setProcessing: () => void;
  setPartial: (text: string) => void;
  setDone: (cleanText: string, durationMs: number) => void;
  setIdle: () => void;
  setServices: (health: ServiceHealth) => void;
}

export const useRecordingStore = create<RecordingStore>((set) => ({
  phase: 'idle',
  sessionId: null,
  partialText: '',
  lastText: '',
  appContext: 'generic',
  windowTitle: '',
  durationMs: 0,
  services: { whisper: false, llm: false, rag: false },

  setRecording: (sessionId, appContext, windowTitle) =>
    set({ phase: 'recording', sessionId, appContext, windowTitle, partialText: '', durationMs: 0 }),

  setProcessing: () => set({ phase: 'processing' }),

  setPartial: (text) => set({ partialText: text }),

  setDone: (cleanText, durationMs) =>
    set({ phase: 'done', lastText: cleanText, durationMs, partialText: '' }),

  setIdle: () =>
    set({ phase: 'idle', sessionId: null, partialText: '', durationMs: 0 }),

  setServices: (health) => set({ services: health }),
}));
