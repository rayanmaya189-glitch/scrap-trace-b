import React from 'react';
import { Routes, Route } from 'react-router-dom';

export function ComplianceRoutes() {
  return (
    <Routes>
      <Route index element={<div className="p-4">Compliance Dashboard (Coming Soon)</div>} />
    </Routes>
  );
}
