import { useEffect, useMemo, useState } from 'react';
import { Check, ClipboardCopy, Loader2, Sparkles, X } from 'lucide-react';
import type { CvTailoringResponse } from '@job-copilot/shared/cv-tailoring';

import { tailorCvForJob } from '../../api/cvTailoring';
import { useToast } from '../../context/ToastContext';
import { cn } from '../../lib/cn';
import { Badge } from '../ui/Badge';
import { Button } from '../ui/Button';

type CvTailoringModalProps = {
  jobId: string;
  open: boolean;
  onClose: () => void;
};

type CopyKey =
  | 'highlight'
  | 'mention'
  | 'gaps'
  | 'summary'
  | 'phrases'
  | 'all';

export function CvTailoringModal({
  jobId,
  open,
  onClose,
}: CvTailoringModalProps) {
  const { showToast } = useToast();
  const [data, setData] = useState<CvTailoringResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [copiedKey, setCopiedKey] = useState<CopyKey | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    let cancelled = false;

    async function loadTailoringSuggestions() {
      setIsLoading(true);
      setError(null);

      try {
        const response = await tailorCvForJob(jobId);

        if (!cancelled) {
          setData(response);
        }
      } catch (value) {
        if (!cancelled) {
          setError(
            value instanceof Error
              ? value.message
              : 'Failed to generate CV tailoring suggestions.',
          );
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    }

    void loadTailoringSuggestions();

    return () => {
      cancelled = true;
    };
  }, [jobId, open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose();
      }
    }

    document.addEventListener('keydown', handleKeyDown);

    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onClose, open]);

  const copyAllText = useMemo(() => {
    if (!data) {
      return '';
    }

    const { suggestions } = data;
    const sections: string[] = [];

    if (suggestions.skillsToHighlight.length > 0) {
      sections.push(
        `Skills to Highlight:\n${suggestions.skillsToHighlight.join(', ')}`,
      );
    }

    if (suggestions.skillsToMention.length > 0) {
      sections.push(
        `Skills to Mention:\n${suggestions.skillsToMention.join(', ')}`,
      );
    }

    if (suggestions.gapsToAddress.length > 0) {
      sections.push(
        [
          'Skill Gaps:',
          ...suggestions.gapsToAddress.map((gap) =>
            gap.suggestion
              ? `- ${gap.skill}: ${gap.suggestion}`
              : `- ${gap.skill}`,
          ),
        ].join('\n'),
      );
    }

    if (suggestions.summaryRewrite) {
      sections.push(`Suggested Summary:\n${suggestions.summaryRewrite}`);
    }

    if (suggestions.keyPhrases.length > 0) {
      sections.push(`Key Phrases:\n${suggestions.keyPhrases.join(', ')}`);
    }

    return sections.join('\n\n');
  }, [data]);

  async function copyText(key: CopyKey, text: string) {
    if (!text.trim()) {
      return;
    }

    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
      } else {
        const textarea = document.createElement('textarea');
        textarea.value = text;
        textarea.setAttribute('readonly', 'true');
        textarea.style.position = 'fixed';
        textarea.style.left = '-9999px';
        textarea.style.top = '0';
        document.body.appendChild(textarea);
        textarea.focus();
        textarea.select();

        const copied = document.execCommand('copy');
        document.body.removeChild(textarea);

        if (!copied) {
          throw new Error('Clipboard copy failed');
        }
      }

      setCopiedKey(key);
      showToast({
        type: 'success',
        message: 'Copied!',
      });

      window.setTimeout(() => {
        setCopiedKey((current) => (current === key ? null : current));
      }, 2000);
    } catch {
      showToast({
        type: 'error',
        message: 'Could not copy',
        description: 'Copy the text manually from the modal.',
      });
    }
  }

  if (!open) {
    return null;
  }

  const suggestions = data?.suggestions;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4 py-6 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="cv-tailoring-title"
      onMouseDown={onClose}
    >
      <div
        className="flex max-h-[88vh] w-full max-w-3xl flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border bg-surface shadow-2xl"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="flex items-start justify-between gap-4 border-b border-border/80 px-5 py-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.18em] text-primary">
              <Sparkles className="h-4 w-4" />
              CV tailoring
            </div>

            <h2 id="cv-tailoring-title" className="text-xl font-semibold">
              Tailor CV for this role
            </h2>

            <p className="max-w-2xl text-sm text-muted-foreground">
              Suggestions are generated from your profile and this job
              description. Use them as a checklist before applying.
            </p>
          </div>

          <Button
            type="button"
            variant="icon"
            size="icon"
            aria-label="Close CV tailoring modal"
            onClick={onClose}
          >
            <X className="h-4 w-4" />
          </Button>
        </header>

        <main className="overflow-y-auto px-5 py-5">
          {isLoading ? <CvTailoringSkeleton /> : null}

          {!isLoading && error ? (
            <div className="rounded-[var(--radius-lg)] border border-destructive/30 bg-destructive/10 p-4 text-sm text-destructive">
              {error}
            </div>
          ) : null}

          {!isLoading && !error && suggestions ? (
            <div className="space-y-5">
              {suggestions.skillsToHighlight.length > 0 ? (
                <SuggestionSection
                  title="Skills to Highlight"
                  description="Put these closer to the top of your CV or summary."
                  action={
                    <CopyButton
                      copied={copiedKey === 'highlight'}
                      label="Copy skills list"
                      onClick={() =>
                        copyText(
                          'highlight',
                          suggestions.skillsToHighlight.join(', '),
                        )
                      }
                    />
                  }
                >
                  <ChipList
                    values={suggestions.skillsToHighlight}
                    variant="green"
                  />
                </SuggestionSection>
              ) : null}

              {suggestions.skillsToMention.length > 0 ? (
                <SuggestionSection
                  title="Skills to Mention"
                  description="Useful supporting skills to mention in bullets or project context."
                  action={
                    <CopyButton
                      copied={copiedKey === 'mention'}
                      label="Copy skills list"
                      onClick={() =>
                        copyText(
                          'mention',
                          suggestions.skillsToMention.join(', '),
                        )
                      }
                    />
                  }
                >
                  <ChipList
                    values={suggestions.skillsToMention}
                    variant="blue"
                  />
                </SuggestionSection>
              ) : null}

              {suggestions.gapsToAddress.length > 0 ? (
                <SuggestionSection
                  title="Skill Gaps"
                  description="Do not fake experience. Use these as prompts for honest adjacent experience."
                  action={
                    <CopyButton
                      copied={copiedKey === 'gaps'}
                      label="Copy gaps"
                      onClick={() =>
                        copyText(
                          'gaps',
                          suggestions.gapsToAddress
                            .map((gap) =>
                              gap.suggestion
                                ? `${gap.skill}: ${gap.suggestion}`
                                : gap.skill,
                            )
                            .join('\n'),
                        )
                      }
                    />
                  }
                >
                  <div className="grid gap-2">
                    {suggestions.gapsToAddress.map((gap) => (
                      <div
                        key={`${gap.skill}-${gap.suggestion}`}
                        className="rounded-[var(--radius-md)] border border-fit-fair/25 bg-fit-fair/10 px-3 py-2"
                        title={gap.suggestion}
                      >
                        <div className="text-sm font-semibold text-fit-fair">
                          {gap.skill}
                        </div>

                        {gap.suggestion ? (
                          <p className="mt-1 text-xs leading-5 text-muted-foreground">
                            {gap.suggestion}
                          </p>
                        ) : null}
                      </div>
                    ))}
                  </div>
                </SuggestionSection>
              ) : null}

              {suggestions.summaryRewrite ? (
                <SuggestionSection
                  title="Suggested Summary"
                  description="Selectable text. Adjust tone and facts before using it."
                  action={
                    <CopyButton
                      copied={copiedKey === 'summary'}
                      label="Copy summary"
                      onClick={() =>
                        copyText('summary', suggestions.summaryRewrite)
                      }
                    />
                  }
                >
                  <pre className="whitespace-pre-wrap rounded-[var(--radius-lg)] border border-border bg-background/60 p-4 text-sm leading-6 text-foreground">
                    {suggestions.summaryRewrite}
                  </pre>
                </SuggestionSection>
              ) : null}

              {suggestions.keyPhrases.length > 0 ? (
                <SuggestionSection
                  title="Key Phrases"
                  description="Small phrases that may help mirror the job language."
                  action={
                    <CopyButton
                      copied={copiedKey === 'phrases'}
                      label="Copy phrases"
                      onClick={() =>
                        copyText('phrases', suggestions.keyPhrases.join(', '))
                      }
                    />
                  }
                >
                  <ChipList values={suggestions.keyPhrases} variant="gray" />
                </SuggestionSection>
              ) : null}

              <div className="flex flex-col gap-3 border-t border-border pt-5 sm:flex-row sm:items-center sm:justify-between">
                <p className="text-xs text-muted-foreground">
                  Provider: {data.provider} · Generated at{' '}
                  {new Date(data.generatedAt).toLocaleString()}
                </p>

                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => copyText('all', copyAllText)}
                  disabled={!copyAllText}
                >
                  {copiedKey === 'all' ? (
                    <Check className="h-4 w-4" />
                  ) : (
                    <ClipboardCopy className="h-4 w-4" />
                  )}
                  {copiedKey === 'all' ? 'Copied!' : 'Copy all'}
                </Button>
              </div>
            </div>
          ) : null}
        </main>
      </div>
    </div>
  );
}

