import { useState } from 'react';
import { Bell, BellOff, Trash2 } from 'lucide-react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { JobAlert } from '@job-copilot/shared';
import { createAlert, deleteAlert, getAlerts, toggleAlert } from '../api';
import { queryKeys } from '../queryKeys';
import { SkeletonList } from '../components/Skeleton';

export default function Alerts() {
  const queryClient = useQueryClient();
  const [keywordsRaw, setKeywordsRaw] = useState('');
  const [chatId, setChatId] = useState('');
  const [formError, setFormError] = useState<string | null>(null);

  const { data: items = [], isLoading, error } = useQuery({
    queryKey: queryKeys.alerts.all(),
    queryFn: getAlerts,
  });

  const createMutation = useMutation({
    mutationFn: (vars: { keywords: string[]; telegramChatId: string }) => createAlert(vars),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.alerts.all() });
      setKeywordsRaw('');
      setChatId('');
      setFormError(null);
    },
    onError: (e: unknown) => setFormError(e instanceof Error ? e.message : 'Error'),
  });

  const toggleMutation = useMutation({
    mutationFn: (alert: JobAlert) => toggleAlert(alert.id, !alert.active),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.alerts.all(), (old: JobAlert[] = []) =>
        old.map((a) => (a.id === updated.id ? updated : a)),
      );
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAlert(id),
    onSuccess: (_, id) => {
      queryClient.setQueryData(queryKeys.alerts.all(), (old: JobAlert[] = []) =>
        old.filter((a) => a.id !== id),
      );
    },
  });

  function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const keywords = keywordsRaw.split(',').map((k) => k.trim()).filter(Boolean);
    if (keywords.length === 0 || !chatId.trim()) {
      setFormError('Enter at least one keyword and your Chat ID');
      return;
    }
    createMutation.mutate({ keywords, telegramChatId: chatId.trim() });
  }

  return (
    <div>
      <h1>Telegram Alerts</h1>
      <p className="muted" style={{ marginBottom: 24 }}>
        Get notified when a new job matches your keywords.
      </p>

      {/* Setup guide */}
      <div className="card" style={{ marginBottom: 24, fontSize: 13 }}>
        <p className="eyebrow" style={{ marginBottom: 8 }}>Setup (once)</p>
        <ol style={{ paddingLeft: 18, lineHeight: 1.8 }}>
          <li>Open Telegram → search <b>@BotFather</b> → send <code>/newbot</code> → copy your <b>BOT_TOKEN</b></li>
          <li>Add <code>TELEGRAM_BOT_TOKEN=your_token</code> to <code>apps/api-legacy/.env</code> and restart the API</li>
          <li>Send any message to your bot → open <code>api.telegram.org/bot{'<TOKEN>'}/getUpdates</code> → find your <b>chat id</b></li>
          <li>Paste the Chat ID below and create an alert</li>
        </ol>
      </div>

      {/* Create form */}
      <div className="card" style={{ marginBottom: 24 }}>
        <p className="eyebrow" style={{ marginBottom: 12 }}>New alert</p>
        <form className="form" onSubmit={handleCreate}>
          <label>
            Keywords <span className="muted">(comma-separated, e.g. React Native, senior, remote)</span>
            <input
              value={keywordsRaw}
              onChange={(e) => setKeywordsRaw(e.target.value)}
              placeholder="React Native, TypeScript, Kyiv"
            />
          </label>
          <label>
            Telegram Chat ID
            <input
              value={chatId}
              onChange={(e) => setChatId(e.target.value)}
              placeholder="123456789"
            />
          </label>
          {formError && <p className="error">{formError}</p>}
          <button type="submit" disabled={createMutation.isPending}>
            {createMutation.isPending ? 'Saving…' : 'Create alert'}
          </button>
        </form>
      </div>

      {/* Alert list */}
      {isLoading && <SkeletonList rows={3} />}
      {error && <p className="error">{error instanceof Error ? error.message : 'Error'}</p>}
      {!isLoading && items.length === 0 && (
        <p className="muted">No alerts yet.</p>
      )}
      {items.map((alert) => (
        <div
          key={alert.id}
          className="card"
          style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8, opacity: alert.active ? 1 : 0.5 }}
        >
          <div style={{ flex: 1 }}>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 4 }}>
              {alert.keywords.map((kw) => (
                <span key={kw} className="pill">{kw}</span>
              ))}
            </div>
            <span className="muted" style={{ fontSize: 12 }}>Chat ID: {alert.telegramChatId}</span>
          </div>
          <button
            onClick={() => toggleMutation.mutate(alert)}
            style={{ whiteSpace: 'nowrap', background: alert.active ? 'var(--accent, #38a169)' : undefined, display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            {alert.active ? <><Bell size={14} /> Active</> : <><BellOff size={14} /> Paused</>}
          </button>
          <button
            onClick={() => {
              if (!confirm('Delete this alert?')) return;
              deleteMutation.mutate(alert.id);
            }}
            style={{ background: 'transparent', color: 'var(--danger, #e53e3e)', border: '1px solid currentColor', display: 'inline-flex', alignItems: 'center', gap: 4 }}
          >
            <Trash2 size={14} /> Delete
          </button>
        </div>
      ))}
    </div>
  );
}
