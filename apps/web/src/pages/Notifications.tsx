import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Bell, BriefcaseBusiness, Clock3, RefreshCcw } from 'lucide-react';
import toast from 'react-hot-toast';

import {
	getNotifications,
	markNotificationRead,
} from '../api/notifications';
import type { AppNotification } from '../api/notifications';
import { Badge } from '../components/ui/Badge';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatCard } from '../components/ui/StatCard';
import { cn } from '../lib/cn';
import { formatDate, formatEnumLabel } from '../lib/format';
import { queryKeys } from '../queryKeys';

const LIST_LIMIT = 50;

function readProfileId() {
	return window.localStorage.getItem('engine_api_profile_id');
}

const NOTIFICATION_META: Record<
	AppNotification['type'],
	{
		label: string;
		badgeVariant: 'default' | 'warning' | 'success';
		iconClass: string;
	}
> = {
	new_jobs_found: {
		label: 'New matches',
		badgeVariant: 'default',
		iconClass: 'border-primary/15 bg-primary/10 text-primary',
	},
	job_reactivated: {
		label: 'Reactivated',
		badgeVariant: 'warning',
		iconClass: 'border-amber-500/20 bg-amber-500/10 text-amber-300',
	},
	application_due_soon: {
		label: 'Due soon',
		badgeVariant: 'success',
		iconClass: 'border-fit-excellent/20 bg-fit-excellent/10 text-fit-excellent',
	},
};

function formatTimestamp(value: string) {
	return formatDate(value, 'uk-UA', {
		day: 'numeric',
		month: 'short',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
	});
}

function NotificationRow({
	notification,
	onMarkRead,
	isPending,
}: {
	notification: AppNotification;
	onMarkRead: (id: string) => void;
	isPending: boolean;
}) {
	const meta = NOTIFICATION_META[notification.type];

	return (
		<Card
			className={cn(
				'border-border bg-card transition-colors',
				!notification.readAt && 'border-primary/15 bg-primary/[0.03]',
			)}
		>
			<CardContent className="flex flex-col gap-4 px-5 py-5 lg:flex-row lg:items-start lg:justify-between">
				<div className="flex min-w-0 gap-4">
					<div
						className={cn(
							'mt-0.5 flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border',
							meta.iconClass,
						)}
					>
						<Bell className="h-4 w-4" />
					</div>
					<div className="min-w-0">
						<div className="flex flex-wrap items-center gap-2">
							<p className="m-0 text-sm font-semibold text-card-foreground md:text-base">
								{notification.title}
							</p>
							<Badge
								variant={meta.badgeVariant}
								className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em]"
							>
								{meta.label}
							</Badge>
							{!notification.readAt && (
								<Badge
									variant="muted"
									className="px-2 py-0.5 text-[10px] uppercase tracking-[0.14em] text-primary"
								>
									Unread
								</Badge>
							)}
						</div>
						{notification.body && (
							<p className="m-0 mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
								{notification.body}
							</p>
						)}
						<div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
							<span className="inline-flex items-center gap-1 rounded-full border border-border bg-white/[0.04] px-2.5 py-1">
								<Clock3 className="h-3.5 w-3.5" />
								{formatTimestamp(notification.createdAt)}
							</span>
							<span className="inline-flex items-center gap-1 rounded-full border border-border bg-white/[0.04] px-2.5 py-1">
								<BriefcaseBusiness className="h-3.5 w-3.5" />
								{formatEnumLabel(notification.type)}
							</span>
						</div>
					</div>
				</div>
				<div className="flex shrink-0 items-center gap-2 lg:pl-4">
					{notification.readAt ? (
						<span className="text-xs text-muted-foreground">
							Read {formatTimestamp(notification.readAt)}
						</span>
					) : (
						<Button
							variant="ghost"
							size="sm"
							className="h-10 rounded-xl px-3 text-primary hover:text-primary"
							onClick={() => onMarkRead(notification.id)}
							disabled={isPending}
						>
							Mark read
						</Button>
					)}
				</div>
			</CardContent>
		</Card>
	);
}

export default function Notifications() {
	const queryClient = useQueryClient();
	const profileId = readProfileId();

	const {
		data: notifications = [],
		isLoading,
		error,
	} = useQuery({
		queryKey: queryKeys.notifications.list(profileId ?? 'none', LIST_LIMIT),
		queryFn: () => getNotifications(profileId ?? undefined, LIST_LIMIT),
		enabled: !!profileId,
	});

	const markReadMutation = useMutation({
		mutationFn: markNotificationRead,
		onSuccess: async () => {
			await queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all() });
		},
		onError: (value: unknown) => {
			toast.error(value instanceof Error ? value.message : 'Failed to update notification');
		},
	});

	const unreadCount = notifications.filter((notification) => !notification.readAt).length;

	return (
		<Page>
			<PageHeader
				title="Notifications"
				description="A lightweight in-app inbox for fresh matches and lifecycle changes relevant to your active profile."
				actions={
					<Button
						variant="outline"
						size="sm"
						onClick={() => {
							void queryClient.invalidateQueries({ queryKey: queryKeys.notifications.all() });
						}}
						disabled={!profileId || isLoading}
					>
						<RefreshCcw className="h-4 w-4" />
						Refresh
					</Button>
				}
			/>

			{!profileId ? (
				<EmptyState
					icon={<Bell className="h-5 w-5" />}
					message="Notifications need an active profile"
					description="Create or load a profile first so Job Copilot can scope notifications to the right candidate context."
				/>
			) : (
				<>
					<div className="grid gap-4 md:grid-cols-3">
						<StatCard
							title="Unread"
							value={String(unreadCount)}
							description="Items still waiting for acknowledgement"
						/>
						<StatCard
							title="Total"
							value={String(notifications.length)}
							description="Latest notifications in your in-app inbox"
						/>
						<StatCard
							title="Profile scope"
							value={profileId.slice(0, 8)}
							description="Notifications are stored per profile"
						/>
					</div>

					<Card className="border-border bg-card">
						<CardHeader className="gap-3">
							<CardTitle>Latest activity</CardTitle>
							<p className="m-0 text-sm leading-6 text-muted-foreground">
								Newest notifications appear first. Use the action on each unread item to clear it.
							</p>
						</CardHeader>
						<CardContent className="space-y-4">
							{isLoading ? (
								<div className="space-y-3">
									{Array.from({ length: 3 }).map((_, index) => (
										<div
											key={index}
											className="h-28 animate-pulse rounded-2xl border border-border/70 bg-white/[0.04]"
										/>
									))}
								</div>
							) : error ? (
								<EmptyState
									message="Failed to load notifications"
									description={
										error instanceof Error
											? error.message
											: 'The inbox could not be loaded.'
									}
								/>
							) : notifications.length === 0 ? (
								<EmptyState
									icon={<Bell className="h-5 w-5" />}
									message="No notifications yet"
									description="New matches and job lifecycle changes will show up here after ingestion runs."
								/>
							) : (
								notifications.map((notification) => (
									<NotificationRow
										key={notification.id}
										notification={notification}
										onMarkRead={(id) => markReadMutation.mutate(id)}
										isPending={markReadMutation.isPending}
									/>
								))
							)}
						</CardContent>
					</Card>
				</>
			)}
		</Page>
	);
}
