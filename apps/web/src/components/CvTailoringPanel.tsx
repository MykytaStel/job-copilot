import { useMemo, useState, type ReactNode } from 'react';
import {
  Check,
  ClipboardCopy,
  FileText,
  Loader2,
  RefreshCcw,
  Sparkles,
  X,
} from 'lucide-react';
import type { CvTailoringResponse } from '@job-copilot/shared/cv-tailoring';

import { tailorCvForJob } from '../api/cvTailoring';
import { useToast } from '../context/ToastContext';
import { cn } from '../lib/cn';
import { Badge } from './ui/Badge';
import { Button } from './ui/Button';

type TailoringSectionId = 'summary' | 'skills' | 'gaps' | 'phrases';

type CvTailoringPanelProps = {
  jobId: string;
  open: boolean;
  onClose: () => void;
};

type CopyKey = TailoringSectionId | 'all';

const sectionOptions: Array<{
  id: TailoringSectionId;
  label: string;
  description: string;
}> = [
  {
    id: 'summary',
    label: 'Summary',
    description: 'Rewrite the top profile summary for this job.',
  },
  {
    id: 'skills',
    label: 'Skills',
    description: 'Prioritize matched and adjacent skills.',
  },
  {
    id: 'gaps',
    label: 'Gaps',
    description: 'Handle missing requirements honestly.',
  },
  {
    id: 'phrases',
    label: 'Key phrases',
    description: 'Mirror important language from the vacancy.',
  },
];

const defaultSelectedSections: TailoringSectionId[] = [
  'summary',
  'skills',
  'gaps',
];

