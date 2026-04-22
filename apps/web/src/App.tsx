import { lazy } from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';

import AppShell from './AppShell';

const Dashboard = lazy(() => import('./pages/Dashboard'));
const JobDetails = lazy(() => import('./pages/JobDetails'));
const ApplicationBoard = lazy(() => import('./pages/ApplicationBoard'));
const ApplicationDetail = lazy(() => import('./pages/ApplicationDetail'));
const Profile = lazy(() => import('./pages/Profile'));
const FeedbackCenter = lazy(() => import('./pages/FeedbackCenter'));
const Analytics = lazy(() => import('./pages/Analytics'));
const Market = lazy(() => import('./pages/Market'));
const Notifications = lazy(() => import('./pages/Notifications'));
const Settings = lazy(() => import('./pages/Settings'));

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Dashboard />} />
          <Route path="jobs/:id" element={<JobDetails />} />
          <Route path="applications" element={<ApplicationBoard />} />
          <Route path="applications/:id" element={<ApplicationDetail />} />
          <Route path="profile" element={<Profile />} />
          <Route path="feedback" element={<FeedbackCenter />} />
          <Route path="analytics" element={<Analytics />} />
          <Route path="market" element={<Market />} />
          <Route path="notifications" element={<Notifications />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
