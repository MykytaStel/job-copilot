import { useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { CheckCircle2, FileText, Trash2, Upload } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import {
  activateResume,
  deleteResume,
  getProfile,
  getResumes,
  getStoredProfileRawText,
  uploadResume,
} from '../../api/profiles';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { formatOptionalDate } from '../../lib/format';
import { queryKeys } from '../../queryKeys';
import { buildProfileCompletionState } from '../profile/profileCompletion';
import { SettingsSection, SettingRow } from './settingsShared';

export function ProfileSettingsSection() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const resumeUploadInputRef = useRef<HTMLInputElement>(null);
  const [resumeUploadError, setResumeUploadError] = useState<string | null>(null);

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });
  const { data: resumes = [], isLoading: resumesLoading } = useQuery({
    queryKey: queryKeys.resumes.all(),
    queryFn: getResumes,
    enabled: !!profile,
  });
  const { data: rawText = '' } = useQuery({
    queryKey: queryKeys.profile.rawText(),
    queryFn: getStoredProfileRawText,
    enabled: !!profile,
  });

  const activateResumeMutation = useMutation({
    mutationFn: activateResume,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.active() });
    },
  });

  const uploadResumeMutation = useMutation({
    mutationFn: uploadResume,
    onMutate: () => {
      setResumeUploadError(null);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.active() });
    },
  });

  const deleteResumeMutation = useMutation({
    mutationFn: deleteResume,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.resumes.active() });
    },
  });

  if (!profile) return null;

  const completion = buildProfileCompletionState({
    name: profile.name,
    email: profile.email,
    rawText,
    skills: profile.skills,
    salaryMin: profile.salaryMin ? String(profile.salaryMin) : '',
    salaryMax: profile.salaryMax ? String(profile.salaryMax) : '',
    salaryCurrency: profile.salaryCurrency ?? '',
    languages: profile.languages,
    preferredLocations: profile.preferredLocations,
    targetRegions: profile.searchPreferences?.targetRegions ?? [],
    workModes: profile.searchPreferences?.workModes ?? [],
    preferredRoles: profile.searchPreferences?.preferredRoles ?? [],
  });

  function confirmDeleteResume(resume: { id: string; filename: string }) {
    const confirmed = window.confirm(`Delete CV "${resume.filename}"? This cannot be undone.`);
    if (confirmed) deleteResumeMutation.mutate(resume.id);
  }

  async function handleResumeUpload(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file) return;

    try {
      const text = await readResumeFile(file);
      uploadResumeMutation.mutate({ filename: file.name, rawText: text });
    } catch (error) {
      console.error('Resume upload failed before API submission:', error);
      setResumeUploadError('Use PDF, TXT, or MD under 5 MB with readable text.');
    }
  }

  return (
    <SettingsSection title="Profile & Account">
      <div className="space-y-6">
        <div className="grid gap-3 md:grid-cols-2">
          <SettingRow label="Name" value={profile.name} />
          <SettingRow label="Email" value={profile.email} />
          <SettingRow label="Location" value={profile.location ?? 'Not set'} />
          <SettingRow
            label="Preferred locations"
            value={
              profile.preferredLocations.length > 0
                ? profile.preferredLocations.join(', ')
                : 'Not set'
            }
          />
          <SettingRow
            label="Compensation"
            value={
              profile.salaryMin && profile.salaryMax
                ? `${profile.salaryMin}–${profile.salaryMax} ${profile.salaryCurrency}`
                : 'Not set'
            }
          />
        </div>

        <div>
          <p className="mb-2 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            Languages
          </p>
          <div className="flex flex-wrap gap-2">
            {profile.languages.length > 0 ? (
              profile.languages.map((lang) => (
                <Badge key={lang.language} variant="muted" className="px-2 py-0.5">
                  {lang.language} · {lang.level}
                </Badge>
              ))
            ) : (
              <Badge variant="muted" className="px-2 py-0.5">
                No languages set
              </Badge>
            )}
          </div>
        </div>

        <div>
          <p className="mb-1 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            Profile readiness
          </p>
          <p className="mt-1 text-2xl font-semibold text-card-foreground">{completion.percent}%</p>
          <div className="mt-2 h-2 rounded-full bg-surface-soft">
            <div
              className="h-2 rounded-full bg-[image:var(--gradient-button)]"
              style={{ width: `${completion.percent}%` }}
            />
          </div>
          {completion.missingLabels.length > 0 && (
            <p className="mt-1 text-sm text-muted-foreground">
              Missing: {completion.missingLabels.join(', ')}
            </p>
          )}
        </div>

        <div className="rounded-[var(--radius-lg)] border border-border bg-surface-soft/40 p-4">
          <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p className="text-sm font-semibold text-foreground">CV Management</p>
              <p className="mt-1 text-sm text-muted-foreground">
                Review uploaded CV versions and choose which one powers matching flows. Profile
                edits reuse the active CV unless the CV text changes.
              </p>
            </div>

            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => resumeUploadInputRef.current?.click()}
              disabled={uploadResumeMutation.isPending}
              className="shrink-0"
            >
              <Upload className="h-4 w-4" />
              {uploadResumeMutation.isPending ? 'Uploading CV' : 'Upload new CV'}
            </Button>
            <input
              ref={resumeUploadInputRef}
              type="file"
              accept=".pdf,.txt,.md,.text,application/pdf,text/plain,text/markdown"
              className="hidden"
              onChange={handleResumeUpload}
            />
          </div>

          {resumesLoading && (
            <p className="text-sm text-muted-foreground">Loading uploaded CVs…</p>
          )}

          {!resumesLoading && resumes.length === 0 && (
            <EmptyState
              icon={<FileText className="h-5 w-5" />}
              message="No CVs uploaded"
              description="Use Profile to upload a CV and create the first resume version."
              className="bg-surface"
            />
          )}

          {!resumesLoading && resumes.length > 0 && (
            <div className="space-y-3">
              {resumes.map((resume) => {
                const isActivating =
                  activateResumeMutation.isPending &&
                  activateResumeMutation.variables === resume.id;
                const isDeleting =
                  deleteResumeMutation.isPending && deleteResumeMutation.variables === resume.id;

                return (
                  <div
                    key={resume.id}
                    className="flex flex-col gap-3 rounded-[var(--radius-md)] border border-border bg-surface p-3 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <div className="flex min-w-0 items-center gap-3">
                      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-md)] border border-primary/25 bg-primary/10 text-primary">
                        <FileText className="h-4 w-4" />
                      </div>
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="truncate text-sm font-medium text-foreground">
                            {resume.filename}
                          </p>
                          {resume.isActive && (
                            <Badge
                              variant="default"
                              className="border-0 bg-primary/15 px-2 py-0.5 text-[10px] uppercase tracking-[0.14em] text-primary"
                            >
                              <CheckCircle2 className="h-3 w-3" />
                              Active
                            </Badge>
                          )}
                        </div>
                        <p className="mt-1 text-xs text-muted-foreground">
                          Uploaded {formatOptionalDate(resume.uploadedAt) ?? 'n/a'}
                        </p>
                      </div>
                    </div>

                    <div className="flex flex-wrap gap-2 sm:justify-end">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => activateResumeMutation.mutate(resume.id)}
                        disabled={
                          resume.isActive ||
                          isActivating ||
                          activateResumeMutation.isPending ||
                          deleteResumeMutation.isPending
                        }
                      >
                        {isActivating ? 'Activating' : 'Activate'}
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => confirmDeleteResume(resume)}
                        disabled={isDeleting || activateResumeMutation.isPending}
                        className="text-danger hover:text-danger"
                      >
                        <Trash2 className="h-4 w-4" />
                        {isDeleting ? 'Deleting' : 'Delete'}
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}

          {activateResumeMutation.isError && (
            <p className="mt-3 text-sm text-danger">Could not activate this CV. Please try again.</p>
          )}
          {deleteResumeMutation.isError && (
            <p className="mt-3 text-sm text-danger">Could not delete this CV. Please try again.</p>
          )}
          {(uploadResumeMutation.isError || resumeUploadError) && (
            <p className="mt-3 text-sm text-danger">
              {resumeUploadError ?? 'Could not upload this CV. Use PDF, TXT, or MD under 5 MB.'}
            </p>
          )}
        </div>

        <Button onClick={() => navigate('/profile')}>Open profile</Button>
      </div>
    </SettingsSection>
  );
}

async function readResumeFile(file: File): Promise<string> {
  const maxSizeBytes = 5 * 1024 * 1024;

  if (file.size > maxSizeBytes) throw new Error('Resume file is too large');

  const extension = file.name.split('.').pop()?.toLowerCase();
  const allowedExtensions = new Set(['pdf', 'txt', 'md', 'text']);

  if (!allowedExtensions.has(extension ?? '')) throw new Error('Unsupported resume file type');

  if (file.type === 'application/pdf' || extension === 'pdf') {
    const { extractPdfText } = await import('../profile/profile.pdf.utils');
    const text = await extractPdfText(file);
    if (!text.trim()) throw new Error('Resume file is empty');
    return text;
  }

  const { cleanupExtractedResumeText } = await import('../profile/profile.pdf.utils');
  const text = cleanupExtractedResumeText(await file.text());
  if (!text.trim()) throw new Error('Resume file is empty');
  return text;
}
