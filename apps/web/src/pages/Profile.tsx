import { useEffect, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { Upload } from 'lucide-react';
import PdfjsWorker from 'pdfjs-dist/build/pdf.worker.mjs?worker&inline';

import {
  analyzeStoredProfile,
  buildSearchProfile,
  getProfile,
  getRoles,
  getSources,
  getStoredProfileRawText,
  saveProfile,
  type RoleCatalogItem,
  type SearchProfileBuildResult,
  type SearchTargetRegion,
  type SearchWorkMode,
  type SourceCatalogItem,
} from '../api';
import { queryKeys } from '../queryKeys';

const TARGET_REGION_OPTIONS: Array<{ id: SearchTargetRegion; label: string }> = [
  { id: 'ua', label: 'Ukraine' },
  { id: 'eu', label: 'EU' },
  { id: 'eu_remote', label: 'EU remote' },
  { id: 'poland', label: 'Poland' },
  { id: 'germany', label: 'Germany' },
  { id: 'uk', label: 'UK' },
  { id: 'us', label: 'US' },
];

const WORK_MODE_OPTIONS: Array<{ id: SearchWorkMode; label: string }> = [
  { id: 'remote', label: 'Remote' },
  { id: 'hybrid', label: 'Hybrid' },
  { id: 'onsite', label: 'Onsite' },
];

async function extractPdfText(file: File): Promise<string> {
  // Lazy-load pdfjs so it doesn't bloat the initial bundle
  const pdfjsLib = await import('pdfjs-dist');
  if (!pdfjsLib.GlobalWorkerOptions.workerPort) {
    pdfjsLib.GlobalWorkerOptions.workerPort = new PdfjsWorker();
  }

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

function toggleValue<T>(current: T[], value: T): T[] {
  return current.includes(value)
    ? current.filter((existing) => existing !== value)
    : [...current, value];
}

function parseKeywordInput(value: string): string[] {
  const keywords: string[] = [];

  for (const item of value.split(/[\n,]/)) {
    const normalized = item.trim();

    if (normalized && !keywords.includes(normalized)) {
      keywords.push(normalized);
    }
  }

  return keywords;
}

function formatFallbackLabel(value: string): string {
  return value
    .split('_')
    .map((part) => {
      if (part.length <= 2) {
        return part.toUpperCase();
      }

      return part.charAt(0).toUpperCase() + part.slice(1);
    })
    .join(' ');
}

function resolveRoleLabel(roles: RoleCatalogItem[], roleId: string): string {
  return roles.find((role) => role.id === roleId)?.displayName ?? formatFallbackLabel(roleId);
}

function resolveSourceLabel(sources: SourceCatalogItem[], sourceId: string): string {
  return (
    sources.find((source) => source.id === sourceId)?.displayName ??
    formatFallbackLabel(sourceId)
  );
}

function renderPillList(items: string[], emptyLabel: string) {
  if (items.length === 0) {
    return (
      <p className="muted" style={{ margin: 0 }}>
        {emptyLabel}
      </p>
    );
  }

  return (
    <div className="pillWrap">
      {items.map((item) => (
        <span key={item} className="pill">
          {item}
        </span>
      ))}
    </div>
  );
}

function SearchProfileResult({
  result,
  roles,
  sources,
}: {
  result: SearchProfileBuildResult;
  roles: RoleCatalogItem[];
  sources: SourceCatalogItem[];
}) {
  return (
    <div className="grid" style={{ marginTop: 16 }}>
      <section className="card">
        <p className="eyebrow">Analyzed profile</p>
        <p style={{ marginTop: 0 }}>{result.analyzedProfile.summary}</p>

        <div className="detailGrid">
          <div>
            <span className="detailLabel">Primary role</span>
            <strong>{resolveRoleLabel(roles, result.analyzedProfile.primaryRole)}</strong>
          </div>
          <div>
            <span className="detailLabel">Seniority</span>
            <strong>{formatFallbackLabel(result.analyzedProfile.seniority)}</strong>
          </div>
        </div>

        <div className="resultSection">
          <span className="detailLabel">Skills</span>
          {renderPillList(result.analyzedProfile.skills, 'No skills detected yet.')}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Suggested search terms</span>
          {renderPillList(
            result.analyzedProfile.suggestedSearchTerms,
            'No suggested search terms returned.',
          )}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Role candidates</span>
          {result.analyzedProfile.roleCandidates.length > 0 ? (
            <div className="stackList">
              {result.analyzedProfile.roleCandidates.map((candidate) => (
                <div key={candidate.role} className="stackListItem">
                  <strong>{resolveRoleLabel(roles, candidate.role)}</strong>
                  <span className="muted">
                    score {candidate.score} · confidence {candidate.confidence}%
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="muted" style={{ margin: 0 }}>
              No role candidates returned.
            </p>
          )}
        </div>
      </section>

      <section className="card">
        <p className="eyebrow">Search profile</p>

        <div className="detailGrid">
          <div>
            <span className="detailLabel">Primary role</span>
            <strong>{resolveRoleLabel(roles, result.searchProfile.primaryRole)}</strong>
          </div>
          <div>
            <span className="detailLabel">Seniority</span>
            <strong>{formatFallbackLabel(result.searchProfile.seniority)}</strong>
          </div>
        </div>

        <div className="resultSection">
          <span className="detailLabel">Target roles</span>
          {renderPillList(
            result.searchProfile.targetRoles.map((role) => resolveRoleLabel(roles, role)),
            'No target roles returned.',
          )}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Target regions</span>
          {renderPillList(
            result.searchProfile.targetRegions.map((region) => formatFallbackLabel(region)),
            'No target regions selected.',
          )}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Work modes</span>
          {renderPillList(
            result.searchProfile.workModes.map((mode) => formatFallbackLabel(mode)),
            'No work modes selected.',
          )}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Allowed sources</span>
          {renderPillList(
            result.searchProfile.allowedSources.map((source) => resolveSourceLabel(sources, source)),
            'No source restrictions selected.',
          )}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Search terms</span>
          {renderPillList(result.searchProfile.searchTerms, 'No search terms returned.')}
        </div>

        <div className="resultSection">
          <span className="detailLabel">Exclude terms</span>
          {renderPillList(result.searchProfile.excludeTerms, 'No exclude terms selected.')}
        </div>
      </section>
    </div>
  );
}

export default function Profile() {
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [location, setLocation] = useState('');
  const [rawText, setRawText] = useState('');
  const [targetRegions, setTargetRegions] = useState<SearchTargetRegion[]>([]);
  const [workModes, setWorkModes] = useState<SearchWorkMode[]>([]);
  const [preferredRoles, setPreferredRoles] = useState<string[]>([]);
  const [allowedSources, setAllowedSources] = useState<string[]>([]);
  const [includeKeywordsInput, setIncludeKeywordsInput] = useState('');
  const [excludeKeywordsInput, setExcludeKeywordsInput] = useState('');
  const [buildResult, setBuildResult] = useState<SearchProfileBuildResult | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });
  const { data: roles = [], error: rolesError } = useQuery({
    queryKey: queryKeys.roles.all(),
    queryFn: getRoles,
  });
  const { data: sources = [], error: sourcesError } = useQuery({
    queryKey: queryKeys.sources.all(),
    queryFn: getSources,
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

  const buildMutation = useMutation({
    mutationFn: () =>
      buildSearchProfile({
        rawText,
        preferences: {
          targetRegions,
          workModes,
          preferredRoles,
          allowedSources,
          includeKeywords: parseKeywordInput(includeKeywordsInput),
          excludeKeywords: parseKeywordInput(excludeKeywordsInput),
        },
      }),
    onSuccess: (result) => {
      setBuildResult(result);
      toast.success('Search profile built');
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

      <section className="card" style={{ marginTop: 16 }}>
        <div className="pageHeader" style={{ marginBottom: 16, alignItems: 'flex-start' }}>
          <div>
            <p className="eyebrow">Search profile builder</p>
            <h2 style={{ margin: '0 0 8px' }}>Build from current raw text</h2>
            <p className="muted" style={{ margin: 0 }}>
              Uses the CV text above plus explicit preferences. No persistence required.
            </p>
          </div>
          <button
            type="button"
            onClick={() => buildMutation.mutate()}
            disabled={buildMutation.isPending || !rawText.trim()}
          >
            {buildMutation.isPending ? 'Building…' : 'Build search profile'}
          </button>
        </div>

        <div className="form">
          <div className="fieldGroup">
            <span className="fieldLabel">Target regions</span>
            <div className="optionGrid">
              {TARGET_REGION_OPTIONS.map((option) => (
                <label key={option.id} className="optionCard">
                  <input
                    type="checkbox"
                    checked={targetRegions.includes(option.id)}
                    onChange={() =>
                      setTargetRegions((current) => toggleValue(current, option.id))
                    }
                  />
                  <span>{option.label}</span>
                </label>
              ))}
            </div>
          </div>

          <div className="fieldGroup">
            <span className="fieldLabel">Work modes</span>
            <div className="optionGrid">
              {WORK_MODE_OPTIONS.map((option) => (
                <label key={option.id} className="optionCard">
                  <input
                    type="checkbox"
                    checked={workModes.includes(option.id)}
                    onChange={() => setWorkModes((current) => toggleValue(current, option.id))}
                  />
                  <span>{option.label}</span>
                </label>
              ))}
            </div>
          </div>

          <div className="fieldGroup">
            <span className="fieldLabel">Preferred roles</span>
            {roles.length > 0 ? (
              <div className="optionGrid">
                {roles.map((role) => (
                  <label key={role.id} className="optionCard">
                    <input
                      type="checkbox"
                      checked={preferredRoles.includes(role.id)}
                      onChange={() =>
                        setPreferredRoles((current) => toggleValue(current, role.id))
                      }
                    />
                    <span>{role.displayName}</span>
                  </label>
                ))}
              </div>
            ) : (
              <p className="muted" style={{ margin: 0 }}>
                Role catalog unavailable.
              </p>
            )}
            {rolesError && (
              <p className="error" style={{ marginTop: 8 }}>
                {rolesError instanceof Error ? rolesError.message : 'Failed to load roles'}
              </p>
            )}
          </div>

          <div className="fieldGroup">
            <span className="fieldLabel">Allowed sources</span>
            {sources.length > 0 ? (
              <div className="optionGrid">
                {sources.map((source) => (
                  <label key={source.id} className="optionCard">
                    <input
                      type="checkbox"
                      checked={allowedSources.includes(source.id)}
                      onChange={() =>
                        setAllowedSources((current) => toggleValue(current, source.id))
                      }
                    />
                    <span>{source.displayName}</span>
                  </label>
                ))}
              </div>
            ) : (
              <p className="muted" style={{ margin: 0 }}>
                Source catalog unavailable.
              </p>
            )}
            {sourcesError && (
              <p className="error" style={{ marginTop: 8 }}>
                {sourcesError instanceof Error ? sourcesError.message : 'Failed to load sources'}
              </p>
            )}
          </div>

          <div className="fieldGroup">
            <span className="fieldLabel">Include keywords</span>
            <textarea
              rows={3}
              value={includeKeywordsInput}
              onChange={(e) => setIncludeKeywordsInput(e.target.value)}
              placeholder="Comma or newline separated keywords"
            />
          </div>

          <div className="fieldGroup">
            <span className="fieldLabel">Exclude keywords</span>
            <textarea
              rows={3}
              value={excludeKeywordsInput}
              onChange={(e) => setExcludeKeywordsInput(e.target.value)}
              placeholder="Comma or newline separated keywords"
            />
          </div>
        </div>
      </section>

      {buildResult && <SearchProfileResult result={buildResult} roles={roles} sources={sources} />}

      {profile && (
        <section className="card" style={{ marginTop: 16 }}>
          <p className="eyebrow">Latest analysis</p>
          {profile.summary ? (
            <>
              <p style={{ marginTop: 0 }}>{profile.summary}</p>
              {profile.skills.length > 0 && (
                <div className="pillWrap">
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
