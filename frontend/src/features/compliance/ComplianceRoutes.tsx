import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { ComplianceDashboard } from './ComplianceDashboard';

export function ComplianceRoutes() {
  return (
    <Routes>
      <Route index element={<ComplianceDashboard />} />
      <Route path="reports" element={<ComplianceDashboard defaultTab="reports" />} />
      <Route path="consent" element={<ComplianceDashboard defaultTab="consent" />} />
      <Route path="audit" element={<ComplianceDashboard defaultTab="audit" />} />
    </Routes>
  );
}
