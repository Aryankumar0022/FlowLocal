// ============================================================
// components/Overlay/Overlay.tsx
//
// Floating overlay pill shown during recording & processing.
// Rendered in the dedicated "overlay" Tauri window which is
// transparent and always-on-top.
// ============================================================

import { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useRecordingStore } from '@/stores/recordingStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useEvents } from '@/hooks/useEvents';

function WaveformBars() {
  return (
    <div className="waveform">
      {Array.from({ length: 7 }, (_, i) => (
        <div key={i} className="waveform-bar" />
      ))}
    </div>
  );
}

function Spinner() {
  return <div className="spinner" />;
}

export function Overlay() {
  useEvents();

  const { phase, partialText, lastText, appContext } = useRecordingStore();
  const { settings } = useSettingsStore();

  const isRecordingOrProcessing = phase !== 'idle';
  const visible = isRecordingOrProcessing && settings.show_overlay;

  useEffect(() => {
    const w = getCurrentWindow();
    if (visible) {
      w.show();
    } else {
      // Small delay to allow the exit animation to finish before hiding the OS window
      const timer = setTimeout(() => {
        w.hide();
      }, 350);
      return () => clearTimeout(timer);
    }
  }, [visible]);

  const pillClass = [
    'overlay-pill',
    phase === 'recording' ? 'recording' : '',
    phase === 'processing' ? 'processing' : '',
  ]
    .filter(Boolean)
    .join(' ');

  const displayText =
    phase === 'recording'
      ? partialText || 'Listening…'
      : phase === 'processing'
      ? partialText || 'Processing…'
      : phase === 'done'
      ? lastText
      : '';

  const statusLabel =
    phase === 'recording'
      ? 'Recording'
      : phase === 'processing'
      ? 'Transcribing'
      : 'Done';

  return (
    <div className="overlay-root" data-tauri-drag-region>
      <AnimatePresence>
        {visible && (
          <motion.div
            className={pillClass}
            initial={{ opacity: 0, y: 20, scale: 0.92 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 16, scale: 0.94 }}
            transition={{ type: 'spring', stiffness: 380, damping: 30 }}
          >
            {/* Left indicator */}
            <div style={{ flexShrink: 0 }}>
              {phase === 'recording' && <WaveformBars />}
              {(phase === 'processing' || phase === 'done') && (
                phase === 'processing' ? <Spinner /> : (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none"
                    stroke="var(--success)" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M20 6L9 17l-5-5" />
                  </svg>
                )
              )}
            </div>

            {/* Text */}
            <div className="overlay-text">
              <div className="overlay-status">{statusLabel}</div>
              <div className="overlay-transcript">{displayText}</div>
            </div>

            {/* Context badge */}
            {appContext && appContext !== 'generic' && (
              <div className="overlay-context-badge">{appContext}</div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
