import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Dashboard } from './Dashboard';

// Mock the Zustand stores so we can test UI states cleanly
vi.mock('@/stores/recordingStore', () => ({
  useRecordingStore: () => ({
    phase: 'recording',
    partialText: 'Hello world',
    lastText: '',
    services: { whisper: true, llm: true, rag: true },
    appContext: 'generic'
  })
}));

vi.mock('@/stores/historyStore', () => ({
  useHistoryStore: () => ({
    sessions: [],
    loadSessions: vi.fn(),
  })
}));

vi.mock('@/stores/settingsStore', () => ({
  useSettingsStore: () => ({
    settings: { hotkey: 'F13' }
  })
}));

describe('Dashboard Component', () => {
  it('displays the recording phase and partial text', () => {
    render(<Dashboard />);
    expect(screen.getByText('Recording')).toBeDefined();
    expect(screen.getByText('Hello world')).toBeDefined();
    expect(screen.getByText('All online')).toBeDefined();
  });
});
