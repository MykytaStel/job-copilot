import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { getProfile, getSuggestedSkills, saveProfile } from '../api';
import { queryKeys } from '../queryKeys';

export default function Profile() {
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [location, setLocation] = useState('');
  const [summary, setSummary] = useState('');
  const [skillsRaw, setSkillsRaw] = useState('');

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });

  // Sync form fields when profile loads
  useEffect(() => {
    if (!profile) return;
    setName(profile.name);
    setEmail(profile.email);
    setLocation(profile.location ?? '');
    setSummary(profile.summary ?? '');
    setSkillsRaw(profile.skills.join(', '));
  }, [profile]);

  const saveMutation = useMutation({
    mutationFn: (vars: { name: string; email: string; location?: string; summary?: string; skills: string[] }) =>
      saveProfile(vars),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      toast.success('Profile saved');
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const suggestMutation = useMutation({
    mutationFn: getSuggestedSkills,
    onSuccess: (suggested) => {
      if (suggested.length === 0) {
        toast.error('No skills found in active resume. Upload a resume first.');
        return;
      }
      const existing = skillsRaw.split(',').map((s) => s.trim()).filter(Boolean);
      const merged = [...new Set([...existing, ...suggested])];
      setSkillsRaw(merged.join(', '));
      toast.success(`${suggested.length} skills added`);
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : 'Error'),
  });

  return (
    <div>
      <h1>Profile</h1>
      <form
        className="card form"
        onSubmit={(e) => {
          e.preventDefault();
          const skills = skillsRaw.split(',').map((s) => s.trim()).filter(Boolean);
          saveMutation.mutate({ name, email, location: location || undefined, summary: summary || undefined, skills });
        }}
      >
        <label>
          Name
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Your name" required />
        </label>
        <label>
          Email
          <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="you@email.com" required />
        </label>
        <label>
          Location <span className="muted">(optional)</span>
          <input value={location} onChange={(e) => setLocation(e.target.value)} placeholder="Kyiv, Ukraine / Remote" />
        </label>
        <label>
          Summary <span className="muted">(optional)</span>
          <textarea value={summary} onChange={(e) => setSummary(e.target.value)} rows={3} placeholder="Brief professional summary…" />
        </label>
        <label>
          Skills <span className="muted">(comma-separated)</span>
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              value={skillsRaw}
              onChange={(e) => setSkillsRaw(e.target.value)}
              placeholder="TypeScript, React, Node.js…"
              style={{ flex: 1 }}
            />
            <button
              type="button"
              onClick={() => suggestMutation.mutate()}
              disabled={suggestMutation.isPending}
              style={{ whiteSpace: 'nowrap' }}
              title="Extract skills from your active resume"
            >
              {suggestMutation.isPending ? 'Scanning…' : 'Suggest from CV'}
            </button>
          </div>
        </label>

        {profile && (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {skillsRaw.split(',').map((s) => s.trim()).filter(Boolean).map((s) => (
              <span key={s} className="pill">{s}</span>
            ))}
          </div>
        )}

        <button type="submit" disabled={saveMutation.isPending || !name || !email}>
          {saveMutation.isPending ? 'Saving…' : 'Save Profile'}
        </button>
      </form>
    </div>
  );
}
