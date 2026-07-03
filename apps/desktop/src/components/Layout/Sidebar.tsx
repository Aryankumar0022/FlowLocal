// ============================================================
// components/Layout/Sidebar.tsx
// ============================================================

import { useLocation, useNavigate } from 'react-router-dom';
import { useRecordingStore } from '@/stores/recordingStore';

interface NavItem {
  path: string;
  label: string;
  icon: React.ReactNode;
}

function Icon({ children }: { children: React.ReactNode }) {
  return (
    <svg
      className="nav-icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {children}
    </svg>
  );
}

const NAV_ITEMS: NavItem[] = [
  {
    path: '/',
    label: 'Dashboard',
    icon: (
      <Icon>
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
      </Icon>
    ),
  },
  {
    path: '/history',
    label: 'History',
    icon: (
      <Icon>
        <path d="M12 8v4l3 3" />
        <circle cx="12" cy="12" r="9" />
      </Icon>
    ),
  },
  {
    path: '/dictionary',
    label: 'Dictionary',
    icon: (
      <Icon>
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
      </Icon>
    ),
  },
  {
    path: '/settings',
    label: 'Settings',
    icon: (
      <Icon>
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
      </Icon>
    ),
  },
];

export function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { services, phase } = useRecordingStore();

  return (
    <aside className="sidebar">
      {/* Logo */}
      <div className="sidebar-logo">
        <div className="sidebar-logo-icon">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none"
            stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" y1="19" x2="12" y2="23" />
            <line x1="8" y1="23" x2="16" y2="23" />
          </svg>
        </div>
        <span className="sidebar-logo-text">FlowLocal</span>
      </div>

      {/* Navigation */}
      <nav className="sidebar-nav">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.path}
            className={`nav-item ${location.pathname === item.path ? 'active' : ''}`}
            onClick={() => navigate(item.path)}
          >
            {item.icon}
            {item.label}
          </button>
        ))}
      </nav>

      {/* Footer — service health */}
      <div className="sidebar-footer">
        <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
          letterSpacing: '0.5px', color: 'var(--text-tertiary)', marginBottom: 8 }}>
          AI Services
        </div>
        {[
          { label: 'Whisper', ok: services.whisper },
          { label: 'LLM', ok: services.llm },
          { label: 'RAG', ok: services.rag },
        ].map(({ label, ok }) => (
          <div key={label} className="flex-between" style={{ padding: '4px 0' }}>
            <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>{label}</span>
            <div className={`status-dot ${ok ? 'online' : 'offline'}`} />
          </div>
        ))}

        {/* Recording indicator in footer */}
        {phase !== 'idle' && (
          <div style={{
            marginTop: 12,
            padding: '6px 10px',
            background: phase === 'recording' ? 'var(--rec-subtle)' : 'var(--accent-subtle)',
            border: `1px solid ${phase === 'recording' ? 'rgba(225,29,72,0.25)' : 'rgba(124,58,237,0.25)'}`,
            borderRadius: 'var(--r-md)',
            fontSize: 11,
            fontWeight: 600,
            textTransform: 'uppercase' as const,
            letterSpacing: '0.5px',
            color: phase === 'recording' ? 'var(--rec)' : 'var(--text-accent)',
            display: 'flex',
            alignItems: 'center',
            gap: 6,
          }}>
            <div style={{
              width: 6, height: 6, borderRadius: '50%',
              background: phase === 'recording' ? 'var(--rec)' : 'var(--accent)',
              animation: phase === 'recording' ? 'rec-pulse 1.2s ease-in-out infinite' : 'none',
            }} />
            {phase === 'recording' ? 'Recording' : phase === 'processing' ? 'Processing' : 'Done'}
          </div>
        )}
      </div>
    </aside>
  );
}
