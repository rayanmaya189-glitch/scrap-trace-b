import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { ComplianceDashboard } from './ComplianceDashboard';

export function ComplianceRoutes() {
  return (
    <Routes>
      <Route index element={<ComplianceDashboard />} />
      <Route path="reports" element={<div className="p-4">Report Generator (Coming Soon)</div>} />
      <Route path="consent" element={<div className="p-4">Consent Manager (Coming Soon)</div>} />
      <Route path="audit" element={<div className="p-4">Audit Trail Viewer (Coming Soon)</div>} />
    </Routes>
  );
}
