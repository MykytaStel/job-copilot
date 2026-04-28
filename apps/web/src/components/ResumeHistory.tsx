import type { ResumeVersion } from '@job-copilot/shared/profiles';
import { CheckCircle2, Download, FileText, Trash2 } from 'lucide-react';
import { Badge } from './ui/Badge';
import { Button } from './ui/Button';
import { SurfaceSection } from './ui/Surface';

type ResumeHistoryProps = {
  resumes: ResumeVersion[];
  activatingResumeId?: string;
  deletingResumeId?: string;
  onActivate: (resumeId: string) => void;
  onDelete?: (resumeId: string) => void;
};

export function ResumeHistory({
  resumes,
  activatingResumeId,
  deletingResumeId,
  onActivate,
  onDelete,
}: ResumeHistoryProps) {
  const duplicateCounts = buildDuplicateCounts(resumes);

  return (
    <SurfaceSection className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="eyebrow">CV history</p>
          <h3 className="m-0 text-base font-semibold text-card-foreground">Uploaded CV versions</h3>
          <p className="m-0 mt-1 text-sm text-muted-foreground">
            A new version is created only when the CV text changes.
          </p>
        </div>
        <FileText className="h-5 w-5 text-muted-foreground" />
      </div>

      {resumes.length === 0 ? (
        <p className="m-0 text-sm text-muted-foreground">No CVs uploaded yet. Upload one above.</p>
      ) : (
        <div className="space-y-3">
          {resumes.map((resume) => {
            const isActivating = activatingResumeId === resume.id;
            const isDeleting = deletingResumeId === resume.id;
            const duplicateCount = duplicateCounts.get(normalizeResumeText(resume.rawText)) ?? 0;
            const isDuplicate = duplicateCount > 1;
            return (
              <div
                key={resume.id}
                className={
                  resume.isActive
                    ? 'rounded-[var(--radius-card)] border border-primary/35 bg-primary/8 p-4'
                    : 'rounded-[var(--radius-card)] border border-border bg-surface-muted p-4'
                }
              >
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="m-0 truncate text-sm font-semibold text-card-foreground">
                        {resume.filename}
                      </p>
                      {resume.isActive && (
                        <Badge variant="success" className="px-2 py-0.5 text-xs">
                          <CheckCircle2 className="h-3.5 w-3.5" />
                          Active
                        </Badge>
                      )}
                      {isDuplicate && (
                        <Badge variant="warning" className="px-2 py-0.5 text-xs">
                          Duplicate text
                        </Badge>
                      )}
                    </div>
                    <p className="m-0 mt-1 text-xs text-muted-foreground">
                      Uploaded {formatUploadedDate(resume.uploadedAt)}
                      {isDuplicate ? ` • ${duplicateCount} versions share this CV text` : ''}
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {resume.downloadUrl && (
                      <a
                        href={resume.downloadUrl}
                        className="inline-flex h-9 items-center justify-center gap-2 rounded-[var(--radius-lg)] border border-border px-3.5 text-xs font-semibold text-foreground hover:bg-white-a05"
                      >
                        <Download className="h-4 w-4" />
                        Download
                      </a>
                    )}
                    <Button
                      type="button"
                      variant={resume.isActive ? 'ghost' : 'outline'}
                      size="sm"
                      onClick={() => onActivate(resume.id)}
                      disabled={resume.isActive || Boolean(activatingResumeId)}
                    >
                      {isActivating ? 'Activating...' : resume.isActive ? 'Active' : 'Activate'}
                    </Button>
                    {onDelete && !resume.isActive && isDuplicate ? (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => onDelete(resume.id)}
                        disabled={isDeleting || Boolean(activatingResumeId)}
                      >
                        <Trash2 className="h-4 w-4" />
                        {isDeleting ? 'Deleting...' : 'Delete duplicate'}
                      </Button>
                    ) : null}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </SurfaceSection>
  );
}

function normalizeResumeText(value: string): string {
  return value.replace(/\r\n/g, '\n').trim();
}

function buildDuplicateCounts(resumes: ResumeVersion[]) {
  const counts = new Map<string, number>();
  for (const resume of resumes) {
    const key = normalizeResumeText(resume.rawText);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return counts;
}

function formatUploadedDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(date);
}
