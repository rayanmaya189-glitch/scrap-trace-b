import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { ScoreDashboard } from './ScoreDashboard';

export function ScoreRoutes() {
  return (
    <Routes>
      <Route path="/" element={<ScoreDashboard />} />
      <Route path="/dashboard" element={<ScoreDashboard />} />
    </Routes>
  );
}
