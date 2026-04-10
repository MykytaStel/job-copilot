import { useState } from 'react';
import { Trash2 } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import type { Offer, OfferInput } from '@job-copilot/shared';
import { createOffer, deleteOffer, getJobs, getOffers } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonList } from '../components/Skeleton';

const CURRENCIES = ['UAH', 'USD', 'EUR', 'PLN'];

function salary(offer: Offer): string {
  if (!offer.salary) return '—';
  return `${offer.salary.toLocaleString()} ${offer.currency}`;
}

export default function OffersPage() {
  const queryClient = useQueryClient();

  // Form
  const [formJobId, setFormJobId] = useState('');
  const [formSalary, setFormSalary] = useState('');
  const [formCurrency, setFormCurrency] = useState('UAH');
  const [formEquity, setFormEquity] = useState('');
  const [formBenefits, setFormBenefits] = useState('');
  const [formDeadline, setFormDeadline] = useState('');
  const [formNotes, setFormNotes] = useState('');

  const { data: jobsList = [], isLoading: loadingJobs } = useQuery({
    queryKey: queryKeys.jobs.all(),
    queryFn: getJobs,
  });

  const { data: offersList = [], isLoading: loadingOffers } = useQuery({
    queryKey: queryKeys.offers.all(),
    queryFn: getOffers,
  });

  const loading = loadingJobs || loadingOffers;

  const createMutation = useMutation({
    mutationFn: (input: OfferInput) => createOffer(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.offers.all() });
      setFormJobId('');
      setFormSalary('');
      setFormEquity('');
      setFormBenefits('');
      setFormDeadline('');
      setFormNotes('');
      toast.success('Offer saved');
    },
    onError: (err: unknown) => toast.error(err instanceof Error ? err.message : 'Error'),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteOffer(id),
    onSuccess: (_, id) => {
      queryClient.setQueryData(queryKeys.offers.all(), (old: Offer[] = []) =>
        old.filter((o) => o.id !== id),
      );
    },
    onError: () => toast.error('Delete failed'),
  });

  function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!formJobId) return toast.error('Select a job');
    const input: OfferInput = {
      jobId: formJobId,
      salary: formSalary ? Number(formSalary) : undefined,
      currency: formCurrency,
      equity: formEquity.trim() || undefined,
      benefits: formBenefits.split(',').map((b) => b.trim()).filter(Boolean),
      deadline: formDeadline || undefined,
      notes: formNotes,
    };
    createMutation.mutate(input);
  }

  const jobMap = Object.fromEntries(jobsList.map((j) => [j.id, j]));
  const offeredJobIds = new Set(offersList.map((o) => o.jobId));
  const availableJobs = jobsList.filter((j) => !offeredJobIds.has(j.id));

  return (
    <div>
      <h1>Offers</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Track received offers and compare them side by side.
      </p>

      {/* Add offer form */}
      <div className="card" style={{ marginBottom: 24 }}>
        <p className="eyebrow" style={{ marginBottom: 12 }}>Add offer</p>
        <form className="form" onSubmit={handleCreate}>
          <label>
            Job
            <select value={formJobId} onChange={(e) => setFormJobId(e.target.value)}>
              <option value="">— select a job —</option>
              {availableJobs.map((j) => (
                <option key={j.id} value={j.id}>
                  {j.title} @ {j.company}
                </option>
              ))}
            </select>
          </label>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr auto', gap: 8 }}>
            <label>
              Salary (monthly gross)
              <input
                type="number"
                value={formSalary}
                onChange={(e) => setFormSalary(e.target.value)}
                placeholder="e.g. 80000"
                min={0}
              />
            </label>
            <label>
              Currency
              <select value={formCurrency} onChange={(e) => setFormCurrency(e.target.value)}>
                {CURRENCIES.map((c) => <option key={c} value={c}>{c}</option>)}
              </select>
            </label>
          </div>

          <label>
            Equity / bonus <span className="muted">(optional)</span>
            <input
              value={formEquity}
              onChange={(e) => setFormEquity(e.target.value)}
              placeholder="e.g. 0.5% options over 4 years"
            />
          </label>

          <label>
            Benefits <span className="muted">(comma-separated)</span>
            <input
              value={formBenefits}
              onChange={(e) => setFormBenefits(e.target.value)}
              placeholder="Remote, Medical, 20 days PTO"
            />
          </label>

          <label>
            Offer deadline
            <input
              type="date"
              value={formDeadline}
              onChange={(e) => setFormDeadline(e.target.value)}
            />
          </label>

          <label>
            Notes
            <textarea
              value={formNotes}
              onChange={(e) => setFormNotes(e.target.value)}
              rows={2}
              placeholder="Anything worth remembering…"
            />
          </label>

          <button type="submit" disabled={createMutation.isPending}>
            {createMutation.isPending ? 'Saving…' : 'Add offer'}
          </button>
        </form>
      </div>

      {/* Offers comparison table */}
      {loading && <SkeletonList rows={3} />}

      {!loading && offersList.length > 0 && (
        <div style={{ overflowX: 'auto', marginBottom: 24 }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ textAlign: 'left', borderBottom: '2px solid var(--border)' }}>
                <th style={{ padding: '8px 12px' }}>Company / Role</th>
                <th style={{ padding: '8px 12px' }}>Salary</th>
                <th style={{ padding: '8px 12px' }}>Equity</th>
                <th style={{ padding: '8px 12px' }}>Benefits</th>
                <th style={{ padding: '8px 12px' }}>Deadline</th>
                <th style={{ padding: '8px 12px' }}>Notes</th>
                <th style={{ padding: '8px 4px' }} />
              </tr>
            </thead>
            <tbody>
              {offersList.map((offer, idx) => {
                const job = jobMap[offer.jobId];
                return (
                  <tr
                    key={offer.id}
                    style={{ borderBottom: '1px solid var(--border)', background: idx % 2 ? 'var(--surface, #f9f9f9)' : undefined }}
                  >
                    <td style={{ padding: '8px 12px', fontWeight: 600 }}>
                      {job ? `${job.title} @ ${job.company}` : offer.jobId}
                    </td>
                    <td style={{ padding: '8px 12px' }}>{salary(offer)}</td>
                    <td style={{ padding: '8px 12px' }}>{offer.equity || '—'}</td>
                    <td style={{ padding: '8px 12px' }}>
                      {offer.benefits.length > 0
                        ? offer.benefits.map((b) => (
                          <span key={b} className="pill" style={{ marginRight: 4, fontSize: 11 }}>{b}</span>
                        ))
                        : '—'}
                    </td>
                    <td style={{ padding: '8px 12px' }}>{offer.deadline ?? '—'}</td>
                    <td style={{ padding: '8px 12px', color: 'var(--muted)', maxWidth: 200 }}>{offer.notes || '—'}</td>
                    <td style={{ padding: '8px 4px' }}>
                      <button
                        onClick={() => {
                          if (!confirm('Remove this offer?')) return;
                          deleteMutation.mutate(offer.id);
                        }}
                        style={{ background: 'transparent', color: 'var(--danger, #e53e3e)', border: 'none', cursor: 'pointer', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                        title="Remove offer"
                      >
                        <Trash2 size={13} />
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {!loading && offersList.length === 0 && (
        <p className="muted">No offers yet. Add the first one above.</p>
      )}
    </div>
  );
}
