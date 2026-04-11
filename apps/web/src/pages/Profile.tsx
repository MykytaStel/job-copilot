import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';

import {
  analyzeStoredProfile,
  getProfile,
  getStoredProfileRawText,
  saveProfile,
} from '../api';
import { queryKeys } from '../queryKeys';

export default function Profile() {
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [location, setLocation] = useState('');
  const [rawText, setRawText] = useState('');

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });

  useEffect(() => {
    if (!profile) return;

    setName(profile.name);
    setEmail(profile.email);
    setLocation(profile.location ?? '');

    void getStoredProfileRawText()
      .then(setRawText)
      .catch(() => {});
  }, [profile]);

  const saveMutation = useMutation({
    mutationFn: (vars: {
      name: string;
      email: string;
      location?: string;
      rawText: string;
    }) =>
      saveProfile({
        name: vars.name,
        email: vars.email,
        location: vars.location,
        rawText: vars.rawText,
        summary: undefined,
        skills: [],
      }),
    onSuccess: (updated) => {
      queryClient.setQueryData(queryKeys.profile.root(), updated);
      toast.success('Profile saved');
    },
    onError: (e: unknown) =>
      toast.error(e instanceof Error ? e.message : 'Error'),
  });

  const analyzeMutation = useMutation({
    mutationFn: analyzeStoredProfile,
    onSuccess: (analysis) => {
      queryClient.setQueryData(queryKeys.profile.root(), (current: Awaited<ReturnType<typeof getProfile>>) =>
        current
          ? {
              ...current,
              summary: analysis.summary,
              skills: analysis.skills,
            }
          : current,
      );
      toast.success('Profile analyzed');
    },
    onError: (e: unknown) =>
      toast.error(e instanceof Error ? e.message : 'Error'),
  });

  return (
    <div>
      <div className="pageHeader">
        <div>
          <h1>Profile</h1>
          <p className="muted">
            Persisted in `engine-api` and used for analysis/search-profile flows.
          </p>
        </div>
        <button
          type="button"
          onClick={() => analyzeMutation.mutate()}
          disabled={analyzeMutation.isPending || !profile}
        >
          {analyzeMutation.isPending ? 'Analyzing…' : 'Analyze'}
        </button>
      </div>

      <form
        className="card form"
        onSubmit={(e) => {
          e.preventDefault();
          saveMutation.mutate({
            name,
            email,
            location: location || undefined,
            rawText,
          });
        }}
      >
        <label>
          Name
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Your name"
            required
          />
        </label>
        <label>
          Email
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="you@email.com"
            required
          />
        </label>
        <label>
          Location <span className="muted">(optional)</span>
          <input
            value={location}
            onChange={(e) => setLocation(e.target.value)}
            placeholder="Kyiv / Remote"
          />
        </label>
        <label>
          Raw CV / profile text
          <textarea
            value={rawText}
            onChange={(e) => setRawText(e.target.value)}
            rows={12}
            placeholder="Paste your CV, experience summary, skills, and target roles here."
            required
          />
        </label>

        <button
          type="submit"
          disabled={saveMutation.isPending || !name || !email || !rawText.trim()}
        >
          {saveMutation.isPending ? 'Saving…' : profile ? 'Update Profile' : 'Create Profile'}
        </button>
      </form>

      {profile && (
        <section className="card" style={{ marginTop: 16 }}>
          <p className="eyebrow">Latest analysis</p>
          {profile.summary ? (
            <>
              <p style={{ marginTop: 0 }}>{profile.summary}</p>
              {profile.skills.length > 0 && (
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
                  {profile.skills.map((skill) => (
                    <span key={skill} className="pill">
                      {skill}
                    </span>
                  ))}
                </div>
              )}
            </>
          ) : (
            <p className="muted" style={{ marginBottom: 0 }}>
              No persisted analysis yet. Save the profile, then run Analyze.
            </p>
          )}
        </section>
      )}
    </div>
  );
}
