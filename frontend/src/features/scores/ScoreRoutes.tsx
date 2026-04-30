import React from 'react';
import { Routes, Route } from 'react-router-dom';

export function ScoreRoutes() {
  return (
    <Routes>
      <Route index element={<div className="p-4">Score Dashboard (Coming Soon)</div>} />
    </Routes>
  );
}
