// ============================================================
// components/Layout/Layout.tsx — Main window shell
// ============================================================

import { Routes, Route } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { Dashboard } from '@/pages/Dashboard';
import { History } from '@/pages/History';
import { Dictionary } from '@/pages/Dictionary';
import { Settings } from '@/pages/Settings';

export function Layout() {
  return (
    <div className="app-layout">
      <Sidebar />
      <main className="main-content">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/history" element={<History />} />
          <Route path="/dictionary" element={<Dictionary />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
