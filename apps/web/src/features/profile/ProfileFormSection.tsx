import type {
  ExperienceEntry,
  LanguageLevel,
  LanguageProficiency,
  WorkModePreference,
} from '@job-copilot/shared/profiles';
import type { ChangeEventHandler, RefObject } from 'react';
import { useState } from 'react';
import { BriefcaseBusiness, Code2, ContactRound, Link, Pencil, Upload, X } from 'lucide-react';

import { Button } from '../../components/ui/Button';
import { SurfaceHero, SurfaceSection } from '../../components/ui/Surface';
import { cn } from '../../lib/cn';
import {
  PROFILE_LANGUAGE_LEVEL_OPTIONS,
  PROFILE_LOCATION_QUICK_ADD_OPTIONS,
  PROFILE_SALARY_CURRENCY_OPTIONS,
  PROFILE_WORK_MODE_OPTIONS,
} from './profile.constants';

export function ProfileFormSection({
  name,
  email,
  location,
  rawText,
  yearsOfExperience,
  salaryMin,
  salaryMax,
  salaryCurrency,
  languages,
  preferredLocations,
  experience,
  workModePreference,
  profileExists,
  fileInputRef,
  isSaving,
  isAnalyzing,
  onSave,
  onAnalyze,
  onOpenFilePicker,
  onFileChange,
  setName,
  setEmail,
  setLocation,
  setRawText,
  setYearsOfExperience,
  setSalaryMin,
  setSalaryMax,
  setSalaryCurrency,
  setWorkModePreference,
  onAddPreferredLocation,
  onRemovePreferredLocation,
  onAddExperience,
  onUpdateExperience,
  onRemoveExperience,
  onAddLanguage,
  onRemoveLanguage,
  onUpdateLanguageLevel,
  portfolioUrl,
  githubUrl,
  linkedinUrl,
  setPortfolioUrl,
  setGithubUrl,
  setLinkedinUrl,
}: {
  name: string;
  email: string;
  location: string;
  rawText: string;
  yearsOfExperience: string;
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: LanguageProficiency[];
  preferredLocations: string[];
  experience: ExperienceEntry[];
  workModePreference: WorkModePreference;
  profileExists: boolean;
  fileInputRef: RefObject<HTMLInputElement | null>;
  isSaving: boolean;
  isAnalyzing: boolean;
  onSave: () => void;
  onAnalyze: () => void;
  onOpenFilePicker: () => void;
  onFileChange: ChangeEventHandler<HTMLInputElement>;
  setName: (value: string) => void;
  setEmail: (value: string) => void;
  setLocation: (value: string) => void;
  setRawText: (value: string) => void;
  setYearsOfExperience: (value: string) => void;
  setSalaryMin: (value: string) => void;
  setSalaryMax: (value: string) => void;
  setSalaryCurrency: (value: string) => void;
  setWorkModePreference: (value: WorkModePreference) => void;
  onAddPreferredLocation: (location: string) => void;
  onRemovePreferredLocation: (location: string) => void;
  onAddExperience: (entry: ExperienceEntry) => void;
  onUpdateExperience: (index: number, entry: ExperienceEntry) => void;
  onRemoveExperience: (index: number) => void;
  onAddLanguage: (language: string, level: LanguageLevel) => void;
  onRemoveLanguage: (language: string) => void;
  onUpdateLanguageLevel: (language: string, level: LanguageLevel) => void;
  portfolioUrl: string;
  githubUrl: string;
  linkedinUrl: string;
  setPortfolioUrl: (value: string) => void;
  setGithubUrl: (value: string) => void;
  setLinkedinUrl: (value: string) => void;
}) {
  const [languageInput, setLanguageInput] = useState('');
  const [languageLevel, setLanguageLevel] = useState<LanguageLevel>('B2');
  const [preferredLocationInput, setPreferredLocationInput] = useState('');

  function addLanguage() {
    const trimmed = languageInput.trim();
    if (!trimmed) return;
    onAddLanguage(trimmed, languageLevel);
    setLanguageInput('');
  }

  function addPreferredLocation(value = preferredLocationInput) {
    const trimmed = value.trim();
    if (!trimmed) return;
    onAddPreferredLocation(trimmed);
    setPreferredLocationInput('');
  }

  function hasPreferredLocation(value: string) {
    return preferredLocations.some((location) => location.toLowerCase() === value.toLowerCase());
  }

  return (
    <>
      <SurfaceHero className="flex flex-col gap-5 p-7 md:flex-row md:items-end md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Persisted profile
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white-a04 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              {profileExists ? 'Ready for analysis' : 'Create profile first'}
            </span>
          </div>
          <h2 className="m-0 text-2xl font-bold text-card-foreground">Candidate intake</h2>
          <p className="m-0 mt-2 text-sm leading-6 text-muted-foreground">
            Persisted in `engine-api` and used for analysis/search-profile flows.
          </p>
        </div>
        <Button
          type="button"
          onClick={onAnalyze}
          disabled={!profileExists || isAnalyzing}
          className="w-full md:w-auto"
        >
          {isAnalyzing ? 'Analyzing…' : 'Analyze'}
        </Button>
      </SurfaceHero>

      <SurfaceSection
        as="form"
        className="flex flex-col gap-5"
        onSubmit={(event) => {
          event.preventDefault();
          onSave();
        }}
      >
        <label id="profile-field-name">
          Name
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Your name"
            required
          />
        </label>
        <label id="profile-field-email">
          Email
          <input
            type="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            placeholder="you@email.com"
            required
          />
        </label>
        <label>
          Location <span className="text-muted-foreground">(optional)</span>
          <input
            value={location}
            onChange={(event) => setLocation(event.target.value)}
            placeholder="Kyiv / Remote"
          />
        </label>
        <label>
          Years of experience <span className="text-muted-foreground">(optional)</span>
          <input
            type="number"
            min={0}
            max={80}
            value={yearsOfExperience}
            onChange={(event) => setYearsOfExperience(event.target.value)}
            placeholder="5"
          />
        </label>
        <div className="grid gap-4 md:grid-cols-3">
          <label className="space-y-2">
            <span className="flex items-center gap-2 text-sm font-medium text-foreground">
              <Link className="h-4 w-4 text-muted-foreground" />
              Portfolio URL
            </span>
            <input
              type="url"
              value={portfolioUrl}
              onChange={(event) => setPortfolioUrl(event.target.value)}
              placeholder="https://your-site.com"
              className="w-full rounded-xl border border-border bg-card px-3 py-2 text-sm"
            />
          </label>

          <label className="space-y-2">
            <span className="flex items-center gap-2 text-sm font-medium text-foreground">
              <Code2 className="h-4 w-4 text-muted-foreground" />
              GitHub URL
            </span>
            <input
              type="url"
              value={githubUrl}
              onChange={(event) => setGithubUrl(event.target.value)}
              placeholder="https://github.com/username"
              className="w-full rounded-xl border border-border bg-card px-3 py-2 text-sm"
            />
          </label>

          <label className="space-y-2">
            <span className="flex items-center gap-2 text-sm font-medium text-foreground">
              <ContactRound className="h-4 w-4 text-muted-foreground" />
              LinkedIn URL
            </span>
            <input
              type="url"
              value={linkedinUrl}
              onChange={(event) => setLinkedinUrl(event.target.value)}
              placeholder="https://linkedin.com/in/username"
              className="w-full rounded-xl border border-border bg-card px-3 py-2 text-sm"
            />
          </label>
          {(portfolioUrl || githubUrl || linkedinUrl) && (
            <div className="flex flex-wrap gap-2 text-sm">
              {portfolioUrl && (
                <a
                  href={portfolioUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="rounded-full border border-border px-3 py-1 text-muted-foreground hover:text-foreground"
                >
                  Portfolio
                </a>
              )}
              {githubUrl && (
                <a
                  href={githubUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="rounded-full border border-border px-3 py-1 text-muted-foreground hover:text-foreground"
                >
                  GitHub
                </a>
              )}
              {linkedinUrl && (
                <a
                  href={linkedinUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="rounded-full border border-border px-3 py-1 text-muted-foreground hover:text-foreground"
                >
                  LinkedIn
                </a>
              )}
            </div>
          )}
        </div>
        <div id="profile-field-salary" className="fieldGroup">
          <span className="fieldLabel">Expected salary</span>
          <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_140px]">
            <label>
              Min <span className="text-muted-foreground">(optional)</span>
              <input
                type="number"
                min={0}
                value={salaryMin}
                onChange={(event) => setSalaryMin(event.target.value)}
                placeholder="2500"
              />
            </label>
            <label>
              Max <span className="text-muted-foreground">(optional)</span>
              <input
                type="number"
                min={0}
                value={salaryMax}
                onChange={(event) => setSalaryMax(event.target.value)}
                placeholder="4000"
              />
            </label>
            <label>
              Currency
              <select
                value={salaryCurrency}
                onChange={(event) => setSalaryCurrency(event.target.value)}
              >
                {PROFILE_SALARY_CURRENCY_OPTIONS.map((option) => (
                  <option key={option.id} value={option.id}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
        </div>
        <div id="profile-field-languages" className="fieldGroup">
          <span className="fieldLabel">Languages</span>
          <div className="flex flex-col gap-3">
            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_140px_auto]">
              <label className="m-0">
                Language
                <input
                  value={languageInput}
                  onChange={(event) => setLanguageInput(event.target.value)}
                  placeholder="English"
                />
              </label>
              <label className="m-0">
                Level
                <select
                  value={languageLevel}
                  onChange={(event) => setLanguageLevel(event.target.value as LanguageLevel)}
                >
                  {PROFILE_LANGUAGE_LEVEL_OPTIONS.map((option) => (
                    <option key={option.id} value={option.id}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
              <Button
                type="button"
                variant="outline"
                className="self-end"
                onClick={addLanguage}
                disabled={!languageInput.trim()}
              >
                Add
              </Button>
            </div>

            {languages.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {languages.map((entry) => (
                  <span
                    key={entry.language}
                    className="inline-flex min-h-9 items-center gap-2 rounded-full border border-border bg-surface-muted px-3 py-1 text-xs font-medium text-card-foreground"
                  >
                    <span>{entry.language}</span>
                    <select
                      value={entry.level}
                      onChange={(event) =>
                        onUpdateLanguageLevel(entry.language, event.target.value as LanguageLevel)
                      }
                      className="h-7 rounded-full border-border bg-card px-2 py-0 text-xs"
                      aria-label={`${entry.language} level`}
                    >
                      {PROFILE_LANGUAGE_LEVEL_OPTIONS.map((option) => (
                        <option key={option.id} value={option.id}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                    <Button
                      type="button"
                      variant="icon"
                      size="icon"
                      className="h-6 w-6 rounded-full"
                      onClick={() => onRemoveLanguage(entry.language)}
                      aria-label={`Remove ${entry.language}`}
                      title={`Remove ${entry.language}`}
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
        <div id="profile-field-preferred-locations" className="fieldGroup">
          <span className="fieldLabel">Preferred locations</span>
          <div className="flex flex-col gap-3">
            <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto]">
              <label className="m-0">
                City / country
                <input
                  value={preferredLocationInput}
                  onChange={(event) => setPreferredLocationInput(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault();
                      addPreferredLocation();
                    }
                  }}
                  placeholder="Kyiv, Remote, Lviv"
                />
              </label>
              <Button
                type="button"
                variant="outline"
                className="self-end"
                onClick={() => addPreferredLocation()}
                disabled={!preferredLocationInput.trim()}
              >
                Add
              </Button>
            </div>

            <div className="flex flex-wrap gap-2">
              {PROFILE_LOCATION_QUICK_ADD_OPTIONS.map((option) => (
                <Button
                  key={option}
                  type="button"
                  variant="outline"
                  size="sm"
                  active={hasPreferredLocation(option)}
                  onClick={() => addPreferredLocation(option)}
                  disabled={hasPreferredLocation(option)}
                >
                  {option}
                </Button>
              ))}
            </div>

            {preferredLocations.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {preferredLocations.map((location) => (
                  <span
                    key={location}
                    className="inline-flex min-h-9 items-center gap-2 rounded-full border border-border bg-surface-muted px-3 py-1 text-xs font-medium text-card-foreground"
                  >
                    <span>{location}</span>
                    <Button
                      type="button"
                      variant="icon"
                      size="icon"
                      className="h-6 w-6 rounded-full"
                      onClick={() => onRemovePreferredLocation(location)}
                      aria-label={`Remove ${location}`}
                      title={`Remove ${location}`}
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
        <fieldset id="profile-field-work-mode-preference" className="fieldGroup">
          <legend className="fieldLabel">Work mode preference</legend>
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            {PROFILE_WORK_MODE_OPTIONS.map((option) => (
              <label
                key={option.id}
                className={cn(
                  'flex cursor-pointer items-center gap-3 rounded-2xl border border-border bg-card/70 px-4 py-3.5 text-sm transition-colors',
                  workModePreference === option.id &&
                    'border-primary/35 bg-primary/8 text-card-foreground shadow-[inset_0_0_0_1px_rgba(149,167,255,0.1)]',
                )}
              >
                <input
                  type="radio"
                  name="work_mode_preference"
                  value={option.id}
                  checked={workModePreference === option.id}
                  onChange={() => setWorkModePreference(option.id)}
                  className="h-4 w-4 accent-[var(--color-primary)]"
                />
                <span className="leading-6">{option.label}</span>
              </label>
            ))}
          </div>
        </fieldset>
        <ExperienceTimelineEditor
          entries={experience}
          onAdd={onAddExperience}
          onUpdate={onUpdateExperience}
          onRemove={onRemoveExperience}
        />
        <label id="profile-field-cv">
          <span className="flex items-center justify-between gap-3">
            CV / текст профілю
            <Button type="button" variant="ghost" size="sm" onClick={onOpenFilePicker}>
              <Upload size={13} />
              Завантажити .pdf / .txt / .md
            </Button>
          </span>
          <input
            ref={fileInputRef}
            type="file"
            accept=".pdf,.txt,.md,.text"
            className="hidden"
            onChange={onFileChange}
          />
          <textarea
            value={rawText}
            onChange={(event) => setRawText(event.target.value)}
            rows={12}
            placeholder="Вставте ваше CV, досвід, навички та цільові ролі. Або натисніть «Завантажити» для .txt / .md файлу."
            required
          />
        </label>

        <Button
          type="submit"
          disabled={isSaving || !name || !email || !rawText.trim()}
          className="w-full md:w-auto"
        >
          {isSaving ? 'Saving…' : profileExists ? 'Update Profile' : 'Create Profile'}
        </Button>
      </SurfaceSection>
    </>
  );
}

const EMPTY_EXPERIENCE_ENTRY: ExperienceEntry = {
  company: '',
  role: '',
  from: '',
  to: undefined,
  description: undefined,
};

function ExperienceTimelineEditor({
  entries,
  onAdd,
  onUpdate,
  onRemove,
}: {
  entries: ExperienceEntry[];
  onAdd: (entry: ExperienceEntry) => void;
  onUpdate: (index: number, entry: ExperienceEntry) => void;
  onRemove: (index: number) => void;
}) {
  const [draft, setDraft] = useState<ExperienceEntry>(EMPTY_EXPERIENCE_ENTRY);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);

  const sortedEntries = [...entries].sort((left, right) => right.from.localeCompare(left.from));
  const canSave = Boolean(draft.company.trim() && draft.role.trim() && draft.from.trim());

  function resetDraft() {
    setDraft(EMPTY_EXPERIENCE_ENTRY);
    setEditingIndex(null);
  }

  function submitDraft() {
    if (!canSave) return;
    const entry = normalizeExperienceEntry(draft);
    if (editingIndex === null) {
      onAdd(entry);
    } else {
      onUpdate(editingIndex, entry);
    }
    resetDraft();
  }

  function startEdit(index: number) {
    setDraft(sortedEntries[index]);
    setEditingIndex(index);
  }

  return (
    <section id="profile-field-experience" className="fieldGroup">
      <div className="flex items-center justify-between gap-3">
        <span className="fieldLabel">Experience timeline</span>
        {editingIndex !== null && (
          <Button type="button" variant="ghost" size="sm" onClick={resetDraft}>
            Cancel edit
          </Button>
        )}
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        <label className="m-0">
          Company
          <input
            value={draft.company}
            onChange={(event) => setDraft({ ...draft, company: event.target.value })}
            placeholder="Company"
          />
        </label>
        <label className="m-0">
          Role
          <input
            value={draft.role}
            onChange={(event) => setDraft({ ...draft, role: event.target.value })}
            placeholder="Senior Frontend Engineer"
          />
        </label>
        <label className="m-0">
          From
          <input
            type="month"
            value={draft.from}
            onChange={(event) => setDraft({ ...draft, from: event.target.value })}
          />
        </label>
        <label className="m-0">
          To <span className="text-muted-foreground">(blank means current)</span>
          <input
            type="month"
            value={draft.to ?? ''}
            onChange={(event) => setDraft({ ...draft, to: event.target.value || undefined })}
          />
        </label>
        <label className="m-0 md:col-span-2">
          Description <span className="text-muted-foreground">(optional)</span>
          <textarea
            value={draft.description ?? ''}
            onChange={(event) =>
              setDraft({ ...draft, description: event.target.value || undefined })
            }
            rows={3}
            placeholder="Scope, impact, stack, or responsibilities"
          />
        </label>
      </div>

      <Button type="button" variant="outline" onClick={submitDraft} disabled={!canSave}>
        <BriefcaseBusiness className="h-4 w-4" />
        {editingIndex === null ? 'Add experience' : 'Save experience'}
      </Button>

      {sortedEntries.length > 0 && (
        <ol className="space-y-3">
          {sortedEntries.map((entry, index) => (
            <li
              key={`${entry.company}-${entry.role}-${entry.from}-${index}`}
              className="rounded-[var(--radius-card)] border border-border bg-surface-muted p-4"
            >
              <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="m-0 font-semibold text-card-foreground">
                      {entry.role} at {entry.company}
                    </p>
                    {!entry.to && (
                      <span className="rounded-full border border-fit-excellent/25 bg-fit-excellent/12 px-2 py-0.5 text-xs font-semibold text-fit-excellent">
                        Current
                      </span>
                    )}
                  </div>
                  <p className="m-0 mt-1 text-sm text-muted-foreground">
                    {formatExperienceDate(entry.from)} -{' '}
                    {entry.to ? formatExperienceDate(entry.to) : 'Current'}
                  </p>
                  {entry.description && (
                    <p className="m-0 mt-3 whitespace-pre-wrap text-sm leading-6 text-card-foreground">
                      {entry.description}
                    </p>
                  )}
                </div>
                <div className="flex gap-2">
                  <Button
                    type="button"
                    variant="icon"
                    size="icon"
                    onClick={() => startEdit(index)}
                    aria-label={`Edit ${entry.role} at ${entry.company}`}
                    title={`Edit ${entry.role} at ${entry.company}`}
                  >
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <Button
                    type="button"
                    variant="icon"
                    size="icon"
                    onClick={() => onRemove(index)}
                    aria-label={`Remove ${entry.role} at ${entry.company}`}
                    title={`Remove ${entry.role} at ${entry.company}`}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

function normalizeExperienceEntry(entry: ExperienceEntry): ExperienceEntry {
  return {
    company: entry.company.trim(),
    role: entry.role.trim(),
    from: entry.from.trim(),
    to: entry.to?.trim() || undefined,
    description: entry.description?.trim() || undefined,
  };
}

function formatExperienceDate(value: string): string {
  if (!value) return '';
  const [year, month] = value.split('-');
  if (!year || !month) return value;
  const date = new Date(Number(year), Number(month) - 1, 1);
  return new Intl.DateTimeFormat('en', { month: 'short', year: 'numeric' }).format(date);
}
