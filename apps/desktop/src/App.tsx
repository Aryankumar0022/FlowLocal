// ============================================================
// App.tsx — Root component with window-label-based routing
// ============================================================

import { useEffect } from 'react';
import { HashRouter } from 'react-router-dom';
import { useWindowLabel } from '@/hooks/useWindowLabel';
import { useEvents } from '@/hooks/useEvents';
import { useSettingsStore } from '@/stores/settingsStore';
import { Layout } from '@/components/Layout/Layout';
import { Overlay } from '@/components/Overlay/Overlay';

// Main app — rendered in the "main" window
function MainApp() {
  useEvents();
  const { load } = useSettingsStore();

  useEffect(() => {
    load();
  }, []);

  return (
    <HashRouter>
      <Layout />
    </HashRouter>
  );
}

// Overlay app — rendered in the transparent "overlay" window
function OverlayApp() {
  useEffect(() => {
    document.body.style.background = 'transparent';
  }, []);
  return <Overlay />;
}

export default function App() {
  const windowLabel = useWindowLabel();

  // null = running in browser (Tauri API unavailable) — show main app
  if (windowLabel === null || windowLabel === 'main') {
    return <MainApp />;
  }

  if (windowLabel === 'overlay') {
    return <OverlayApp />;
  }

  return <MainApp />;
}
