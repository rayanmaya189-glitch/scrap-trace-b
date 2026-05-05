import { apiClient } from './client';

export interface DigitalHandshake {
  id: string;
  material_id: string;
  from_party: string;
  to_party: string;
  payload_hash: string;
  hash_prev: string;
  hash_current: string;
  signature_from: string;
  signature_to: string;
  public_key_from: string;
  public_key_to: string;
  timestamp_utc: string;
  status: 'pending' | 'confirmed' | 'disputed' | 'rejected';
  dispute_reason?: string;
  created_at: string;
}

export interface InitiateHandshakeRequest {
  material_id: string;
  to_party: string;
  data: string;
}

export interface ConfirmHandshakeRequest {
  handshake_id: string;
  accept: boolean;
  signature: string;
  public_key: string;
}

export interface RaiseDisputeRequest {
  handshake_id: string;
  reason: string;
  evidence?: string;
}

export interface HandshakeApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
  timestamp: string;
}

export const handshakeApi = {
  /**
   * Get all handshakes for current user
   */
  getHandshakes: async (): Promise<DigitalHandshake[]> => {
    const response = await apiClient.get<HandshakeApiResponse<DigitalHandshake[]>>('/handshakes');
    return response.data.data;
  },

  /**
   * Get handshakes for a specific material
   */
  getMaterialHandshakes: async (materialId: string): Promise<DigitalHandshake[]> => {
    const response = await apiClient.get<HandshakeApiResponse<DigitalHandshake[]>>(
      `/handshakes/material/${materialId}`
    );
    return response.data.data;
  },

  /**
   * Initiate a new handshake
   */
  initiateHandshake: async (
    request: InitiateHandshakeRequest
  ): Promise<{ handshake_id: string; qr_data: string }> => {
    const response = await apiClient.post<HandshakeApiResponse<{ handshake_id: string; qr_data: string }>>(
      '/handshakes/initiate',
      request
    );
    return response.data.data;
  },

  /**
   * Confirm a handshake with cryptographic signature
   */
  confirmHandshake: async (request: ConfirmHandshakeRequest): Promise<DigitalHandshake> => {
    const response = await apiClient.post<HandshakeApiResponse<DigitalHandshake>>(
      '/handshakes/confirm',
      request
    );
    return response.data.data;
  },

  /**
   * Raise a dispute on a handshake
   */
  raiseDispute: async (request: RaiseDisputeRequest): Promise<DigitalHandshake> => {
    const response = await apiClient.post<HandshakeApiResponse<DigitalHandshake>>(
      '/handshakes/dispute',
      request
    );
    return response.data.data;
  },

  /**
   * Get handshake history timeline for a material
   */
  getHandshakeHistory: async (materialId: string): Promise<DigitalHandshake[]> => {
    return handshakeApi.getMaterialHandshakes(materialId);
  },
};
