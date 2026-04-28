import type { ResumeVersion } from '@job-copilot/shared/profiles';
import { CheckCircle2, Download, FileText } from 'lucide-react';
import { Badge } from './ui/Badge';
import { Button } from './ui/Button';
import { SurfaceSection } from './ui/Surface';

type ResumeHistoryProps = {
  resumes: ResumeVersion[];
  activatingResumeId?: string;
  onActivate: (resumeId: string) => void;
};

export function ResumeHistory({ resumes, activatingResumeId, onActivate }: ResumeHistoryProps) {
  return (
    <SurfaceSection className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="eyebrow">CV history</p>
          <h3 className="m-0 text-base font-semibold text-card-foreground">Uploaded CV versions</h3>
        </div>
        <FileText className="h-5 w-5 text-muted-foreground" />
      </div>

      {resumes.length === 0 ? (
        <p className="m-0 text-sm text-muted-foreground">No CVs uploaded yet. Upload one above.</p>
      ) : (
        <div className="space-y-3">
          {resumes.map((resume) => {
            const isActivating = activatingResumeId === resume.id;
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
                    </div>
                    <p className="m-0 mt-1 text-xs text-muted-foreground">
                      Uploaded {formatUploadedDate(resume.uploadedAt)}
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

function formatUploadedDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(date);
}
