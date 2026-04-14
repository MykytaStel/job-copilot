import { BrowserRouter, Route, Routes } from 'react-router-dom';

import Layout from './Layout';
import Dashboard from './pages/Dashboard';
import JobDetails from './pages/JobDetails';
import ApplicationBoard from './pages/ApplicationBoard';
import ApplicationDetail from './pages/ApplicationDetail';
import Profile from './pages/Profile';
import FeedbackCenter from './pages/FeedbackCenter';
import Analytics from './pages/Analytics';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="jobs/:id" element={<JobDetails />} />
          <Route path="applications" element={<ApplicationBoard />} />
          <Route path="applications/:id" element={<ApplicationDetail />} />
          <Route path="profile" element={<Profile />} />
          <Route path="feedback" element={<FeedbackCenter />} />
          <Route path="analytics" element={<Analytics />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
