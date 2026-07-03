// ============================================================
// pages/Dashboard.tsx — Main status & overview page
// ============================================================

import { useEffect, type CSSProperties } from 'react';
import { motion } from 'framer-motion';
import { useRecordingStore } from '@/stores/recordingStore';
import { useHistoryStore } from '@/stores/historyStore';
import { useSettingsStore } from '@/stores/settingsStore';


function VoiceWave({ phase }: { phase: string }) {
  return (
    <div className={`voice-wave ${phase !== 'idle' ? phase : ''}`} aria-hidden="true">
      {Array.from({ length: 28 }, (_, i) => (
        <span key={i} style={{ '--i': i, '--h': `${18 + (i % 7) * 6}px` } as CSSProperties} />
      ))}
    </div>
  );
}

function StatusIcon({ phase }: { phase: string }) {
  if (phase === 'recording') {
    return (
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none"
        stroke="var(--rec)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <line x1="12" y1="19" x2="12" y2="23" />
        <line x1="8" y1="23" x2="16" y2="23" />
      </svg>
    );
  }
  if (phase === 'processing') {
    return <div className="spinner" style={{ width: 28, height: 28, borderWidth: 3 }} />;
  }
  if (phase === 'done') {
    return (
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none"
        stroke="var(--success)" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
        <path d="M20 6L9 17l-5-5" />
      </svg>
    );
  }
  // idle
  return (
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none"
      stroke="var(--text-tertiary)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" y1="19" x2="12" y2="23" />
      <line x1="8" y1="23" x2="16" y2="23" />
    </svg>
  );
}

function formatRelative(ms: number): string {
  const diff = Date.now() - ms;
  const s = Math.floor(diff / 1000);
  if (s < 60) return 'just now';
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

const PHASE_LABELS: Record<string, string> = {
  idle: 'Ready',
  recording: 'Recording',
  processing: 'Processing',
  done: 'Done',
};

export function Dashboard() {
  const { phase, partialText, lastText, services, appContext } = useRecordingStore();
  const { sessions, loadSessions } = useHistoryStore();
  const { settings } = useSettingsStore();

  useEffect(() => {
    loadSessions(5, 0);
  }, []);

  // Stats
  const todayMs = new Date().setHours(0, 0, 0, 0);
  const todaySessions = sessions.filter((s) => s.created_at > todayMs);
  const todayWords = todaySessions.reduce((a, s) => a + s.word_count, 0);
  const todayChars = todaySessions.reduce((a, s) => a + s.char_count, 0);

  const cardClass = ['recording-status-card', phase !== 'idle' ? phase : '']
    .filter(Boolean)
    .join(' ');

  return (
    <div className="page">
      <div className="page-header">
        <h1 className="page-title">Dashboard</h1>
        <p className="page-subtitle">Local-first voice dictation — all inference on your machine</p>
      </div>

      {/* Status card */}
      <motion.div
        className={cardClass}
        layout
        transition={{ type: 'spring', stiffness: 300, damping: 28 }}
      >
        <VoiceWave phase={phase} />

        <div className={`status-icon-ring ${phase !== 'idle' ? phase : ''}`}>
          <StatusIcon phase={phase} />
        </div>

        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: 20, fontWeight: 700, letterSpacing: '-0.5px',
            color: phase === 'recording' ? 'var(--rec)'
              : phase === 'processing' ? 'var(--text-accent)'
              : phase === 'done' ? 'var(--success)'
              : 'var(--text-secondary)' }}>
            {PHASE_LABELS[phase]}
          </div>

          {phase === 'recording' && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              style={{ fontSize: 13, color: 'var(--text-secondary)', marginTop: 4 }}
            >
              {partialText
                ? <span style={{ color: 'var(--text-primary)' }}>{partialText}</span>
                : 'Listening for speech…'}
            </motion.div>
          )}

          {phase === 'done' && lastText && (
            <motion.div
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              style={{ fontSize: 13, color: 'var(--text-primary)', marginTop: 4,
                maxWidth: 400, textAlign: 'center' }}
            >
              "{lastText.length > 120 ? lastText.slice(0, 120) + '…' : lastText}"
            </motion.div>
          )}

          {phase === 'idle' && (
            <div style={{ fontSize: 13, color: 'var(--text-tertiary)', marginTop: 4 }}>
              Press <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-accent)',
                background: 'var(--border-faint)', padding: '1px 6px', borderRadius: 4,
                fontSize: 12 }}>{settings.hotkey}</span> to start dictating
            </div>
          )}
        </div>

        {appContext && appContext !== 'generic' && phase === 'recording' && (
          <div className="overlay-context-badge"
            style={{ position: 'absolute', top: 16, right: 16 }}>
            {appContext}
          </div>
        )}
      </motion.div>

      {/* Stats row */}
      <div className="grid-3 mt-6">
        <div className="stat-card">
          <div className="stat-value">{todaySessions.length}</div>
          <div className="stat-label">Sessions today</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">{todayWords.toLocaleString()}</div>
          <div className="stat-label">Words dictated</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">{todayChars.toLocaleString()}</div>
          <div className="stat-label">Characters typed</div>
        </div>
      </div>

      {/* Services */}
      <div className="mt-6">
        <div className="card">
          <div className="card-header">
            <span className="card-title">AI Services</span>
            <span className={`badge ${services.whisper && services.llm && services.rag
              ? 'badge-success' : 'badge-warning'}`}>
              {services.whisper && services.llm && services.rag ? 'All online' : 'Starting…'}
            </span>
          </div>

          {[
            { name: 'Whisper', desc: 'Speech transcription', ok: services.whisper, port: 7771 },
            { name: 'Ollama LLM', desc: 'Text cleanup & commands', ok: services.llm, port: 7772 },
            { name: 'ChromaDB RAG', desc: 'Context memory', ok: services.rag, port: 7773 },
          ].map(({ name, desc, ok, port }) => (
            <div key={name} className="service-row">
              <div>
                <div style={{ fontSize: 13, fontWeight: 500, color: 'var(--text-primary)' }}>{name}</div>
                <div style={{ fontSize: 11, color: 'var(--text-tertiary)', marginTop: 1 }}>
                  {desc} · :{port}
                </div>
              </div>
              <span className={`badge ${ok ? 'badge-success' : 'badge-neutral'}`}>
                {ok ? 'Online' : 'Offline'}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Recent sessions */}
      {sessions.length > 0 && (
        <div className="mt-6">
          <div className="card">
            <div className="card-header">
              <span className="card-title">Recent dictations</span>
            </div>
            {sessions.slice(0, 5).map((s) => (
              <div key={s.id} className="session-item" style={{ padding: '12px 0',
                borderBottom: '1px solid var(--border-faint)' }}>
                <div className="session-meta">
                  <span className="badge badge-accent">{s.app_context}</span>
                  <span className="text-xs text-tertiary">{s.word_count}w</span>
                  <span className="text-xs text-tertiary">{formatRelative(s.created_at)}</span>
                </div>
                <div className="session-preview">{s.clean_text}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
