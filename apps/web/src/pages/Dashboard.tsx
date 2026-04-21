import { useDashboardPage } from '../features/dashboard/useDashboardPage';

import { DashboardContent } from './dashboard/DashboardContent';

export default function Dashboard() {
  const state = useDashboardPage();

  return <DashboardContent state={state} />;
}
