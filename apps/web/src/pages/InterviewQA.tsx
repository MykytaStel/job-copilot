import { useState } from 'react';
import { Pencil, Trash2, X, Check } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { InterviewCategory, InterviewQA } from '@job-copilot/shared';
import {
  createInterviewQA,
  deleteInterviewQA,
  getInterviewQA,
  getJobs,
  updateInterviewQA,
} from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonList } from '../components/Skeleton';

const CATEGORIES: InterviewCategory[] = ['behavioral', 'technical', 'situational', 'company'];

const CATEGORY_LABELS: Record<InterviewCategory, string> = {
  behavioral: 'Behavioral',
  technical: 'Technical',
  situational: 'Situational',
  company: 'Company',
};

export default function InterviewQAPage() {
  const queryClient = useQueryClient();
  const [filterJobId, setFilterJobId] = useState('');
  const [filterCategory, setFilterCategory] = useState<InterviewCategory | ''>('');

  // Form
  const [formJobId, setFormJobId] = useState('');
  const [question, setQuestion] = useState('');
  const [answer, setAnswer] = useState('');
  const [category, setCategory] = useState<InterviewCategory>('behavioral');

  // Edit
  const [editId, setEditId] = useState<string | null>(null);
  const [editAnswer, setEditAnswer] = useState('');

  const { data: jobsList = [], isLoading: loadingJobs } = useQuery({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const { data: qaList = [], isLoading: loadingQA } = useQuery({
    queryKey: queryKeys.interviewQA.all(),
    queryFn: () => getInterviewQA(),
  });

  const loading = loadingJobs || loadingQA;

  const createMutation = useMutation({
    mutationFn: (vars: { jobId: string; question: string; answer: string; category: InterviewCategory }) =>
      createInterviewQA(vars),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.interviewQA.all() });
      setQuestion('');
      setAnswer('');
      toast.success('Question saved');
    },
    onError: (err: unknown) => toast.error(err instanceof Error ? err.message : 'Error'),
  });

  const updateMutation = useMutation({
    mutationFn: (vars: { id: string; answer: string }) => updateInterviewQA(vars.id, { answer: vars.answer }),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.interviewQA.all(), (old: InterviewQA[] = []) =>
        old.map((q) => (q.id === updated.id ? updated : q)),
      );
      setEditId(null);
      toast.success('Answer saved');
    },
    onError: (err: unknown) => toast.error(err instanceof Error ? err.message : 'Error'),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteInterviewQA(id),
    onSuccess: (_, id) => {
      queryClient.setQueryData(queryKeys.interviewQA.all(), (old: InterviewQA[] = []) =>
        old.filter((q) => q.id !== id),
      );
    },
    onError: () => toast.error('Delete failed'),
  });

  function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!formJobId || !question.trim()) return toast.error('Select a job and enter a question');
    createMutation.mutate({ jobId: formJobId, question, answer, category });
  }

  const jobMap = Object.fromEntries(jobsList.map((j) => [j.id, j]));

  const filtered = qaList.filter((q) => {
    if (filterJobId && q.jobId !== filterJobId) return false;
    if (filterCategory && q.category !== filterCategory) return false;
    return true;
  });

  return (
    <div>
      <h1>Interview Q&amp;A</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Build a question bank per job. AI-suggested questions coming soon.
      </p>

      {/* Add question form */}
      <div className="card" style={{ marginBottom: 24 }}>
        <p className="eyebrow" style={{ marginBottom: 12 }}>Add question</p>
        <form className="form" onSubmit={handleCreate}>
          <label>
            Job
            <select value={formJobId} onChange={(e) => setFormJobId(e.target.value)}>
              <option value="">— select a job —</option>
              {jobsList.map((j) => (
                <option key={j.id} value={j.id}>
                  {j.title} @ {j.company}
                </option>
              ))}
            </select>
          </label>

          <label>
            Category
            <select value={category} onChange={(e) => setCategory(e.target.value as InterviewCategory)}>
              {CATEGORIES.map((c) => (
                <option key={c} value={c}>{CATEGORY_LABELS[c]}</option>
              ))}
            </select>
          </label>

          <label>
            Question
            <input
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
              placeholder="e.g. Tell me about a time you handled conflict…"
            />
          </label>

          <label>
            Your answer <span className="muted">(optional, fill later)</span>
            <textarea
              value={answer}
              onChange={(e) => setAnswer(e.target.value)}
              rows={3}
              placeholder="Draft your answer here…"
            />
          </label>

          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <button type="submit" disabled={createMutation.isPending}>
              {createMutation.isPending ? 'Saving…' : 'Add question'}
            </button>
            <span className="muted" style={{ fontSize: 12 }}>
              AI suggestions: coming soon
            </span>
          </div>
        </form>
      </div>

      {/* Filters */}
      <div style={{ display: 'flex', gap: 12, marginBottom: 16, flexWrap: 'wrap' }}>
        <select
          value={filterJobId}
          onChange={(e) => setFilterJobId(e.target.value)}
          style={{ fontSize: 13, padding: '4px 8px' }}
        >
          <option value="">All jobs</option>
          {jobsList.map((j) => (
            <option key={j.id} value={j.id}>{j.title} @ {j.company}</option>
          ))}
        </select>
        <select
          value={filterCategory}
          onChange={(e) => setFilterCategory(e.target.value as InterviewCategory | '')}
          style={{ fontSize: 13, padding: '4px 8px' }}
        >
          <option value="">All categories</option>
          {CATEGORIES.map((c) => (
            <option key={c} value={c}>{CATEGORY_LABELS[c]}</option>
          ))}
        </select>
      </div>

      {/* Q&A list */}
      {loading && <SkeletonList rows={4} />}
      {!loading && filtered.length === 0 && <p className="muted">No questions yet.</p>}

      {filtered.map((qa) => {
        const job = jobMap[qa.jobId];
        return (
          <div key={qa.id} className="card" style={{ marginBottom: 12 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8 }}>
              <div>
                <span className="pill" style={{ fontSize: 11, marginRight: 8 }}>
                  {CATEGORY_LABELS[qa.category]}
                </span>
                {job && (
                  <span className="muted" style={{ fontSize: 12 }}>
                    {job.title} @ {job.company}
                  </span>
                )}
              </div>
              <button
                onClick={() => {
                  if (!confirm('Delete this question?')) return;
                  deleteMutation.mutate(qa.id);
                }}
                style={{ background: 'transparent', color: 'var(--danger, #e53e3e)', border: 'none', cursor: 'pointer', fontSize: 12, display: 'inline-flex', alignItems: 'center', gap: 4 }}
              >
                <Trash2 size={13} /> Delete
              </button>
            </div>

            <p style={{ fontWeight: 600, marginBottom: 8 }}>{qa.question}</p>

            {editId === qa.id ? (
              <div>
                <textarea
                  value={editAnswer}
                  onChange={(e) => setEditAnswer(e.target.value)}
                  rows={5}
                  style={{ width: '100%', boxSizing: 'border-box', fontFamily: 'inherit', fontSize: 13 }}
                />
                <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                  <button onClick={() => updateMutation.mutate({ id: qa.id, answer: editAnswer })} style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}><Check size={13} /> Save</button>
                  <button
                    onClick={() => setEditId(null)}
                    style={{ background: 'transparent', border: '1px solid var(--border)', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                  >
                    <X size={13} /> Cancel
                  </button>
                </div>
              </div>
            ) : (
              <div>
                {qa.answer ? (
                  <p style={{ fontSize: 13, whiteSpace: 'pre-wrap', color: 'var(--text)', marginBottom: 8 }}>
                    {qa.answer}
                  </p>
                ) : (
                  <p className="muted" style={{ fontSize: 13, marginBottom: 8 }}>No answer yet.</p>
                )}
                <button
                  onClick={() => { setEditId(qa.id); setEditAnswer(qa.answer); }}
                  style={{ fontSize: 12, background: 'transparent', border: '1px solid var(--border)', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                >
                  <Pencil size={12} /> Edit answer
                </button>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
