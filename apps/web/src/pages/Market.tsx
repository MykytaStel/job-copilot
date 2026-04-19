import type { ReactNode } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
	ArrowDownRight,
	ArrowUpRight,
	BriefcaseBusiness,
	Building2,
	CircleDollarSign,
	Minus,
	RadioTower,
	TrendingUp,
	Wifi,
	type LucideIcon,
} from 'lucide-react';

import {
	getMarketCompanies,
	getMarketOverview,
	getMarketRoles,
	getMarketSalaries,
	type MarketCompany,
	type MarketRoleDemand,
	type MarketSalaryTrend,
	type MarketTrend,
} from '../api/market';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { StatCard } from '../components/ui/StatCard';
import { cn } from '../lib/cn';
import { queryKeys } from '../queryKeys';

const numberFormatter = new Intl.NumberFormat('en-US');
const percentFormatter = new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 });

function formatCount(value: number) {
	return numberFormatter.format(value);
}

function formatPercent(value: number) {
	return `${percentFormatter.format(value)}%`;
}

function formatSalary(value: number) {
	return numberFormatter.format(Math.round(value));
}

function titleCase(value: string) {
	return value.charAt(0).toUpperCase() + value.slice(1);
}

function getTrendMeta(trend: MarketTrend | number) {
	if (typeof trend === 'number') {
		if (trend > 0) {
			return {
				icon: ArrowUpRight,
				label: `+${trend}`,
				className: 'border-fit-excellent/25 bg-fit-excellent/10 text-fit-excellent',
			};
		}

		if (trend < 0) {
			return {
				icon: ArrowDownRight,
				label: `${trend}`,
				className: 'border-destructive/25 bg-destructive/10 text-destructive',
			};
		}

		return {
			icon: Minus,
			label: '0',
			className: 'border-border bg-white/[0.04] text-muted-foreground',
		};
	}

	if (trend === 'up') {
		return {
			icon: ArrowUpRight,
			label: 'Up',
			className: 'border-fit-excellent/25 bg-fit-excellent/10 text-fit-excellent',
		};
	}

	if (trend === 'down') {
		return {
			icon: ArrowDownRight,
			label: 'Down',
			className: 'border-destructive/25 bg-destructive/10 text-destructive',
		};
	}

	return {
		icon: Minus,
		label: 'Flat',
		className: 'border-border bg-white/[0.04] text-muted-foreground',
	};
}

function deriveMedianSalary(trends: MarketSalaryTrend[]) {
	if (trends.length === 0) {
		return null;
	}

	const ordered = [...trends].sort((left, right) => left.median - right.median);
	const totalWeight = ordered.reduce((sum, item) => sum + item.sampleCount, 0);

	if (totalWeight <= 0) {
		return ordered[Math.floor(ordered.length / 2)]?.median ?? null;
	}

	let cumulativeWeight = 0;
	for (const item of ordered) {
		cumulativeWeight += item.sampleCount;
		if (cumulativeWeight >= totalWeight / 2) {
			return item.median;
		}
	}

	return ordered.at(-1)?.median ?? null;
}

function MarketSection({
	title,
	description,
	icon: Icon,
	children,
}: {
	title: string;
	description: string;
	icon: LucideIcon;
	children: ReactNode;
}) {
	return (
		<Card className="border-border bg-card">
			<CardHeader className="gap-3">
				<div className="flex items-start gap-3">
					<div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
						<Icon className="h-5 w-5" />
					</div>
					<div>
						<CardTitle className="text-base font-semibold">{title}</CardTitle>
						<p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
					</div>
				</div>
			</CardHeader>
			<CardContent>{children}</CardContent>
		</Card>
	);
}

function StatCardSkeleton() {
	return (
		<div className="h-[140px] animate-pulse rounded-[24px] border border-border bg-card/80" />
	);
}

function ListSkeleton({ rows = 5 }: { rows?: number }) {
	return (
		<div className="space-y-3">
			{Array.from({ length: rows }).map((_, index) => (
				<div
					key={index}
					className="h-16 animate-pulse rounded-2xl border border-border/70 bg-white/[0.04]"
				/>
			))}
		</div>
	);
}

function TrendBadge({ trend }: { trend: MarketTrend | number }) {
	const meta = getTrendMeta(trend);
	const Icon = meta.icon;

	return (
		<span
			className={cn(
				'inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-xs font-medium',
				meta.className,
			)}
		>
			<Icon className="h-3.5 w-3.5" />
			{meta.label}
		</span>
	);
}

