import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from './features/auth/AuthProvider';
import { ProtectedRoute } from './features/auth/ProtectedRoute';
import { AuthRoutes } from './features/auth/AuthRoutes';
import { DashboardLayout } from './features/dashboard/DashboardLayout';
import { DashboardPage } from './features/dashboard/DashboardPage';
import { MaterialRoutes } from './features/materials/MaterialRoutes';
import { HandshakeRoutes } from './features/handshakes/HandshakeRoutes';
import { ScoreRoutes } from './features/scores/ScoreRoutes';
import { ComplianceRoutes } from './features/compliance/ComplianceRoutes';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Router>
          <Routes>
            {/* Public routes */}
            <Route path="/auth/*" element={<AuthRoutes />} />
            
            {/* Protected routes with dashboard layout */}
            <Route
              path="/"
              element={
                <ProtectedRoute>
                  <DashboardLayout />
                </ProtectedRoute>
              }
            >
              <Route index element={<DashboardPage />} />
              <Route path="materials/*" element={<MaterialRoutes />} />
              <Route path="handshakes/*" element={<HandshakeRoutes />} />
              <Route path="scores/*" element={<ScoreRoutes />} />
              <Route path="compliance/*" element={<ComplianceRoutes />} />
            </Route>

            {/* Catch all - redirect to dashboard */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </Router>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