export function CvTailoringPanel({
  jobId,
  open,
  onClose,
}: CvTailoringPanelProps) {
  const { showToast } = useToast();
  const [selectedSections, setSelectedSections] = useState<TailoringSectionId[]>(
    defaultSelectedSections,
  );
  const [data, setData] = useState<CvTailoringResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [copiedKey, setCopiedKey] = useState<CopyKey | null>(null);

  const suggestions = data?.suggestions;
  const selected = useMemo(
    () => new Set<TailoringSectionId>(selectedSections),
    [selectedSections],
  );

  const hasSelectedSections = selectedSections.length > 0;

  const copyAllText = useMemo(() => {
    if (!suggestions) {
      return '';
    }

    const blocks: string[] = [];

    if (selected.has('summary') && suggestions.summaryRewrite) {
      blocks.push(`Summary rewrite:\n${suggestions.summaryRewrite}`);
    }

    if (
      selected.has('skills') &&
      (suggestions.skillsToHighlight.length > 0 ||
        suggestions.skillsToMention.length > 0)
    ) {
      const skillLines = [
        suggestions.skillsToHighlight.length > 0
          ? `Highlight: ${suggestions.skillsToHighlight.join(', ')}`
          : '',
        suggestions.skillsToMention.length > 0
          ? `Mention: ${suggestions.skillsToMention.join(', ')}`
          : '',
      ].filter(Boolean);

      blocks.push(`Skills:\n${skillLines.join('\n')}`);
    }

    if (selected.has('gaps') && suggestions.gapsToAddress.length > 0) {
      blocks.push(
        [
          'Gaps:',
          ...suggestions.gapsToAddress.map((gap) =>
            gap.suggestion
              ? `${gap.skill}: ${gap.suggestion}`
              : gap.skill,
          ),
        ].join('\n'),
      );
    }

    if (selected.has('phrases') && suggestions.keyPhrases.length > 0) {
      blocks.push(`Key phrases:\n${suggestions.keyPhrases.join(', ')}`);
    }

    return blocks.join('\n\n');
  }, [selected, suggestions]);

  function toggleSection(section: TailoringSectionId) {
    setSelectedSections((current) =>
      current.includes(section)
        ? current.filter((item) => item !== section)
        : [...current, section],
    );
  }

  async function loadTailoringSuggestions() {
    if (!hasSelectedSections || isLoading) {
      return;
    }

    setIsLoading(true);
    setError(null);
    setCopiedKey(null);

    try {
      const response = await tailorCvForJob(jobId);
      setData(response);
    } catch (value) {
      setError(
        value instanceof Error
          ? value.message
          : 'Failed to generate CV tailoring suggestions.',
      );
    } finally {
      setIsLoading(false);
    }
  }

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
      showToast({ type: 'success', message: 'Copied!' });
      window.setTimeout(() => {
        setCopiedKey((current) => (current === key ? null : current));
      }, 2000);
    } catch {
      showToast({
        type: 'error',
        message: 'Could not copy',
        description: 'Copy the text manually from the panel.',
      });
    }
  }

  if (!open) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4 py-6 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="cv-tailoring-title"
      onMouseDown={onClose}
    >
      <div
        className="flex max-h-[90vh] w-full max-w-5xl flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border bg-surface shadow-2xl"
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

            <p className="max-w-3xl text-sm text-muted-foreground">
              Choose the CV sections you want to adapt, then review the
              generated suggestions before applying them.
            </p>
          </div>

          <Button
            type="button"
            variant="icon"
            size="icon"
            aria-label="Close CV tailoring panel"
            onClick={onClose}
          >
            <X className="h-4 w-4" />
          </Button>
        </header>

        <main className="grid min-h-0 flex-1 gap-0 overflow-hidden lg:grid-cols-[320px_minmax(0,1fr)]">
          <aside className="border-b border-border bg-white-a03 p-5 lg:border-b-0 lg:border-r">
            <div className="space-y-4">
              <div>
                <h3 className="text-sm font-semibold">Sections</h3>
                <p className="mt-1 text-xs leading-5 text-muted-foreground">
                  The engine generates a complete tailoring pass. This selection
                  controls which parts are shown and copied.
                </p>
              </div>

              <div className="grid gap-2">
                {sectionOptions.map((option) => (
                  <label
                    key={option.id}
                    className={cn(
                      'flex cursor-pointer items-start gap-3 rounded-[var(--radius-lg)] border border-border bg-card/70 px-3 py-3 text-sm transition-colors',
                      selected.has(option.id) &&
                        'border-primary/35 bg-primary/10 text-card-foreground',
                    )}
                  >
                    <input
                      type="checkbox"
                      checked={selected.has(option.id)}
                      onChange={() => toggleSection(option.id)}
                      className="mt-1 h-4 w-4 accent-[var(--color-primary)]"
                    />
                    <span>
                      <span className="block font-semibold">
                        {option.label}
                      </span>
                      <span className="mt-1 block text-xs leading-5 text-muted-foreground">
                        {option.description}
                      </span>
                    </span>
                  </label>
                ))}
              </div>

              {!hasSelectedSections ? (
                <p className="rounded-[var(--radius-md)] border border-fit-fair/25 bg-fit-fair/10 px-3 py-2 text-xs leading-5 text-fit-fair">
                  Select at least one section to generate suggestions.
                </p>
              ) : null}

              <Button
                type="button"
                className="w-full"
                disabled={!hasSelectedSections || isLoading}
                onClick={() => void loadTailoringSuggestions()}
              >
                {isLoading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : data ? (
                  <RefreshCcw className="h-4 w-4" />
                ) : (
                  <Sparkles className="h-4 w-4" />
                )}
                {isLoading
                  ? 'Generating'
                  : data
                    ? 'Regenerate'
                    : 'Generate suggestions'}
              </Button>

              {data ? (
                <div className="space-y-2 border-t border-border pt-4">
                  <p className="text-xs text-muted-foreground">
                    Provider: {data.provider}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Generated: {new Date(data.generatedAt).toLocaleString()}
                  </p>
                </div>
              ) : null}
            </div>
          </aside>

          <section className="min-h-0 overflow-y-auto p-5">
            {isLoading ? <CvTailoringSkeleton /> : null}

            {!isLoading && error ? (
              <div className="rounded-[var(--radius-lg)] border border-destructive/30 bg-destructive/10 p-4 text-sm text-destructive">
                {error}
              </div>
            ) : null}

            {!isLoading && !error && !suggestions ? (
              <EmptyPreview />
            ) : null}

            {!isLoading && !error && suggestions ? (
              <div className="space-y-5">
                {selected.has('summary') && suggestions.summaryRewrite ? (
                  <DiffSection
                    title="Summary"
                    description="Replace a generic top summary with a role-specific one."
                    copyLabel="Copy summary"
                    copied={copiedKey === 'summary'}
                    onCopy={() =>
                      copyText('summary', suggestions.summaryRewrite)
                    }
                  >
                    <DiffLine tone="remove">
                      Generic summary that lists experience without linking it
                      to this vacancy.
                    </DiffLine>
                    <DiffLine tone="add">
                      {suggestions.summaryRewrite}
                    </DiffLine>
                  </DiffSection>
                ) : null}

                {selected.has('skills') &&
                (suggestions.skillsToHighlight.length > 0 ||
                  suggestions.skillsToMention.length > 0) ? (
                  <DiffSection
                    title="Skills"
                    description="Move high-signal skills earlier and add supporting context where truthful."
                    copyLabel="Copy skills"
                    copied={copiedKey === 'skills'}
                    onCopy={() =>
                      copyText(
                        'skills',
                        [
                          suggestions.skillsToHighlight.length > 0
                            ? `Highlight: ${suggestions.skillsToHighlight.join(', ')}`
                            : '',
                          suggestions.skillsToMention.length > 0
                            ? `Mention: ${suggestions.skillsToMention.join(', ')}`
                            : '',
                        ]
                          .filter(Boolean)
                          .join('\n'),
                      )
                    }
                  >
                    {suggestions.skillsToHighlight.length > 0 ? (
                      <DiffLine tone="add">
                        Prioritize:{' '}
                        <ChipList
                          values={suggestions.skillsToHighlight}
                          variant="green"
                        />
                      </DiffLine>
                    ) : null}

                    {suggestions.skillsToMention.length > 0 ? (
                      <DiffLine tone="add">
                        Add supporting evidence for:{' '}
                        <ChipList
                          values={suggestions.skillsToMention}
                          variant="blue"
                        />
                      </DiffLine>
                    ) : null}
                  </DiffSection>
                ) : null}

                {selected.has('gaps') &&
                suggestions.gapsToAddress.length > 0 ? (
                  <DiffSection
                    title="Gaps"
                    description="Frame adjacent experience without overstating it."
                    copyLabel="Copy gaps"
                    copied={copiedKey === 'gaps'}
                    onCopy={() =>
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
                  >
                    {suggestions.gapsToAddress.map((gap) => (
                      <DiffLine
                        key={`${gap.skill}-${gap.suggestion}`}
                        tone="add"
                      >
                        <span className="font-semibold text-fit-fair">
                          {gap.skill}
                        </span>
                        {gap.suggestion ? (
                          <span className="mt-1 block text-muted-foreground">
                            {gap.suggestion}
                          </span>
                        ) : null}
                      </DiffLine>
                    ))}
                  </DiffSection>
                ) : null}

                {selected.has('phrases') && suggestions.keyPhrases.length > 0 ? (
                  <DiffSection
                    title="Key phrases"
                    description="Use this language only where it matches your real experience."
                    copyLabel="Copy phrases"
                    copied={copiedKey === 'phrases'}
                    onCopy={() =>
                      copyText('phrases', suggestions.keyPhrases.join(', '))
                    }
                  >
                    <DiffLine tone="add">
                      <ChipList values={suggestions.keyPhrases} variant="gray" />
                    </DiffLine>
                  </DiffSection>
                ) : null}

                {!copyAllText ? (
                  <div className="rounded-[var(--radius-lg)] border border-border bg-white-a04 p-4 text-sm text-muted-foreground">
                    No suggestions matched the selected sections. Try selecting
                    another section or regenerating.
                  </div>
                ) : null}

                <div className="flex flex-col gap-3 border-t border-border pt-5 sm:flex-row sm:items-center sm:justify-between">
                  <p className="text-xs text-muted-foreground">
                    Review each change before editing your CV. The model should
                    reframe evidence, not invent it.
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
                    {copiedKey === 'all' ? 'Copied!' : 'Copy selected'}
                  </Button>
                </div>
              </div>
            ) : null}
          </section>
        </main>
      </div>
    </div>
  );
}

