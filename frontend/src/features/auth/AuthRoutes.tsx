import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { LoginPage } from './LoginPage';
import { useAuth } from './AuthProvider';

export const AuthRoutes: React.FC = () => {
  const { isAuthenticated } = useAuth();

  if (isAuthenticated) {
    return <Navigate to="/" replace />;
  }

  return (
    <Routes>
      <Route path="login" element={<LoginPage />} />
      <Route path="*" element={<Navigate to="/auth/login" replace />} />
    </Routes>
  );
};
