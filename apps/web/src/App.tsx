import { lazy, Suspense, type ReactNode } from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';

import AppShell from './AppShell';
import { withErrorBoundary } from './components/ErrorBoundary';
import { ToastProvider } from './context/ToastContext';

const Auth = lazy(() => import('./pages/Auth'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const JobDetails = lazy(() => import('./pages/JobDetails'));
const ApplicationBoard = lazy(() => import('./pages/ApplicationBoard'));
const ApplicationDetail = lazy(() => import('./pages/ApplicationDetail'));
const Profile = lazy(() => import('./pages/Profile'));
const FeedbackCenter = lazy(() => import('./pages/FeedbackCenter'));
const Analytics = lazy(() => import('./pages/Analytics'));
const Market = lazy(() => import('./pages/Market'));
const CompanyDetail = lazy(() => import('./pages/CompanyDetail'));
const Notifications = lazy(() => import('./pages/Notifications'));
const Settings = lazy(() => import('./pages/Settings'));
const Setup = lazy(() => import('./pages/Setup'));

function route(element: ReactNode) {
  return withErrorBoundary(<Suspense fallback={null}>{element}</Suspense>);
}

export default function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/auth" element={route(<Auth />)} />
          <Route path="/setup" element={route(<Setup />)} />

          <Route path="/" element={<AppShell />}>
            <Route index element={route(<Dashboard />)} />
            <Route path="jobs/:id" element={route(<JobDetails />)} />
            <Route path="applications" element={route(<ApplicationBoard />)} />
            <Route path="applications/:id" element={route(<ApplicationDetail />)} />
            <Route path="profile" element={route(<Profile />)} />
            <Route path="feedback" element={route(<FeedbackCenter />)} />
            <Route path="analytics" element={route(<Analytics />)} />
            <Route path="market" element={route(<Market />)} />
            <Route path="market/companies/:slug" element={route(<CompanyDetail />)} />
            <Route path="notifications" element={route(<Notifications />)} />
            <Route path="settings" element={route(<Settings />)} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ToastProvider>
  );
}