function EmptyPreview() {
  return (
    <div className="flex min-h-[360px] items-center justify-center rounded-[var(--radius-lg)] border border-dashed border-border bg-white-a03 p-6 text-center">
      <div className="max-w-sm">
        <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-[var(--radius-lg)] border border-primary/25 bg-primary/10 text-primary">
          <FileText className="h-6 w-6" />
        </div>
        <h3 className="mt-4 text-base font-semibold">
          Select sections and generate
        </h3>
        <p className="mt-2 text-sm leading-6 text-muted-foreground">
          Suggestions will appear as focused edits you can copy into your CV.
        </p>
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
          <p className="text-sm font-semibold">Generating suggestions...</p>
          <p className="text-xs text-muted-foreground">
            Reading job details and your active profile.
          </p>
        </div>
      </div>

      {[0, 1, 2].map((item) => (
        <div
          key={item}
          className="rounded-[var(--radius-lg)] border border-border bg-white-a04 p-4"
        >
          <div className="h-4 w-40 animate-pulse rounded bg-white-a07" />
          <div className="mt-4 space-y-2">
            <div className="h-10 animate-pulse rounded bg-white-a07" />
            <div className="h-10 animate-pulse rounded bg-white-a07" />
          </div>
        </div>
      ))}
    </div>
  );
}

