import type { ApplicationDetail } from '@job-copilot/shared';
import { NotebookPen } from 'lucide-react';

import { Button } from '../../components/ui/Button';
import { EmptyState } from '../../components/ui/EmptyState';
import { formatDate } from '../../lib/format';
import { Panel } from './ApplicationDetailLayout';

export function NotesSection({
  notes,
  noteContent,
  isPending,
  setNoteContent,
  onSubmit,
}: {
  notes: ApplicationDetail['notes'];
  noteContent: string;
  isPending: boolean;
  setNoteContent: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <Panel
      title="Notes"
      description="Capture recruiter context, interview takeaways, and decision rationale."
      icon={NotebookPen}
    >
      <form
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
      >
        <textarea
          value={noteContent}
          onChange={(event) => setNoteContent(event.target.value)}
          rows={4}
          placeholder="Add context from recruiter calls, takeaways, or follow-up reminders."
        />
        <div className="flex justify-end">
          <Button type="submit" disabled={isPending || !noteContent.trim()}>
            {isPending ? 'Saving...' : 'Add note'}
          </Button>
        </div>
      </form>

      {notes.length === 0 ? (
        <EmptyState message="No notes yet" />
      ) : (
        <div className="space-y-3">
          {notes.map((note) => (
            <div
              key={note.id}
              className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4"
            >
              <p className="m-0 text-sm leading-7 text-card-foreground">{note.content}</p>
              <p className="m-0 mt-3 text-xs text-muted-foreground">{formatDate(note.createdAt)}</p>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}
