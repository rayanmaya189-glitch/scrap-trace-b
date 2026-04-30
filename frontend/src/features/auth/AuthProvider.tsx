import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { useAuthStore } from '@/stores/useAuthStore';
import { useSyncStore } from '@/stores/useSyncStore';

interface AuthContextType {
  user: ReturnType<typeof useAuthStore>['user'];
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (phone: string, otp: string) => Promise<void>;
  requestOtp: (phone: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const { user, tokens, isAuthenticated, isLoading, login, requestOtp, logout } = useAuthStore();
  const { checkConnectivity } = useSyncStore();

  useEffect(() => {
    // Check connectivity on mount
    checkConnectivity();
    
    // Set up periodic connectivity checks
    const interval = setInterval(checkConnectivity, 5000);
    
    return () => clearInterval(interval);
  }, [checkConnectivity]);

  const value = {
    user,
    token: tokens?.accessToken || null,
    isAuthenticated,
    isLoading,
    login,
    requestOtp,
    logout
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