function CompanyRow({ company }: { company: MarketCompany }) {
	return (
		<div className="grid gap-3 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4 lg:grid-cols-[minmax(0,1.3fr)_120px_180px] lg:items-center">
			<div className="min-w-0">
				<p className="m-0 truncate text-sm font-semibold text-card-foreground">
					{company.companyName}
				</p>
				<p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
					{formatCount(company.thisWeek)} new this week
					{' • '}
					{formatCount(company.prevWeek)} previous week
				</p>
			</div>
			<div className="flex items-center justify-between gap-3 lg:block lg:text-right">
				<span className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
					Active jobs
				</span>
				<p className="m-0 mt-1 text-lg font-semibold text-card-foreground">
					{formatCount(company.activeJobs)}
				</p>
			</div>
			<div className="flex items-center justify-between gap-3 lg:justify-end">
				<span className="text-xs text-muted-foreground">Velocity</span>
				<TrendBadge trend={company.velocity} />
			</div>
		</div>
	);
}

function SalaryRow({
	salary,
	minValue,
	maxValue,
}: {
	salary: MarketSalaryTrend;
	minValue: number;
	maxValue: number;
}) {
	const domain = Math.max(maxValue - minValue, 1);
	const rangeStart = ((salary.p25 - minValue) / domain) * 100;
	const rangeWidth = ((salary.p75 - salary.p25) / domain) * 100;
	const medianPosition = ((salary.median - minValue) / domain) * 100;

	return (
		<div className="rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4">
			<div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
				<div>
					<p className="m-0 text-sm font-semibold text-card-foreground">
						{titleCase(salary.seniority)}
					</p>
					<p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
						Based on {formatCount(salary.sampleCount)} active postings with salary data
					</p>
				</div>
				<div className="text-left sm:text-right">
					<p className="m-0 text-sm font-semibold text-card-foreground">
						Median {formatSalary(salary.median)}
					</p>
					<p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
						{formatSalary(salary.p25)} - {formatSalary(salary.p75)} p25-p75 range
					</p>
				</div>
			</div>
			<div className="mt-4">
				<div className="relative h-3 rounded-full bg-white/[0.05]">
					<div
						className="absolute top-0 h-full rounded-full bg-primary/25"
						style={{ left: `${rangeStart}%`, width: `${Math.max(rangeWidth, 3)}%` }}
					/>
					<div
						className="absolute top-1/2 h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-background bg-primary shadow-[0_0_0_4px_rgba(90,132,255,0.18)]"
						style={{ left: `${medianPosition}%` }}
					/>
				</div>
				<div className="mt-2 flex items-center justify-between text-[11px] uppercase tracking-[0.12em] text-muted-foreground">
					<span>{formatSalary(minValue)}</span>
					<span>{formatSalary(maxValue)}</span>
				</div>
			</div>
		</div>
	);
}

function RoleRow({ role }: { role: MarketRoleDemand }) {
	const delta = role.thisPeriod - role.prevPeriod;
	const deltaLabel = delta > 0 ? `+${delta}` : `${delta}`;

	return (
		<div className="flex items-center justify-between gap-4 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-4">
			<div className="min-w-0">
				<p className="m-0 text-sm font-semibold text-card-foreground">{role.roleGroup}</p>
				<p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
					{formatCount(role.thisPeriod)} this period
					{' • '}
					{formatCount(role.prevPeriod)} previous period
				</p>
			</div>
			<div className="text-right">
				<TrendBadge trend={role.trend} />
				<p className="m-0 mt-2 text-xs text-muted-foreground">{deltaLabel} net change</p>
			</div>
		</div>
	);
}

