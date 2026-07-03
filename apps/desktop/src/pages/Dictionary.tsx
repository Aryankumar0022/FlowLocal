// ============================================================
// pages/Dictionary.tsx — Personal word correction dictionary
// ============================================================

import { useEffect, useState, useRef } from 'react';
import { useHistoryStore } from '@/stores/historyStore';

export function Dictionary() {
  const { dictionary, isLoadingDict, loadDictionary, addEntry, deleteEntry, importDictionary } =
    useHistoryStore();

  const [wrong, setWrong] = useState('');
  const [correct, setCorrect] = useState('');
  const [adding, setAdding] = useState(false);
  const [error, setError] = useState('');
  const [query, setQuery] = useState('');
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadDictionary();
  }, []);

  const filtered = dictionary.filter(
    (e) =>
      e.wrong.toLowerCase().includes(query.toLowerCase()) ||
      e.correct.toLowerCase().includes(query.toLowerCase())
  );

  const handleAdd = async () => {
    const w = wrong.trim();
    const c = correct.trim();
    if (!w || !c) { setError('Both fields are required'); return; }
    if (w === c) { setError('Wrong and correct cannot be the same'); return; }
    setAdding(true);
    setError('');
    try {
      await addEntry(w, c);
      setWrong('');
      setCorrect('');
    } catch (e) {
      setError(String(e));
    } finally {
      setAdding(false);
    }
  };

  const handleExport = () => {
    const obj: Record<string, string> = {};
    dictionary.forEach((e) => { obj[e.wrong] = e.correct; });
    const blob = new Blob([JSON.stringify(obj, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'flowlocal-dictionary.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async (ev) => {
      try {
        const obj = JSON.parse(ev.target?.result as string);
        const entries: Array<[string, string]> = Object.entries(obj);
        await importDictionary(entries);
      } catch {
        setError('Invalid JSON file');
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  return (
    <div className="page">
      <div className="page-header">
        <div className="flex-between">
          <div>
            <h1 className="page-title">Dictionary</h1>
            <p className="page-subtitle">
              Teach FlowLocal your vocabulary — corrections are applied during transcription
            </p>
          </div>
          <div className="flex gap-2">
            <button className="btn btn-ghost btn-sm" onClick={handleExport}
              disabled={dictionary.length === 0}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              Export
            </button>
            <button className="btn btn-ghost btn-sm" onClick={() => fileRef.current?.click()}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              Import
              <input ref={fileRef} type="file" accept=".json" style={{ display: 'none' }}
                onChange={handleImport} />
            </button>
          </div>
        </div>
      </div>

      {/* Add entry form */}
      <div className="add-entry-form" style={{ marginBottom: 24 }}>
        <div className="form-group" style={{ marginBottom: 0 }}>
          <label className="form-label">Heard as (wrong)</label>
          <input className="input input-mono"
            placeholder='e.g. "tauri"'
            value={wrong}
            onChange={(e) => setWrong(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
          />
        </div>
        <div className="form-group" style={{ marginBottom: 0 }}>
          <label className="form-label">Replace with (correct)</label>
          <input className="input input-mono"
            placeholder='e.g. "Tauri"'
            value={correct}
            onChange={(e) => setCorrect(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
          />
        </div>
        <button className="btn btn-primary" onClick={handleAdd} disabled={adding}
          style={{ alignSelf: 'flex-end' }}>
          {adding ? <div className="spinner" style={{ width: 14, height: 14, borderWidth: 2 }} /> : (
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" strokeWidth="2.5">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          )}
          Add entry
        </button>
        {error && (
          <div style={{ gridColumn: '1 / -1', fontSize: 12, color: 'var(--error)' }}>{error}</div>
        )}
      </div>

      {/* Search */}
      {dictionary.length > 5 && (
        <div className="search-wrapper" style={{ marginBottom: 16 }}>
          <svg className="search-icon" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input className="input search-input" placeholder="Filter entries…"
            value={query} onChange={(e) => setQuery(e.target.value)} />
        </div>
      )}

      {/* Table */}
      <div className="card" style={{ padding: 0 }}>
        {isLoadingDict ? (
          <div className="empty-state"><div className="spinner" /></div>
        ) : filtered.length === 0 ? (
          <div className="empty-state">
            <svg className="empty-icon" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" strokeWidth="1.5">
              <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
              <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
            </svg>
            <div style={{ fontSize: 14, fontWeight: 500 }}>
              {query ? 'No matching entries' : 'Dictionary is empty'}
            </div>
            <div style={{ fontSize: 12 }}>
              {query ? 'Try a different search' : 'Add corrections above to teach FlowLocal your vocabulary'}
            </div>
          </div>
        ) : (
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Heard as</th>
                  <th>Corrected to</th>
                  <th>Uses</th>
                  <th style={{ width: 48 }} />
                </tr>
              </thead>
              <tbody>
                {filtered.map((entry) => (
                  <tr key={entry.id}>
                    <td>
                      <span className="text-mono" style={{ color: 'var(--error)',
                        fontSize: 12 }}>{entry.wrong}</span>
                    </td>
                    <td>
                      <span className="text-mono" style={{ color: 'var(--success)',
                        fontSize: 12 }}>{entry.correct}</span>
                    </td>
                    <td>
                      <span className="text-secondary text-xs">{entry.use_count}</span>
                    </td>
                    <td>
                      <button className="btn-icon"
                        onClick={() => deleteEntry(entry.id)}
                        title="Delete entry">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                          stroke="currentColor" strokeWidth="2" strokeLinecap="round"
                          strokeLinejoin="round">
                          <polyline points="3 6 5 6 21 6" />
                          <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