function CvTailoringSkeleton() {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3 rounded-[var(--radius-lg)] border border-border bg-white-a04 p-4">
        <Loader2 className="h-5 w-5 animate-spin text-primary" />
        <div>
          <p className="text-sm font-semibold">Generating suggestions…</p>
          <p className="text-xs text-muted-foreground">
            Reading job details and your profile.
          </p>
        </div>
      </div>

      {[0, 1, 2].map((item) => (
        <div
          key={item}
          className="rounded-[var(--radius-lg)] border border-border bg-white-a04 p-4"
        >
          <div className="h-4 w-40 animate-pulse rounded bg-white-a07" />
          <div className="mt-4 flex flex-wrap gap-2">
            <div className="h-7 w-24 animate-pulse rounded-full bg-white-a07" />
            <div className="h-7 w-32 animate-pulse rounded-full bg-white-a07" />
            <div className="h-7 w-20 animate-pulse rounded-full bg-white-a07" />
          </div>
        </div>
      ))}
    </div>
  );
}

function SuggestionSection({
  title,
  description,
  action,
  children,
}: {
  title: string;
  description?: string;
  action?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-[var(--radius-lg)] border border-border bg-white-a04 p-4">
      <div className="mb-3 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h3 className="text-sm font-semibold">{title}</h3>

          {description ? (
            <p className="mt-1 text-xs leading-5 text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>

        {action}
      </div>

      {children}
    </section>
  );
}

function ChipList({
  values,
  variant,
}: {
  values: string[];
  variant: 'green' | 'blue' | 'orange' | 'gray';
}) {
  return (
    <div className="flex flex-wrap gap-2">
      {values.map((value) => (
        <Badge
          key={value}
          className={cn(
            'px-2.5 py-1 text-xs',
            variant === 'green' &&
              'border-fit-excellent/25 bg-fit-excellent/10 text-fit-excellent',
            variant === 'blue' &&
              'border-primary/25 bg-primary/10 text-primary',
            variant === 'orange' &&
              'border-fit-fair/25 bg-fit-fair/10 text-fit-fair',
            variant === 'gray' && 'border-border bg-white-a04 text-muted-foreground',
          )}
        >
          {value}
        </Badge>
      ))}
    </div>
  );
}

function CopyButton({
  copied,
  label,
  onClick,
}: {
  copied: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button type="button" variant="outline" size="sm" onClick={onClick}>
      {copied ? (
        <Check className="h-4 w-4" />
      ) : (
        <ClipboardCopy className="h-4 w-4" />
      )}
      {copied ? 'Copied!' : label}
    </Button>
  );
}