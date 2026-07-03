// ============================================================
// stores/historyStore.ts — Zustand store for sessions & dictionary
// ============================================================

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Session, DictionaryEntry } from '@/types';

interface HistoryStore {
  sessions: Session[];
  dictionary: DictionaryEntry[];
  isLoadingSessions: boolean;
  isLoadingDict: boolean;
  // Session actions
  loadSessions: (limit?: number, offset?: number) => Promise<void>;
  deleteSession: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  reinjectSession: (id: string) => Promise<void>;
  prependSession: (session: Session) => void;
  // Dictionary actions
  loadDictionary: () => Promise<void>;
  addEntry: (wrong: string, correct: string) => Promise<void>;
  deleteEntry: (id: number) => Promise<void>;
  importDictionary: (entries: Array<[string, string]>) => Promise<void>;
}

export const useHistoryStore = create<HistoryStore>((set, get) => ({
  sessions: [],
  dictionary: [],
  isLoadingSessions: false,
  isLoadingDict: false,

  loadSessions: async (limit = 100, offset = 0) => {
    set({ isLoadingSessions: true });
    try {
      const response = await invoke<{ items: Session[], total: number, limit: number, offset: number }>('get_history', { limit, offset });
      set({ sessions: response.items, isLoadingSessions: false });
    } catch {
      set({ isLoadingSessions: false });
    }
  },

  deleteSession: async (id) => {
    await invoke('delete_session', { id });
    set((s) => ({ sessions: s.sessions.filter((sess) => sess.id !== id) }));
  },

  clearHistory: async () => {
    await invoke('clear_history');
    set({ sessions: [] });
  },

  reinjectSession: async (id) => {
    await invoke('reinject_session', { id });
  },

  prependSession: (session) =>
    set((s) => ({ sessions: [session, ...s.sessions] })),

  loadDictionary: async () => {
    set({ isLoadingDict: true });
    try {
      const dictionary = await invoke<DictionaryEntry[]>('get_dictionary');
      set({ dictionary, isLoadingDict: false });
    } catch {
      set({ isLoadingDict: false });
    }
  },

  addEntry: async (wrong, correct) => {
    await invoke('add_dictionary_entry', { wrong, correct });
    await get().loadDictionary();
  },

  deleteEntry: async (id) => {
    await invoke('delete_dictionary_entry', { id });
    set((s) => ({ dictionary: s.dictionary.filter((e) => e.id !== id) }));
  },

  importDictionary: async (entries) => {
    await invoke('import_dictionary', { entries });
    await get().loadDictionary();
  },
}));
