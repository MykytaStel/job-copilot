import { useRef, useState } from 'react';
import toast from 'react-hot-toast';
import { downloadBackup, restoreBackup } from '../api';

export default function BackupPage() {
  const [exporting, setExporting] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [restoreFile, setRestoreFile] = useState<File | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  async function handleExport() {
    setExporting(true);
    try {
      const data = await downloadBackup();
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `job-copilot-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success('Backup downloaded');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Export failed');
    } finally {
      setExporting(false);
    }
  }

  async function handleRestore(e: React.FormEvent) {
    e.preventDefault();
    if (!restoreFile) return toast.error('Select a backup file');
    if (!confirm('This will REPLACE all your data. Continue?')) return;

    setRestoring(true);
    try {
      const text = await restoreFile.text();
      const data = JSON.parse(text);
      await restoreBackup(data);
      toast.success(`Restored! (backup from ${data.exportedAt ?? 'unknown date'})`);
      setRestoreFile(null);
      if (fileRef.current) fileRef.current.value = '';
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Restore failed');
    } finally {
      setRestoring(false);
    }
  }

  return (
    <div>
      <h1>Backup &amp; Restore</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Export all your data as JSON. Restore wipes the database and re-imports the backup.
      </p>

      {/* Export */}
      <div className="card" style={{ marginBottom: 24 }}>
        <p className="eyebrow" style={{ marginBottom: 8 }}>Export backup</p>
        <p className="muted" style={{ fontSize: 13, marginBottom: 16 }}>
          Downloads a JSON file with your profile, jobs, resumes, applications, contacts, and all P3/P4 data.
        </p>
        <button onClick={handleExport} disabled={exporting}>
          {exporting ? 'Exporting…' : 'Download backup'}
        </button>
      </div>

      {/* Restore */}
      <div className="card">
        <p className="eyebrow" style={{ marginBottom: 8 }}>Restore from backup</p>
        <p className="muted" style={{ fontSize: 13, marginBottom: 16 }}>
          Upload a previously exported JSON file. This action is destructive — all current data will be replaced.
        </p>
        <form className="form" onSubmit={handleRestore}>
          <label>
            Backup file (.json)
            <input
              ref={fileRef}
              type="file"
              accept=".json,application/json"
              onChange={(e) => setRestoreFile(e.target.files?.[0] ?? null)}
            />
          </label>
          {restoreFile && (
            <p className="muted" style={{ fontSize: 12 }}>Selected: {restoreFile.name}</p>
          )}
          <button
            type="submit"
            disabled={restoring || !restoreFile}
            style={{ background: 'var(--danger, #e53e3e)', borderColor: 'var(--danger, #e53e3e)' }}
          >
            {restoring ? 'Restoring…' : 'Restore (replaces all data)'}
          </button>
        </form>
      </div>
    </div>
  );
}
