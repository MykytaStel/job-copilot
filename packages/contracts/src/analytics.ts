import type { ApplicationStatus } from './applications';

export interface DashboardStats {
  total: number;
  byStatus: Record<ApplicationStatus, number>;
  topMissingSkills: Array<{ skill: string; count: number }>;
  avgScore: number | null;
  tasksDueSoon: number;
}
