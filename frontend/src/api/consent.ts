import { apiClient } from './client';

export interface ConsentRecord {
  id: string;
  supplier_id: string;
  purpose: string;
  granted: boolean;
  revoked_at: string | null;
  created_at: string;
}

export interface CreateConsentRequest {
  supplier_id: string;
  purpose: string;
  granted: boolean;
}

export interface ConsentApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
  timestamp: string;
}

export const consentApi = {
  /**
   * Get current user's consent records
   */
  getMyConsents: async (): Promise<ConsentRecord[]> => {
    const response = await apiClient.get<ConsentApiResponse<ConsentRecord[]>>('/consent/my');
    return response.data.data;
  },

  /**
   * Get consent records for a specific supplier
   */
  getSupplierConsents: async (supplierId: string): Promise<ConsentRecord[]> => {
    const response = await apiClient.get<ConsentApiResponse<ConsentRecord[]>>(
      `/consent/${supplierId}`
    );
    return response.data.data;
  },

  /**
   * Create a new consent record
   */
  createConsent: async (request: CreateConsentRequest): Promise<ConsentRecord> => {
    const response = await apiClient.post<ConsentApiResponse<ConsentRecord>>('/consent', request);
    return response.data.data;
  },

  /**
   * Revoke a consent record
   */
  revokeConsent: async (consentId: string): Promise<ConsentRecord> => {
    const response = await apiClient.post<ConsentApiResponse<ConsentRecord>>(
      `/consent/${consentId}/revoke`
    );
    return response.data.data;
  },
};
