import { useState } from 'react';
import { Pencil, Trash2, X, Check } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { CoverLetter, CoverLetterTone } from '@job-copilot/shared';
import {
  createCoverLetter,
  deleteCoverLetter,
  getCoverLetters,
  getJobs,
  updateCoverLetter,
} from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonList } from '../components/Skeleton';

const TONES: CoverLetterTone[] = ['formal', 'casual', 'enthusiastic'];

const TONE_LABELS: Record<CoverLetterTone, string> = {
  formal: 'Formal',
  casual: 'Casual',
  enthusiastic: 'Enthusiastic',
};

export default function CoverLetterPage() {
  const queryClient = useQueryClient();
  const [selectedJobId, setSelectedJobId] = useState('');
  const [tone, setTone] = useState<CoverLetterTone>('formal');
  const [content, setContent] = useState('');
  const [editId, setEditId] = useState<string | null>(null);
  const [editContent, setEditContent] = useState('');

  const { data: jobsList = [], isLoading: loadingJobs } = useQuery({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const { data: letters = [], isLoading: loadingLetters } = useQuery({
    queryKey: queryKeys.coverLetters.all(),
    queryFn: () => getCoverLetters(),
  });

  const loading = loadingJobs || loadingLetters;

  const createMutation = useMutation({
    mutationFn: (vars: { jobId: string; tone: CoverLetterTone; content?: string }) =>
      createCoverLetter(vars),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.coverLetters.all() });
      setContent('');
      toast.success('Cover letter saved');
    },
    onError: (err: unknown) => toast.error(err instanceof Error ? err.message : 'Error'),
  });

  const updateMutation = useMutation({
    mutationFn: (vars: { id: string; content: string }) => updateCoverLetter(vars.id, vars.content),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.coverLetters.all(), (old: CoverLetter[] = []) =>
        old.map((l) => (l.id === updated.id ? updated : l)),
      );
      setEditId(null);
      toast.success('Saved');
    },
    onError: (err: unknown) => toast.error(err instanceof Error ? err.message : 'Error'),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteCoverLetter(id),
    onSuccess: (_, id) => {
      queryClient.setQueryData(queryKeys.coverLetters.all(), (old: CoverLetter[] = []) =>
        old.filter((l) => l.id !== id),
      );
    },
    onError: (err: unknown) => toast.error(err instanceof Error ? err.message : 'Error'),
  });

  function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedJobId) return toast.error('Select a job');
    createMutation.mutate({ jobId: selectedJobId, tone, content: content.trim() || undefined });
  }

  const jobMap = Object.fromEntries(jobsList.map((j) => [j.id, j]));

  return (
    <div>
      <h1>Cover Letters</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Draft cover letters per job. AI generation will be available once you enable it in settings.
      </p>

      {/* Create form */}
      <div className="card" style={{ marginBottom: 24 }}>
        <p className="eyebrow" style={{ marginBottom: 12 }}>New cover letter</p>
        <form className="form" onSubmit={handleCreate}>
          <label>
            Job
            <select value={selectedJobId} onChange={(e) => setSelectedJobId(e.target.value)}>
              <option value="">— select a job —</option>
              {jobsList.map((j) => (
                <option key={j.id} value={j.id}>
                  {j.title} @ {j.company}
                </option>
              ))}
            </select>
          </label>

          <label>
            Tone
            <select value={tone} onChange={(e) => setTone(e.target.value as CoverLetterTone)}>
              {TONES.map((t) => (
                <option key={t} value={t}>{TONE_LABELS[t]}</option>
              ))}
            </select>
          </label>

          <label>
            Content{' '}
            <span className="muted">(optional — leave blank for AI placeholder)</span>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              rows={6}
              placeholder="Write your letter here, or leave blank and edit later…"
            />
          </label>

          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <button type="submit" disabled={createMutation.isPending}>
              {createMutation.isPending ? 'Saving…' : 'Save letter'}
            </button>
            <span className="muted" style={{ fontSize: 12 }}>
              AI generation: coming soon
            </span>
          </div>
        </form>
      </div>

      {/* Letters list */}
      {loading && <SkeletonList rows={3} />}
      {!loading && letters.length === 0 && (
        <p className="muted">No cover letters yet.</p>
      )}

      {letters.map((letter) => {
        const job = jobMap[letter.jobId];
        return (
          <div key={letter.id} className="card" style={{ marginBottom: 12 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 8 }}>
              <div>
                <span style={{ fontWeight: 600 }}>
                  {job ? `${job.title} @ ${job.company}` : letter.jobId}
                </span>
                <span className="pill" style={{ marginLeft: 8, fontSize: 11 }}>
                  {TONE_LABELS[letter.tone]}
                </span>
              </div>
              <div style={{ display: 'flex', gap: 8 }}>
                {editId === letter.id ? (
                  <>
                    <button onClick={() => updateMutation.mutate({ id: letter.id, content: editContent })} style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}><Check size={13} /> Save</button>
                    <button
                      onClick={() => setEditId(null)}
                      style={{ background: 'transparent', border: '1px solid var(--border)', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                    >
                      <X size={13} /> Cancel
                    </button>
                  </>
                ) : (
                  <>
                    <button
                      onClick={() => { setEditId(letter.id); setEditContent(letter.content); }}
                      style={{ background: 'transparent', border: '1px solid var(--border)', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                    >
                      <Pencil size={13} /> Edit
                    </button>
                    <button
                      onClick={() => {
                        if (!confirm('Delete this cover letter?')) return;
                        deleteMutation.mutate(letter.id);
                      }}
                      style={{ background: 'transparent', color: 'var(--danger, #e53e3e)', border: '1px solid currentColor', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                    >
                      <Trash2 size={13} /> Delete
                    </button>
                  </>
                )}
              </div>
            </div>

            {editId === letter.id ? (
              <textarea
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                rows={10}
                style={{ width: '100%', boxSizing: 'border-box', fontFamily: 'inherit', fontSize: 13 }}
              />
            ) : (
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap', fontSize: 13, lineHeight: 1.6, color: 'var(--text)' }}>
                {letter.content}
              </pre>
            )}

            <p className="muted" style={{ marginTop: 8, fontSize: 11 }}>
              {new Date(letter.createdAt).toLocaleDateString()}
            </p>
          </div>
        );
      })}
    </div>
  );
}
