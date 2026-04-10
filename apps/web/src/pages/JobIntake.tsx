import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { createJob, fetchJobUrl } from '../api';

export default function JobIntake() {
  const navigate = useNavigate();
  const [title, setTitle] = useState('');
  const [company, setCompany] = useState('');
  const [url, setUrl] = useState('');
  const [rawText, setRawText] = useState('');
  const [fetching, setFetching] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleFetch() {
    if (!url) return;
    setFetching(true);
    setError(null);
    try {
      const result = await fetchJobUrl(url);
      if (result.title) setTitle(result.title);
      if (result.company) setCompany(result.company);
      if (result.description) setRawText(result.description);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error');
    } finally {
      setFetching(false);
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      const job = await createJob({
        source: url ? 'url' : 'manual',
        url: url || undefined,
        title: title || undefined,
        company: company || undefined,
        rawText: rawText || undefined,
      });
      navigate(`/jobs/${job.id}`);
    } catch (err) {
      if (err instanceof Error && err.message.startsWith('Job already saved:')) {
        navigate(`/jobs/${err.message.split(':')[1]}`);
        return;
      }
      setError(err instanceof Error ? err.message : 'Error');
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div>
      <h1>Add Job</h1>
      <form className="card form" onSubmit={handleSubmit}>
        <label>
          Title
          <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="e.g. Frontend Developer" />
        </label>
        <label>
          Company
          <input value={company} onChange={(e) => setCompany(e.target.value)} placeholder="e.g. Acme Corp" />
        </label>
        <label>
          URL <span className="muted">(optional — djinni.co, work.ua, robota.ua)</span>
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://..."
              type="url"
              style={{ flex: 1 }}
            />
            <button
              type="button"
              onClick={handleFetch}
              disabled={fetching || !url}
              style={{ whiteSpace: 'nowrap' }}
            >
              {fetching ? 'Fetching…' : 'Fetch'}
            </button>
          </div>
        </label>
        <label>
          Job description
          <textarea
            value={rawText}
            onChange={(e) => setRawText(e.target.value)}
            rows={10}
            placeholder="Paste the full job description here…"
          />
        </label>
        {error && <p className="error">{error}</p>}
        <button type="submit" disabled={submitting || (!rawText && !url)}>
          {submitting ? 'Saving…' : 'Save Job'}
        </button>
      </form>
    </div>
  );
}
