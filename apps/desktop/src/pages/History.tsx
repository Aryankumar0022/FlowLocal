// ============================================================
// pages/History.tsx — Searchable session history
// ============================================================

import { useEffect, useState, useMemo } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { useHistoryStore } from '@/stores/historyStore';
import type { Session } from '@/types';

function formatTime(ms: number): string {
  return new Date(ms).toLocaleString(undefined, {
    month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  });
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function SessionRow({
  session,
  onDelete,
  onReinject,
}: {
  session: Session;
  onDelete: () => void;
  onReinject: () => void;
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className={`session-item ${expanded ? 'expanded' : ''}`}
      style={{ borderRadius: expanded ? 'var(--r-md)' : 0 }}>
      {/* Header row */}
      <div onClick={() => setExpanded((v) => !v)} style={{ cursor: 'pointer' }}>
        <div className="session-meta">
          <span className="badge badge-accent">{session.app_context || 'generic'}</span>
          <span className="badge badge-neutral">{session.language}</span>
          <span className="text-xs text-tertiary">{session.word_count}w</span>
          <span className="text-xs text-tertiary">{formatDuration(session.duration_ms)}</span>
          <span className="text-xs text-tertiary" style={{ marginLeft: 'auto' }}>
            {formatTime(session.created_at)}
          </span>
        </div>
        <div className="session-preview">{session.clean_text}</div>
      </div>

      {/* Expanded content */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.18 }}
            style={{ overflow: 'hidden' }}
          >
            <div className="session-expanded">
              {/* Raw text */}
              {session.raw_text && session.raw_text !== session.clean_text && (
                <div>
                  <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
                    letterSpacing: '0.5px', color: 'var(--text-tertiary)', marginBottom: 6 }}>
                    Raw transcript
                  </div>
                  <div className="text-block raw">{session.raw_text}</div>
                </div>
              )}

              {/* Clean text */}
              <div>
                <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
                  letterSpacing: '0.5px', color: 'var(--text-tertiary)', marginBottom: 6 }}>
                  Cleaned text
                </div>
                <div className="text-block" style={{ userSelect: 'text' }}>{session.clean_text}</div>
              </div>

              {/* Actions */}
              <div className="flex gap-2" style={{ paddingTop: 4 }}>
                <button className="btn btn-secondary btn-sm" onClick={onReinject}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="1 4 1 10 7 10" />
                    <path d="M3.51 15a9 9 0 1 0 .49-3.51" />
                  </svg>
                  Re-inject
                </button>
                <button className="btn btn-ghost btn-sm" onClick={async () => {
                  await navigator.clipboard.writeText(session.clean_text);
                }}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  Copy
                </button>
                <button className="btn btn-danger btn-sm" style={{ marginLeft: 'auto' }} onClick={onDelete}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                    <path d="M10 11v6M14 11v6" />
                  </svg>
                  Delete
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export function History() {
  const { sessions, isLoadingSessions, loadSessions, deleteSession, clearHistory, reinjectSession } =
    useHistoryStore();
  const [query, setQuery] = useState('');
  const [showClearConfirm, setShowClearConfirm] = useState(false);

  useEffect(() => {
    loadSessions(200, 0);
  }, []);

  const filtered = useMemo(() => {
    if (!query.trim()) return sessions;
    const q = query.toLowerCase();
    return sessions.filter(
      (s) =>
        s.clean_text.toLowerCase().includes(q) ||
        s.raw_text.toLowerCase().includes(q) ||
        s.app_context.toLowerCase().includes(q)
    );
  }, [sessions, query]);

  return (
    <div className="page">
      <div className="page-header">
        <div className="flex-between">
          <div>
            <h1 className="page-title">History</h1>
            <p className="page-subtitle">{sessions.length} dictation sessions stored locally</p>
          </div>

          {sessions.length > 0 && (
            <div className="flex gap-2">
              {showClearConfirm ? (
                <>
                  <span style={{ fontSize: 12, color: 'var(--text-secondary)',
                    alignSelf: 'center' }}>Are you sure?</span>
                  <button className="btn btn-danger btn-sm"
                    onClick={async () => { await clearHistory(); setShowClearConfirm(false); }}>
                    Clear all
                  </button>
                  <button className="btn btn-ghost btn-sm"
                    onClick={() => setShowClearConfirm(false)}>Cancel</button>
                </>
              ) : (
                <button className="btn btn-ghost btn-sm"
                  onClick={() => setShowClearConfirm(true)}>
                  Clear history
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Search */}
      <div className="search-wrapper" style={{ marginBottom: 24 }}>
        <svg className="search-icon" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          className="input search-input"
          placeholder="Search sessions…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </div>

      {/* List */}
      <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
        {isLoadingSessions ? (
          <div className="empty-state">
            <div className="spinner" />
          </div>
        ) : filtered.length === 0 ? (
          <div className="empty-state">
            <svg className="empty-icon" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" strokeWidth="1.5">
              <circle cx="12" cy="12" r="9" /><line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <div style={{ fontSize: 14, fontWeight: 500 }}>
              {query ? 'No results found' : 'No sessions yet'}
            </div>
            <div style={{ fontSize: 12 }}>
              {query ? 'Try a different search term' : 'Start dictating to see your history here'}
            </div>
          </div>
        ) : (
          <AnimatePresence initial={false}>
            {filtered.map((session) => (
              <motion.div
                key={session.id}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.15 }}
              >
                <SessionRow
                  session={session}
                  onDelete={() => deleteSession(session.id)}
                  onReinject={() => reinjectSession(session.id)}
                />
              </motion.div>
            ))}
          </AnimatePresence>
        )}
      </div>
    </div>
  );
}
