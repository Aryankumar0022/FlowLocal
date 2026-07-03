// ============================================================
// pages/Settings.tsx — Full app configuration UI (6 tabs)
// ============================================================

import { useEffect, useState } from 'react';
import { useSettingsStore } from '@/stores/settingsStore';
import type { Settings } from '@/types';

type Tab = 'general' | 'audio' | 'cleanup' | 'ollama' | 'memory' | 'ui';
const TABS: { id: Tab; label: string }[] = [
  { id: 'general', label: 'General' },
  { id: 'audio', label: 'Audio' },
  { id: 'cleanup', label: 'AI Cleanup' },
  { id: 'ollama', label: 'Ollama' },
  { id: 'memory', label: 'Memory' },
  { id: 'ui', label: 'UI' },
];

function ToggleRow({
  label,
  hint,
  checked,
  onChange,
}: {
  label: string;
  hint?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  const id = label.replace(/\s+/g, '-').toLowerCase();
  return (
    <div className="toggle-row">
      <div className="toggle-info">
        <div className="toggle-label">{label}</div>
        {hint && <div className="toggle-hint">{hint}</div>}
      </div>
      <label className="toggle">
        <input type="checkbox" id={id} checked={checked} onChange={(e) => onChange(e.target.checked)} />
        <div className="toggle-track" />
        <div className="toggle-thumb" />
      </label>
    </div>
  );
}

function HotkeyRecorder({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  const [recording, setRecording] = useState(false);

  const startRecord = () => {
    setRecording(true);
    const handleKey = (e: KeyboardEvent) => {
      e.preventDefault();
      const parts: string[] = [];
      if (e.ctrlKey) parts.push('Ctrl');
      if (e.shiftKey) parts.push('Shift');
      if (e.altKey) parts.push('Alt');
      if (e.metaKey) parts.push('Super');
      const key = e.key;
      if (!['Control', 'Shift', 'Alt', 'Meta'].includes(key)) {
        parts.push(key === ' ' ? 'Space' : key);
      }
      if (parts.length > 0) {
        onChange(parts.join('+'));
        setRecording(false);
        window.removeEventListener('keydown', handleKey);
      }
    };
    window.addEventListener('keydown', handleKey, { once: false });
    setTimeout(() => {
      setRecording(false);
      window.removeEventListener('keydown', handleKey);
    }, 5000);
  };

  return (
    <div className="flex-center gap-3">
      <div className="hotkey-display">
        {value.split('+').map((k, i) => (
          <span key={i}>
            {i > 0 && <span style={{ color: 'var(--text-tertiary)', margin: '0 2px' }}>+</span>}
            <span className="hotkey-key">{k}</span>
          </span>
        ))}
      </div>
      <button className="btn btn-secondary btn-sm" onClick={startRecord}>
        {recording ? '…press a key' : 'Change'}
      </button>
    </div>
  );
}

export function Settings() {
  const { settings, isLoading, isSaving, isDirty, load, update, save, reset } =
    useSettingsStore();
  const [tab, setTab] = useState<Tab>('general');
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [audioDevices, setAudioDevices] = useState<string[]>([]);

  useEffect(() => {
    load();
    useSettingsStore.getState().getAudioDevices().then(setAudioDevices);
  }, []);

  const handleSave = async () => {
    await save();
    setSaveSuccess(true);
    setTimeout(() => setSaveSuccess(false), 2000);
  };

  const set = (patch: Partial<Settings>) => update(patch);

  if (isLoading) {
    return (
      <div className="page empty-state">
        <div className="spinner" />
      </div>
    );
  }

  return (
    <div className="page" style={{ paddingBottom: 80 }}>
      <div className="page-header">
        <div className="flex-between">
          <div>
            <h1 className="page-title">Settings</h1>
            <p className="page-subtitle">Configure FlowLocal to match your workflow</p>
          </div>
          <div className="flex gap-2">
            <button className="btn btn-ghost btn-sm" onClick={reset}>Reset defaults</button>
            <button
              className="btn btn-primary"
              onClick={handleSave}
              disabled={isSaving || !isDirty}
            >
              {isSaving ? (
                <div className="spinner" style={{ width: 14, height: 14, borderWidth: 2 }} />
              ) : saveSuccess ? (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                  stroke="currentColor" strokeWidth="2.5">
                  <path d="M20 6L9 17l-5-5" />
                </svg>
              ) : (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                  stroke="currentColor" strokeWidth="2.5">
                  <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                  <polyline points="17 21 17 13 7 13 7 21" />
                  <polyline points="7 3 7 8 15 8" />
                </svg>
              )}
              {saveSuccess ? 'Saved!' : 'Save changes'}
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="tabs">
        {TABS.map((t) => (
          <button
            key={t.id}
            className={`tab ${tab === t.id ? 'active' : ''}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* ── GENERAL ──────────────────────────────────────────── */}
      {tab === 'general' && (
        <div className="settings-section">
          <div className="form-group">
            <label className="form-label">Global hotkey</label>
            <HotkeyRecorder value={settings.hotkey} onChange={(v) => set({ hotkey: v })} />
            <div className="form-hint">
              This key activates dictation globally across all applications
            </div>
          </div>

          <div className="form-group">
            <label className="form-label">Hotkey mode</label>
            <select className="input" value={settings.hotkey_mode}
              onChange={(e) => set({ hotkey_mode: e.target.value as 'hold' | 'toggle' })}>
              <option value="hold">Hold to record (release to stop)</option>
              <option value="toggle">Toggle on/off</option>
            </select>
            <div className="form-hint">
              Hold mode is more natural; toggle is better for long dictations
            </div>
          </div>

          <div className="card" style={{ marginTop: 8 }}>
            <ToggleRow
              label="Auto-detect language"
              hint="Automatically detect the spoken language on each dictation"
              checked={settings.auto_detect_language}
              onChange={(v) => set({ auto_detect_language: v })}
            />
            <ToggleRow
              label="Context-aware formatting"
              hint="Use the active application type to guide how text is formatted"
              checked={settings.context_aware}
              onChange={(v) => set({ context_aware: v })}
            />
            <ToggleRow
              label="Detect active application"
              hint="Read the currently focused app name for context"
              checked={settings.detect_active_app}
              onChange={(v) => set({ detect_active_app: v })}
            />
          </div>

          {!settings.auto_detect_language && (
            <div className="form-group" style={{ marginTop: 16 }}>
              <label className="form-label">Fixed language</label>
              <select className="input" value={settings.language}
                onChange={(e) => set({ language: e.target.value })}>
                {[
                  ['en', 'English'],
                  ['hi', 'Hindi'],
                  ['es', 'Spanish'],
                  ['fr', 'French'],
                  ['de', 'German'],
                  ['zh', 'Chinese'],
                  ['ja', 'Japanese'],
                  ['ko', 'Korean'],
                  ['ar', 'Arabic'],
                  ['pt', 'Portuguese'],
                  ['ru', 'Russian'],
                ].map(([code, name]) => (
                  <option key={code} value={code}>{name} ({code})</option>
                ))}
              </select>
            </div>
          )}
        </div>
      )}

      {/* ── AUDIO ────────────────────────────────────────────── */}
      {tab === 'audio' && (
        <div className="settings-section">
          <div className="form-group">
            <label className="form-label">Microphone</label>
            <select className="input" value={settings.audio_device ?? ''}
              onChange={(e) => set({ audio_device: e.target.value || null })}>
              <option value="">System default</option>
              {audioDevices.map((d) => <option key={d} value={d}>{d}</option>)}
            </select>
          </div>

          <div className="form-group">
            <label className="form-label">Whisper model</label>
            <select className="input" value={settings.whisper_model}
              onChange={(e) => set({ whisper_model: e.target.value as any })}>
              {([
                ['tiny', 'Tiny (39M) — fastest, least accurate'],
                ['tiny.en', 'Tiny English (39M) — English only'],
                ['base', 'Base (74M) — good balance ✓ recommended'],
                ['base.en', 'Base English (74M)'],
                ['small', 'Small (244M) — more accurate'],
                ['small.en', 'Small English (244M)'],
                ['medium', 'Medium (769M) — high accuracy'],
                ['medium.en', 'Medium English (769M)'],
                ['large-v3', 'Large v3 (1.5B) — best accuracy, slow on CPU'],
              ] as [string, string][]).map(([v, l]) => (
                <option key={v} value={v}>{l}</option>
              ))}
            </select>
            <div className="form-hint">Larger models are more accurate but slower to load and transcribe</div>
          </div>

          <div className="card">
            <ToggleRow
              label="Voice activity detection (VAD)"
              hint="Use Silero VAD to filter silence and improve accuracy"
              checked={settings.vad_enabled}
              onChange={(v) => set({ vad_enabled: v })}
            />
            <ToggleRow
              label="Use GPU for Whisper"
              hint="Requires CUDA-capable GPU (NVIDIA RTX recommended)"
              checked={settings.use_gpu}
              onChange={(v) => set({ use_gpu: v })}
            />
          </div>

          {settings.vad_enabled && (
            <div style={{ marginTop: 16 }}>
              <div className="form-group">
                <label className="form-label">
                  VAD threshold: {settings.vad_threshold.toFixed(2)}
                </label>
                <input type="range" className="slider"
                  min={0.1} max={0.9} step={0.05}
                  value={settings.vad_threshold}
                  onChange={(e) => set({ vad_threshold: parseFloat(e.target.value) })}
                />
                <div className="form-hint">Higher = stricter (less false positives). 0.5 is recommended</div>
              </div>

              <div className="form-group">
                <label className="form-label">
                  Silence duration: {settings.silence_duration_ms}ms
                </label>
                <input type="range" className="slider"
                  min={200} max={2000} step={100}
                  value={settings.silence_duration_ms}
                  onChange={(e) => set({ silence_duration_ms: parseInt(e.target.value) })}
                />
                <div className="form-hint">How long of silence before speech is considered ended</div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* ── AI CLEANUP ───────────────────────────────────────── */}
      {tab === 'cleanup' && (
        <div className="settings-section">
          <div className="card" style={{ marginBottom: 16 }}>
            <ToggleRow
              label="Enable AI cleanup"
              hint="Use Ollama LLM to clean up transcription (remove fillers, fix grammar)"
              checked={settings.cleanup_enabled}
              onChange={(v) => set({ cleanup_enabled: v })}
            />
          </div>

          {settings.cleanup_enabled && (
            <>
              <div className="form-group">
                <label className="form-label">Cleanup aggressiveness</label>
                <select className="input" value={settings.cleanup_aggressiveness}
                  onChange={(e) => set({ cleanup_aggressiveness: e.target.value as any })}>
                  <option value="light">Light — fix only obvious errors, minimal changes</option>
                  <option value="moderate">Moderate — standard cleanup (recommended)</option>
                  <option value="aggressive">Aggressive — rewrite for clarity while preserving content</option>
                </select>
              </div>

              <div className="card">
                <ToggleRow
                  label="Remove filler words"
                  hint='Remove "uh", "um", "like", "you know", "basically" etc.'
                  checked={settings.remove_fillers}
                  onChange={(v) => set({ remove_fillers: v })}
                />
                <ToggleRow
                  label="Fix punctuation"
                  hint="Add periods, commas, and question marks where appropriate"
                  checked={settings.fix_punctuation}
                  onChange={(v) => set({ fix_punctuation: v })}
                />
                <ToggleRow
                  label="Fix capitalization"
                  hint="Capitalize sentence starts and proper nouns"
                  checked={settings.fix_capitalization}
                  onChange={(v) => set({ fix_capitalization: v })}
                />
              </div>
            </>
          )}
        </div>
      )}

      {/* ── OLLAMA ───────────────────────────────────────────── */}
      {tab === 'ollama' && (
        <div className="settings-section">
          <div className="form-group">
            <label className="form-label">Ollama host</label>
            <input className="input input-mono" value={settings.ollama_host}
              onChange={(e) => set({ ollama_host: e.target.value })}
              placeholder="http://localhost:11434"
            />
            <div className="form-hint">URL of your local Ollama instance</div>
          </div>

          <div className="form-group">
            <label className="form-label">Cleanup model</label>
            <input className="input input-mono" value={settings.ollama_model}
              onChange={(e) => set({ ollama_model: e.target.value })}
              placeholder="qwen3:4b"
            />
            <div className="form-hint">
              Recommended: qwen3:4b (fast) · qwen3:8b (better) · llama3.2:3b (alternative)
            </div>
          </div>

          <div className="form-group">
            <label className="form-label">Embedding model</label>
            <input className="input input-mono" value={settings.embedding_model}
              onChange={(e) => set({ embedding_model: e.target.value })}
              placeholder="nomic-embed-text"
            />
            <div className="form-hint">
              Used for RAG context retrieval. Run: <code style={{ fontFamily: 'var(--font-mono)',
                fontSize: 11, color: 'var(--text-accent)' }}>ollama pull nomic-embed-text</code>
            </div>
          </div>

          <div className="card" style={{ background: 'var(--bg-elevated)', marginTop: 8 }}>
            <div style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              <strong style={{ color: 'var(--text-primary)', display: 'block', marginBottom: 6 }}>
                Model setup commands
              </strong>
              <div className="text-block text-mono" style={{ userSelect: 'text', marginTop: 8 }}>
                {`ollama pull ${settings.ollama_model}\nollama pull ${settings.embedding_model}`}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ── MEMORY ───────────────────────────────────────────── */}
      {tab === 'memory' && (
        <div className="settings-section">
          <div className="card" style={{ marginBottom: 16 }}>
            <ToggleRow
              label="Enable RAG memory"
              hint="Learn from past corrections to improve future cleanup quality"
              checked={settings.rag_enabled}
              onChange={(v) => set({ rag_enabled: v })}
            />
          </div>

          {settings.rag_enabled && (
            <div className="form-group">
              <label className="form-label">Max context results: {settings.rag_max_results}</label>
              <input type="range" className="slider"
                min={1} max={10} step={1}
                value={settings.rag_max_results}
                onChange={(e) => set({ rag_max_results: parseInt(e.target.value) })}
              />
              <div className="form-hint">
                How many past correction examples to include in the cleanup prompt
              </div>
            </div>
          )}

          <div className="card" style={{ marginTop: 8, background: 'var(--success-subtle)',
            border: '1px solid rgba(16, 185, 129, 0.2)' }}>
            <div style={{ fontSize: 12, color: 'var(--success)', lineHeight: 1.6 }}>
              <strong style={{ display: 'block', marginBottom: 4 }}>How RAG memory works</strong>
              Every time you dictate, your raw speech and the cleaned result are stored as
              embeddings in ChromaDB (locally). Future sessions retrieve similar past
              corrections as context, making cleanup smarter over time.
            </div>
          </div>
        </div>
      )}

      {/* ── UI ───────────────────────────────────────────────── */}
      {tab === 'ui' && (
        <div className="settings-section">
          <div className="card" style={{ marginBottom: 16 }}>
            <ToggleRow
              label="Show recording overlay"
              hint="Display a floating overlay pill during recording and processing"
              checked={settings.show_overlay}
              onChange={(v) => set({ show_overlay: v })}
            />
            <ToggleRow
              label="Auto-start with system"
              hint="Launch FlowLocal when you log into your computer"
              checked={settings.auto_start}
              onChange={(v) => set({ auto_start: v })}
            />
          </div>

          <div className="form-group">
            <label className="form-label">Overlay position</label>
            <select className="input" value={settings.overlay_position}
              onChange={(e) => set({ overlay_position: e.target.value as any })}>
              <option value="bottom">Bottom center</option>
              <option value="top">Top center</option>
            </select>
          </div>
        </div>
      )}
    </div>
  );
}