export default function Market() {
	const overviewQuery = useQuery({
		queryKey: queryKeys.market.overview(),
		queryFn: getMarketOverview,
		staleTime: 5 * 60_000,
	});
	const companiesQuery = useQuery({
		queryKey: queryKeys.market.companies(),
		queryFn: () => getMarketCompanies(12),
		staleTime: 5 * 60_000,
	});
	const salariesQuery = useQuery({
		queryKey: queryKeys.market.salaries(),
		queryFn: () => getMarketSalaries(),
		staleTime: 5 * 60_000,
	});
	const rolesQuery = useQuery({
		queryKey: queryKeys.market.roles(),
		queryFn: () => getMarketRoles(30),
		staleTime: 5 * 60_000,
	});

	const salaryTrends = salariesQuery.data ?? [];
	const roleDemand =
		rolesQuery.data?.filter((role) => role.thisPeriod > 0 || role.prevPeriod > 0) ?? [];
	const marketMedian = deriveMedianSalary(salaryTrends);
	const salarySampleCount = salaryTrends.reduce((sum, item) => sum + item.sampleCount, 0);
	const salaryMin = salaryTrends.length > 0 ? Math.min(...salaryTrends.map((item) => item.p25)) : 0;
	const salaryMax = salaryTrends.length > 0 ? Math.max(...salaryTrends.map((item) => item.p75)) : 0;

	return (
		<Page>
			<PageHeader
				title="Market Intelligence"
				description="Live aggregates from the current job feed: hiring activity, salary ranges, and role demand without leaving the operator dashboard."
				breadcrumb={[
					{ label: 'Dashboard', href: '/' },
					{ label: 'Market' },
				]}
			/>

			<div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
				{overviewQuery.isPending ? (
					<>
						<StatCardSkeleton />
						<StatCardSkeleton />
						<StatCardSkeleton />
					</>
				) : (
					<>
						<StatCard
							title="New jobs this week"
							value={
								overviewQuery.data
									? formatCount(overviewQuery.data.newJobsThisWeek)
									: '—'
							}
							description={
								overviewQuery.data
									? `${formatCount(overviewQuery.data.activeJobsCount)} active jobs tracked right now`
									: 'Overview data unavailable'
							}
							icon={BriefcaseBusiness}
						/>
						<StatCard
							title="Active companies"
							value={
								overviewQuery.data
									? formatCount(overviewQuery.data.activeCompaniesCount)
									: '—'
							}
							description="Companies with at least one active posting"
							icon={Building2}
						/>
						<StatCard
							title="Remote share"
							value={
								overviewQuery.data
									? formatPercent(overviewQuery.data.remotePercentage)
									: '—'
							}
							description="Share of active jobs explicitly marked remote"
							icon={Wifi}
						/>
					</>
				)}

				{salariesQuery.isPending ? (
					<StatCardSkeleton />
				) : (
					<StatCard
						title="Median salary"
						value={marketMedian !== null ? formatSalary(marketMedian) : '—'}
						description={
							salarySampleCount > 0
								? `Derived from ${formatCount(salarySampleCount)} salary-tagged postings`
								: 'No recent salary reports across tracked seniority bands'
						}
						icon={CircleDollarSign}
					/>
				)}
			</div>

			<MarketSection
				title="Top Hiring Companies"
				description="Companies with the largest active footprint in the current feed, plus week-over-week hiring velocity."
				icon={TrendingUp}
			>
				{companiesQuery.isPending ? (
					<ListSkeleton rows={6} />
				) : companiesQuery.isError ? (
					<EmptyState
						message="Unable to load company activity."
						description="The market companies endpoint did not return a usable response."
					/>
				) : companiesQuery.data && companiesQuery.data.length > 0 ? (
					<div className="space-y-3">
						{companiesQuery.data.map((company) => (
							<CompanyRow key={company.companyName} company={company} />
						))}
					</div>
				) : (
					<EmptyState
						message="No active hiring companies yet."
						description="This section fills once the feed has active postings with company attribution."
					/>
				)}
			</MarketSection>

			<div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)]">
				<MarketSection
					title="Salary by Seniority"
					description="P25-p75 ranges with the median marker for the recent active salary sample in each seniority bucket."
					icon={CircleDollarSign}
				>
					{salariesQuery.isPending ? (
						<ListSkeleton rows={4} />
					) : salariesQuery.isError ? (
						<EmptyState
							message="Unable to load salary ranges."
							description="The salary analytics endpoint returned an error."
						/>
					) : salaryTrends.length > 0 ? (
						<div className="space-y-3">
							{salaryTrends.map((salary) => (
								<SalaryRow
									key={salary.seniority}
									salary={salary}
									minValue={salaryMin}
									maxValue={salaryMax}
								/>
							))}
						</div>
					) : (
						<EmptyState
							message="No recent salary distribution data."
							description="Salary ranges appear here when active postings include structured compensation."
						/>
					)}
				</MarketSection>

				<MarketSection
					title="Role Demand"
					description="Current-period volume compared with the previous matching window, grouped by major role families."
					icon={RadioTower}
				>
					{rolesQuery.isPending ? (
						<ListSkeleton rows={6} />
					) : rolesQuery.isError ? (
						<EmptyState
							message="Unable to load role demand."
							description="The role demand endpoint did not return trend data."
						/>
					) : roleDemand.length > 0 ? (
						<div className="space-y-3">
							{roleDemand
								.slice()
								.sort(
									(left, right) =>
										right.thisPeriod - left.thisPeriod ||
										right.prevPeriod - left.prevPeriod,
								)
								.map((role) => (
									<RoleRow key={role.roleGroup} role={role} />
								))}
						</div>
					) : (
						<EmptyState
							message="No role demand signals yet."
							description="Role demand needs enough classified active jobs in the selected period."
						/>
					)}
				</MarketSection>
			</div>
		</Page>
	);
}
