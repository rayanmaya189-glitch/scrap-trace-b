import { Routes, Route } from 'react-router-dom';
import { ProfileSettings } from './ProfileSettings';

export function ProfileRoutes() {
  return (
    <Routes>
      <Route path="/" element={<ProfileSettings />} />
      <Route path="/settings" element={<ProfileSettings />} />
    </Routes>
  );
}
