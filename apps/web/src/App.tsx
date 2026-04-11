import { BrowserRouter, Route, Routes } from 'react-router-dom';

import Layout from './Layout';
import Dashboard from './pages/Dashboard';
import JobDetails from './pages/JobDetails';
import ApplicationBoard from './pages/ApplicationBoard';
import Profile from './pages/Profile';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="jobs/:id" element={<JobDetails />} />
          <Route path="applications" element={<ApplicationBoard />} />
          <Route path="profile" element={<Profile />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
