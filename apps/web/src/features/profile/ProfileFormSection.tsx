import type { ChangeEventHandler, RefObject } from 'react';
import { Upload } from 'lucide-react';

import { Button } from '../../components/ui/Button';
import { OptionCardGroup } from '../../components/ui/OptionCardGroup';
import {
  PROFILE_LANGUAGE_OPTIONS,
  PROFILE_SALARY_CURRENCY_OPTIONS,
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
  onToggleLanguage,
}: {
  name: string;
  email: string;
  location: string;
  rawText: string;
  yearsOfExperience: string;
  salaryMin: string;
  salaryMax: string;
  salaryCurrency: string;
  languages: string[];
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
  onToggleLanguage: (value: string) => void;
}) {
  return (
    <>
      <div className="flex flex-col gap-5 rounded-[28px] border border-border bg-card/85 p-7 shadow-[var(--shadow-hero)] md:flex-row md:items-end md:justify-between">
        <div className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/20 bg-primary/12 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-primary">
              Persisted profile
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white/[0.04] px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
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
      </div>

      <form
        className="flex flex-col gap-5 rounded-[24px] border border-border bg-card/85 p-7"
        onSubmit={(event) => {
          event.preventDefault();
          onSave();
        }}
      >
        <label>
          Name
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Your name"
            required
          />
        </label>
        <label>
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
        <div className="fieldGroup">
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
        <div className="fieldGroup">
          <span className="fieldLabel">Languages</span>
          <OptionCardGroup
            options={PROFILE_LANGUAGE_OPTIONS.map((option) => ({
              id: option.id,
              label: option.label,
            }))}
            value={languages}
            onToggle={onToggleLanguage}
          />
        </div>
        <label>
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
      </form>
    </>
  );
}
