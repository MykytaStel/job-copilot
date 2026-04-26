import { lazy, Suspense } from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';

import AppShell from './AppShell';
import { withErrorBoundary } from './components/ErrorBoundary';

const Auth = lazy(() => import('./pages/Auth'));
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
const Setup = lazy(() => import('./pages/Setup'));

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={null}>
        <Routes>
          <Route path="/auth" element={withErrorBoundary(<Auth />)} />

          <Route element={<AppShell />}>
            <Route index element={withErrorBoundary(<Dashboard />)} />
            <Route path="/jobs/:id" element={withErrorBoundary(<JobDetails />)} />
            <Route path="/applications" element={withErrorBoundary(<ApplicationBoard />)} />
            <Route path="/applications/:id" element={withErrorBoundary(<ApplicationDetail />)} />
            <Route path="/profile" element={withErrorBoundary(<Profile />)} />
            <Route path="/feedback" element={withErrorBoundary(<FeedbackCenter />)} />
            <Route path="/analytics" element={withErrorBoundary(<Analytics />)} />
            <Route path="/market" element={withErrorBoundary(<Market />)} />
            <Route path="/notifications" element={withErrorBoundary(<Notifications />)} />
            <Route path="/settings" element={withErrorBoundary(<Settings />)} />
            <Route path="/setup" element={withErrorBoundary(<Setup />)} />
          </Route>
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}