import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import axios from 'axios';

// Types
export interface User {
  id: string;
  phone: string;
  role: 'dealer' | 'buyer' | 'exporter' | 'nbfc' | 'auditor' | null;
  name?: string;
  businessName?: string;
  isVerified: boolean;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
  tokenType: string;
}

interface AuthState {
  user: User | null;
  tokens: AuthTokens | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  login: (phone: string, otp: string) => Promise<void>;
  requestOtp: (phone: string) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
  clearError: () => void;
  updateUser: (user: Partial<User>) => void;
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

// Configure axios defaults
const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add auth token
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('btrace_access_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor to handle token refresh
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;
    
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;
      
      try {
        const refreshToken = localStorage.getItem('btrace_refresh_token');
        if (!refreshToken) {
          throw new Error('No refresh token');
        }

        const response = await axios.post(`${API_BASE_URL}/v1/auth/refresh`, {
          refresh_token: refreshToken,
        });

        const { access_token } = response.data.data;
        localStorage.setItem('btrace_access_token', access_token);
        
        originalRequest.headers.Authorization = `Bearer ${access_token}`;
        return apiClient(originalRequest);
      } catch (refreshError) {
        // Refresh failed, logout user
        useAuthStore.getState().logout();
        return Promise.reject(refreshError);
      }
    }
    
    return Promise.reject(error);
  }
);

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      tokens: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      requestOtp: async (phone: string) => {
        set({ isLoading: true, error: null });
        try {
          await apiClient.post('/v1/auth/request-otp', { phone });
          set({ isLoading: false });
        } catch (error: any) {
          set({ 
            isLoading: false, 
            error: error.response?.data?.message || 'Failed to send OTP' 
          });
          throw error;
        }
      },

      login: async (phone: string, otp: string) => {
        set({ isLoading: true, error: null });
        try {
          const response = await apiClient.post('/v1/auth/verify-otp', { 
            phone, 
            otp 
          });
          
          const { user, tokens } = response.data.data;
          
          localStorage.setItem('btrace_access_token', tokens.access_token);
          localStorage.setItem('btrace_refresh_token', tokens.refresh_token);
          
          set({
            user,
            tokens,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (error: any) {
          set({ 
            isLoading: false, 
            error: error.response?.data?.message || 'Login failed' 
          });
          throw error;
        }
      },

      logout: () => {
        localStorage.removeItem('btrace_access_token');
        localStorage.removeItem('btrace_refresh_token');
        set({
          user: null,
          tokens: null,
          isAuthenticated: false,
          error: null,
        });
      },

      refreshToken: async () => {
        try {
          const refreshToken = localStorage.getItem('btrace_refresh_token');
          if (!refreshToken) {
            throw new Error('No refresh token available');
          }

          const response = await apiClient.post('/v1/auth/refresh', {
            refresh_token: refreshToken,
          });

          const { access_token, refresh_token } = response.data.data;
          localStorage.setItem('btrace_access_token', access_token);
          localStorage.setItem('btrace_refresh_token', refresh_token);
          
          set((state) => ({
            tokens: state.tokens ? {
              ...state.tokens,
              accessToken: access_token,
              refreshToken: refresh_token,
            } : null,
          }));
        } catch (error) {
          console.error('Token refresh failed:', error);
          get().logout();
        }
      },

      clearError: () => set({ error: null }),

      updateUser: (userData: Partial<User>) => {
        set((state) => ({
          user: state.user ? { ...state.user, ...userData } : null,
        }));
      },
    }),
    {
      name: 'btrace-auth-storage',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
      onRehydrateStorage: () => (state) => {
        // Restore tokens from localStorage on rehydration
        if (state?.isAuthenticated) {
          const accessToken = localStorage.getItem('btrace_access_token');
          const refreshToken = localStorage.getItem('btrace_refresh_token');
          if (accessToken && refreshToken) {
            state.tokens = {
              accessToken,
              refreshToken,
              expiresIn: 86400,
              tokenType: 'Bearer',
            };
          }
        }
      },
    }
  )
);

export { apiClient };
