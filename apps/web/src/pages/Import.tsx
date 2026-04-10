import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import toast from 'react-hot-toast';
import type { ImportResult } from '@job-copilot/shared';
import { importBatch } from '../api';

const STATUS_COLORS: Record<ImportResult['status'], string> = {
  imported: 'var(--accent, #38a169)',
  duplicate: 'var(--muted, #888)',
  error: 'var(--danger, #e53e3e)',
};

const STATUS_LABELS: Record<ImportResult['status'], string> = {
  imported: 'Imported',
  duplicate: 'Duplicate',
  error: 'Error',
};

export default function ImportPage() {
  const navigate = useNavigate();
  const [urlsText, setUrlsText] = useState('');
  const [importing, setImporting] = useState(false);
  const [results, setResults] = useState<ImportResult[] | null>(null);

  async function handleImport(e: React.FormEvent) {
    e.preventDefault();
    const urls = urlsText
      .split('\n')
      .map((u) => u.trim())
      .filter(Boolean);

    if (urls.length === 0) return toast.error('Paste at least one URL');
    if (urls.length > 20) return toast.error('Max 20 URLs per batch');

    setImporting(true);
    setResults(null);
    try {
      const { results: res } = await importBatch(urls);
      setResults(res);
      const imported = res.filter((r) => r.status === 'imported').length;
      const dupes = res.filter((r) => r.status === 'duplicate').length;
      const errors = res.filter((r) => r.status === 'error').length;
      toast.success(`${imported} imported, ${dupes} duplicate, ${errors} error`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Import failed');
    } finally {
      setImporting(false);
    }
  }

  const importedCount = results?.filter((r) => r.status === 'imported').length ?? 0;

  return (
    <div>
      <h1>Batch Import Jobs</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Paste job URLs — one per line (Djinni, Work.ua, Robota.ua, or any site). Max 20 per batch.
      </p>

      <div className="card" style={{ marginBottom: 24 }}>
        <form className="form" onSubmit={handleImport}>
          <label>
            Job URLs (one per line)
            <textarea
              value={urlsText}
              onChange={(e) => setUrlsText(e.target.value)}
              rows={8}
              placeholder={
                'https://djinni.co/jobs/123-senior-react-developer/\nhttps://work.ua/jobs/456/\nhttps://robota.ua/ua/vacancy/789'
              }
              style={{ fontFamily: 'monospace', fontSize: 12 }}
            />
          </label>

          <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
            <button type="submit" disabled={importing}>
              {importing ? 'Importing…' : 'Import jobs'}
            </button>
            {importing && (
              <span className="muted" style={{ fontSize: 12 }}>
                Fetching pages, please wait…
              </span>
            )}
          </div>
        </form>
      </div>

      {/* Results */}
      {results && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
            <p className="eyebrow">Results — {results.length} URLs processed</p>
            {importedCount > 0 && (
              <button onClick={() => navigate('/')}>Go to Dashboard</button>
            )}
          </div>

          {results.map((r, i) => (
            <div
              key={i}
              className="card"
              style={{ marginBottom: 8, borderLeft: `4px solid ${STATUS_COLORS[r.status]}` }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div style={{ flex: 1, minWidth: 0 }}>
                  {r.job ? (
                    <span style={{ fontWeight: 600 }}>
                      {r.job.title} @ {r.job.company}
                    </span>
                  ) : (
                    <span className="muted" style={{ fontSize: 12, wordBreak: 'break-all' }}>
                      {r.url}
                    </span>
                  )}
                  {r.error && (
                    <p className="muted" style={{ fontSize: 12, marginTop: 4 }}>{r.error}</p>
                  )}
                </div>
                <span
                  style={{ color: STATUS_COLORS[r.status], fontSize: 12, fontWeight: 600, marginLeft: 12, whiteSpace: 'nowrap' }}
                >
                  {STATUS_LABELS[r.status]}
                </span>
              </div>
              {r.job && (
                <p className="muted" style={{ fontSize: 11, marginTop: 4, wordBreak: 'break-all' }}>
                  {r.url}
                </p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
