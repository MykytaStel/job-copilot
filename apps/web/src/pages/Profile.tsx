import { useEffect, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { Upload } from 'lucide-react';

import {
  analyzeStoredProfile,
  getProfile,
  getStoredProfileRawText,
  saveProfile,
} from '../api';
import { queryKeys } from '../queryKeys';

async function extractPdfText(file: File): Promise<string> {
  // Lazy-load pdfjs so it doesn't bloat the initial bundle
  const pdfjsLib = await import('pdfjs-dist');
  pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
    'pdfjs-dist/build/pdf.worker.mjs',
    import.meta.url,
  ).href;

  const buffer = await file.arrayBuffer();
  const pdf = await pdfjsLib.getDocument({ data: buffer }).promise;
  const pages: string[] = [];

  for (let i = 1; i <= pdf.numPages; i++) {
    const page = await pdf.getPage(i);
    const content = await page.getTextContent();
    const pageText = content.items
      .map((item) => ('str' in item ? item.str : ''))
      .join(' ');
    pages.push(pageText);
  }

  return pages.join('\n\n');
}

export default function Profile() {
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [location, setLocation] = useState('');
  const [rawText, setRawText] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

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

  async function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    e.target.value = '';

    if (file.type === 'application/pdf') {
      const loadingToast = toast.loading('Читаємо PDF…');
      try {
        const text = await extractPdfText(file);
        if (text.trim()) {
          setRawText(text);
          toast.success(`PDF завантажено: ${file.name}`, { id: loadingToast });
        } else {
          toast.error('PDF порожній або захищений — спробуйте .txt', { id: loadingToast });
        }
      } catch {
        toast.error('Не вдалося прочитати PDF', { id: loadingToast });
      }
      return;
    }

    const reader = new FileReader();
    reader.onload = (ev) => {
      const text = ev.target?.result;
      if (typeof text === 'string' && text.trim()) {
        setRawText(text);
        toast.success(`Файл завантажено: ${file.name}`);
      } else {
        toast.error('Файл порожній або не вдалося прочитати');
      }
    };
    reader.onerror = () => toast.error('Помилка читання файлу');
    reader.readAsText(file, 'UTF-8');
  }

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
          <span style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            CV / текст профілю
            <button
              type="button"
              className="ghostBtn"
              style={{ fontSize: 13, padding: '4px 10px' }}
              onClick={() => fileInputRef.current?.click()}
            >
              <Upload size={13} style={{ marginRight: 4 }} />
              Завантажити .pdf / .txt / .md
            </button>
          </span>
          <input
            ref={fileInputRef}
            type="file"
            accept=".pdf,.txt,.md,.text"
            style={{ display: 'none' }}
            onChange={handleFileChange}
          />
          <textarea
            value={rawText}
            onChange={(e) => setRawText(e.target.value)}
            rows={12}
            placeholder="Вставте ваше CV, досвід, навички та цільові ролі. Або натисніть «Завантажити» для .txt / .md файлу."
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
