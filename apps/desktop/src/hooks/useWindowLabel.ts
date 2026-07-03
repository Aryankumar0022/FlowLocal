// ============================================================
// hooks/useWindowLabel.ts — Detect which Tauri window we are in
// ============================================================

import { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function useWindowLabel(): string | null {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    const w = getCurrentWindow();
    setLabel(w.label);
  }, []);

  return label;
}
