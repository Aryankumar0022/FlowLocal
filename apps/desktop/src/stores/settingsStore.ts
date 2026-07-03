// ============================================================
// stores/settingsStore.ts — Zustand store for app settings
// ============================================================

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Settings } from '@/types';

const DEFAULT_SETTINGS: Settings = {
  hotkey: 'ctrl+space',
  hotkey_mode: 'toggle',
  audio_device: null,
  whisper_model: 'base',
  language: 'en',
  auto_detect_language: true,
  vad_enabled: true,
  vad_threshold: 0.5,
  silence_duration_ms: 500,
  cleanup_enabled: true,
  cleanup_aggressiveness: 'moderate',
  remove_fillers: true,
  fix_punctuation: true,
  fix_capitalization: true,
  ollama_host: 'http://localhost:11434',
  ollama_model: 'qwen3:4b',
  embedding_model: 'nomic-embed-text',
  rag_enabled: true,
  rag_max_results: 5,
  context_aware: true,
  detect_active_app: true,
  use_gpu: true,
  show_overlay: true,
  overlay_position: 'bottom',
  auto_start: false,
};

interface SettingsStore {
  settings: Settings;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  isDirty: boolean;
  // Actions
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => void;
  save: () => Promise<void>;
  reset: () => Promise<void>;
  getAudioDevices: () => Promise<string[]>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  isLoading: false,
  isSaving: false,
  error: null,
  isDirty: false,

  load: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await invoke<Settings>('get_settings');
      set({ settings, isLoading: false, isDirty: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  update: (patch) => {
    set((s) => ({
      settings: { ...s.settings, ...patch },
      isDirty: true,
    }));
  },

  save: async () => {
    const { settings } = get();
    set({ isSaving: true, error: null });
    try {
      await invoke('save_settings', { settings });
      set({ isSaving: false, isDirty: false });
    } catch (e) {
      set({ error: String(e), isSaving: false });
      throw e;
    }
  },

  reset: async () => {
    set({ isLoading: true });
    try {
      await invoke('reset_settings');
      const settings = await invoke<Settings>('get_settings');
      set({ settings, isLoading: false, isDirty: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  getAudioDevices: async () => {
    try {
      return await invoke<string[]>('get_audio_devices');
    } catch {
      return [];
    }
  },
}));
