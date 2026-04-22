import { Activity, ListTodo } from 'lucide-react';
import type { ApplicationDetail } from '@job-copilot/shared';

import { Badge } from '../../components/ui/Badge';
import { EmptyState } from '../../components/ui/EmptyState';
import { formatDate, formatEnumLabel } from '../../lib/format';
import { Panel } from './ApplicationDetailLayout';

export function ActivitiesSection({ activities }: { activities: ApplicationDetail['activities'] }) {
  return (
    <Panel
      title="Activities"
      description="Timeline of synced events and manual updates for this application."
      icon={Activity}
    >
      {activities.length === 0 ? (
        <EmptyState message="No activities yet" />
      ) : (
        <div className="space-y-3">
          {activities.map((activity) => (
            <div
              key={activity.id}
              className="flex items-start gap-3 rounded-2xl border border-border/70 bg-surface-muted px-4 py-4"
            >
              <Badge
                variant="muted"
                className="mt-0.5 px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
              >
                {formatEnumLabel(activity.type)}
              </Badge>
              <div className="min-w-0">
                <p className="m-0 text-sm leading-6 text-card-foreground">{activity.description}</p>
                <p className="m-0 mt-2 text-xs text-muted-foreground">
                  {formatDate(activity.happenedAt)}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}

export function TasksSection({ tasks }: { tasks: ApplicationDetail['tasks'] }) {
  return (
    <Panel
      title="Tasks"
      description="Outstanding follow-ups and reminders attached to this application."
      icon={ListTodo}
    >
      {tasks.length === 0 ? (
        <EmptyState message="No tasks yet" />
      ) : (
        <div className="space-y-3">
          {tasks.map((task) => (
            <div
              key={task.id}
              className="flex items-start gap-3 rounded-2xl border border-border/70 bg-surface-muted px-4 py-4"
            >
              <input type="checkbox" checked={task.done} readOnly className="mt-1 h-4 w-4" />
              <div className="min-w-0">
                <p
                  className={`m-0 text-sm leading-6 ${
                    task.done ? 'text-muted-foreground line-through' : 'text-card-foreground'
                  }`}
                >
                  {task.title}
                </p>
                {task.remindAt ? (
                  <p className="m-0 mt-2 text-xs text-muted-foreground">
                    Remind: {formatDate(task.remindAt)}
                  </p>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}
