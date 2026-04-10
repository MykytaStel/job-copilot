import { BrowserRouter, Route, Routes } from 'react-router-dom';
import Layout from './Layout';
import Dashboard from './pages/Dashboard';
import JobIntake from './pages/JobIntake';
import JobDetails from './pages/JobDetails';
import ResumeUpload from './pages/ResumeUpload';
import ApplicationBoard from './pages/ApplicationBoard';
import ApplicationDetail from './pages/ApplicationDetail';
import Market from './pages/Market';
import Alerts from './pages/Alerts';
import Compare from './pages/Compare';
import Profile from './pages/Profile';
import CoverLetter from './pages/CoverLetter';
import InterviewQA from './pages/InterviewQA';
import Offers from './pages/Offers';
import Import from './pages/Import';
import Backup from './pages/Backup';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="jobs/new" element={<JobIntake />} />
          <Route path="jobs/:id" element={<JobDetails />} />
          <Route path="resume" element={<ResumeUpload />} />
          <Route path="applications" element={<ApplicationBoard />} />
          <Route path="applications/:id" element={<ApplicationDetail />} />
          <Route path="market" element={<Market />} />
          <Route path="alerts" element={<Alerts />} />
          <Route path="compare" element={<Compare />} />
          <Route path="profile" element={<Profile />} />
          {/* P3 — AI scaffolding */}
          <Route path="cover-letters" element={<CoverLetter />} />
          <Route path="interview-qa" element={<InterviewQA />} />
          <Route path="offers" element={<Offers />} />
          {/* P4 — Integrations */}
          <Route path="import" element={<Import />} />
          <Route path="backup" element={<Backup />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
