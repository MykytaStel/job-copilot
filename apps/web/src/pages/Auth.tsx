import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { login, register } from '../api/auth';
import { useToast } from '../context/ToastContext';
import { writeToken } from '../lib/authSession';
import { writeProfileId } from '../lib/profileSession';
import { Button } from '../components/ui/Button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '../components/ui/Card';

type Mode = 'register' | 'login';

export default function Auth() {
  const navigate = useNavigate();
  const { showToast } = useToast();
  const [mode, setMode] = useState<Mode>('register');
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [rawText, setRawText] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setLoading(true);
    try {
      const res =
        mode === 'register'
          ? await register({ name, email, password, raw_text: rawText })
          : await login({ email, password });
      writeToken(res.token);
      writeProfileId(res.profile_id);
      navigate('/');
    } catch (err) {
      showToast({ type: 'error', message: err instanceof Error ? err.message : 'Something went wrong' });
    } finally {
      setLoading(false);
    }
  }

  const inputClass =
    'w-full rounded-[var(--radius-md)] border border-border bg-surface-muted px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary/60 focus:outline-none focus:ring-1 focus:ring-primary/30 disabled:opacity-50';

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <div className="w-full max-w-md">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold tracking-tight text-foreground">Job Copilot</h1>
          <p className="mt-1 text-sm text-muted-foreground">Your personal job tracking dashboard</p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>{mode === 'register' ? 'Create your profile' : 'Sign in'}</CardTitle>
            <CardDescription>
              {mode === 'register'
                ? 'Paste your CV and we will set everything up.'
                : 'Enter your email and password to access your profile.'}
            </CardDescription>
          </CardHeader>

          <CardContent>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              {mode === 'register' && (
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-muted-foreground">Full name</label>
                  <input
                    className={inputClass}
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Alex Kovalenko"
                    required
                    disabled={loading}
                  />
                </div>
              )}

              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-medium text-muted-foreground">Email</label>
                <input
                  className={inputClass}
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="alex@example.com"
                  required
                  disabled={loading}
                />
              </div>

              <div className="flex flex-col gap-1.5">
                <label className="text-xs font-medium text-muted-foreground">Password</label>
                <input
                  className={inputClass}
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  minLength={mode === 'register' ? 8 : undefined}
                  required
                  disabled={loading}
                />
              </div>

              {mode === 'register' && (
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-muted-foreground">
                    CV / profile summary
                  </label>
                  <textarea
                    className={`${inputClass} min-h-[140px] resize-y`}
                    value={rawText}
                    onChange={(e) => setRawText(e.target.value)}
                    placeholder="Paste your CV text or write a short description of your background and skills…"
                    required
                    disabled={loading}
                  />
                </div>
              )}

              <Button type="submit" className="mt-2 w-full" disabled={loading}>
                {loading
                  ? 'Loading…'
                  : mode === 'register'
                    ? 'Start tracking jobs'
                    : 'Sign in'}
              </Button>
            </form>

            <div className="mt-4 text-center">
              {mode === 'register' ? (
                <p className="text-xs text-muted-foreground">
                  Already have an account?{' '}
                  <button
                    className="text-primary hover:underline"
                    onClick={() => setMode('login')}
                    type="button"
                  >
                    Sign in
                  </button>
                </p>
              ) : (
                <p className="text-xs text-muted-foreground">
                  New here?{' '}
                  <button
                    className="text-primary hover:underline"
                    onClick={() => setMode('register')}
                    type="button"
                  >
                    Create profile
                  </button>
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