function DiffSection({
  title,
  description,
  copyLabel,
  copied,
  onCopy,
  children,
}: {
  title: string;
  description: string;
  copyLabel: string;
  copied: boolean;
  onCopy: () => void;
  children: ReactNode;
}) {
  return (
    <section className="rounded-[var(--radius-lg)] border border-border bg-white-a04 p-4">
      <div className="mb-3 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h3 className="text-sm font-semibold">{title}</h3>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            {description}
          </p>
        </div>

        <Button type="button" variant="outline" size="sm" onClick={onCopy}>
          {copied ? (
            <Check className="h-4 w-4" />
          ) : (
            <ClipboardCopy className="h-4 w-4" />
          )}
          {copied ? 'Copied!' : copyLabel}
        </Button>
      </div>

      <div className="overflow-hidden rounded-[var(--radius-md)] border border-border bg-background/50 font-mono text-xs leading-6">
        {children}
      </div>
    </section>
  );
}

function DiffLine({
  tone,
  children,
}: {
  tone: 'add' | 'remove';
  children: ReactNode;
}) {
  return (
    <div
      className={cn(
        'grid grid-cols-[2rem_minmax(0,1fr)] border-b border-border/70 last:border-b-0',
        tone === 'add' && 'bg-fit-excellent/8',
        tone === 'remove' && 'bg-destructive/8',
      )}
    >
      <div
        className={cn(
          'flex items-start justify-center px-2 py-2 font-semibold',
          tone === 'add' && 'text-fit-excellent',
          tone === 'remove' && 'text-destructive',
        )}
      >
        {tone === 'add' ? '+' : '-'}
      </div>
      <div className="min-w-0 px-3 py-2 text-foreground">{children}</div>
    </div>
  );
}

function ChipList({
  values,
  variant,
}: {
  values: string[];
  variant: 'green' | 'blue' | 'gray';
}) {
  return (
    <span className="inline-flex flex-wrap gap-2 align-middle">
      {values.map((value) => (
        <Badge
          key={value}
          className={cn(
            'px-2.5 py-1 text-xs font-sans',
            variant === 'green' &&
              'border-fit-excellent/25 bg-fit-excellent/10 text-fit-excellent',
            variant === 'blue' &&
              'border-primary/25 bg-primary/10 text-primary',
            variant === 'gray' &&
              'border-border bg-white-a04 text-muted-foreground',
          )}
        >
          {value}
        </Badge>
      ))}
    </span>
  );
}
