import { apiClient } from './client';

export interface OtpRequest {
  phone: string;
}

export interface OtpVerifyRequest {
  phone: string;
  otp: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: {
    id: string;
    phone: string;
    role: string;
  };
}

export const authApi = {
  /**
   * Request OTP for login
   */
  requestOtp: async (phone: string): Promise<{ message: string }> => {
    const response = await apiClient.post('/auth/request-otp', { phone });
    return response.data.data;
  },

  /**
   * Verify OTP and get tokens
   */
  verifyOtp: async (phone: string, otp: string): Promise<AuthResponse> => {
    const response = await apiClient.post<ApiResponse<AuthResponse>>('/auth/verify-otp', {
      phone,
      otp,
    });
    return response.data.data;
  },

  /**
   * Refresh access token
   */
  refreshToken: async (refreshToken: string): Promise<AuthResponse> => {
    const response = await apiClient.post<ApiResponse<AuthResponse>>('/auth/refresh', {
      refresh_token: refreshToken,
    });
    return response.data.data;
  },

  /**
   * Logout and blacklist token
   */
  logout: async (): Promise<{ message: string }> => {
    const response = await apiClient.post<ApiResponse<{ message: string }>>('/auth/logout');
    return response.data.data;
  },
};

interface ApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
  timestamp: string;
}
