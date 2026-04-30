import type { LucideIcon } from 'lucide-react';
import {
  BarChart3,
  KanbanSquare,
  LayoutDashboard,
  LineChart,
  MessageSquare,
  Settings,
  User,
  Users,
} from 'lucide-react';

export type NavItem = {
  name: string;
  to: string;
  end: boolean;
  icon: LucideIcon;
};

export const navigation: NavItem[] = [
  { name: 'Dashboard', to: '/', end: true, icon: LayoutDashboard },
  { name: 'Applications', to: '/applications', end: false, icon: KanbanSquare },
  { name: 'Contacts', to: '/contacts', end: false, icon: Users },
  { name: 'Feedback', to: '/feedback', end: false, icon: MessageSquare },
  { name: 'Analytics', to: '/analytics', end: false, icon: BarChart3 },
  { name: 'Market', to: '/market', end: false, icon: LineChart },
  { name: 'Profile', to: '/profile', end: false, icon: User },
  { name: 'Settings', to: '/settings', end: false, icon: Settings },
];
