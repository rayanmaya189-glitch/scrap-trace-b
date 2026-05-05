import { apiClient } from './client';

export interface MaterialPassport {
  id: string;
  supplier_id: string;
  material_type: string;
  grade: string;
  batch_weight_kg: number;
  source_pincode: string;
  buyer_id?: string;
  status: 'in_transit' | 'delivered' | 'quality_check' | 'accepted' | 'rejected';
  cbam_fields?: {
    embedded_carbon_kg?: number;
    energy_consumption_kwh?: number;
    direct_emissions?: number;
    indirect_emissions?: number;
    recycled_content_percent?: number;
    epr_category?: string;
    producer_responsibility_org?: string;
  };
  created_at: string;
  updated_at: string;
}

export interface CreateMaterialRequest {
  material_type: string;
  grade: string;
  batch_weight_kg: number;
  source_pincode: string;
  cbam_fields?: {
    embedded_carbon_kg?: number;
    energy_consumption_kwh?: number;
    direct_emissions?: number;
    indirect_emissions?: number;
    recycled_content_percent?: number;
    epr_category?: string;
    producer_responsibility_org?: string;
  };
}

export interface MaterialApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
  timestamp: string;
}

export const materialApi = {
  /**
   * Get all materials
   */
  getMaterials: async (): Promise<MaterialPassport[]> => {
    const response = await apiClient.get<MaterialApiResponse<MaterialPassport[]>>('/materials');
    return response.data.data;
  },

  /**
   * Get a single material by ID
   */
  getMaterial: async (id: string): Promise<MaterialPassport> => {
    const response = await apiClient.get<MaterialApiResponse<MaterialPassport>>(`/materials/${id}`);
    return response.data.data;
  },

  /**
   * Create a new material
   */
  createMaterial: async (request: CreateMaterialRequest): Promise<MaterialPassport> => {
    const response = await apiClient.post<MaterialApiResponse<MaterialPassport>>(
      '/materials',
      request
    );
    return response.data.data;
  },

  /**
   * Update material status
   */
  updateMaterialStatus: async (
    id: string,
    status: 'in_transit' | 'delivered' | 'quality_check' | 'accepted' | 'rejected'
  ): Promise<MaterialPassport> => {
    const response = await apiClient.patch<MaterialApiResponse<MaterialPassport>>(
      `/materials/${id}/status/${status}`
    );
    return response.data.data;
  },

  /**
   * Assign buyer to material
   */
  assignBuyer: async (materialId: string, buyerId: string): Promise<MaterialPassport> => {
    const response = await apiClient.patch<MaterialApiResponse<MaterialPassport>>(
      `/materials/${materialId}/buyer/${buyerId}`
    );
    return response.data.data;
  },

  /**
   * Get material summary statistics
   */
  getMaterialSummary: async (): Promise<{
    total_materials: number;
    total_weight_kg: number;
    by_status: Record<string, number>;
    by_material_type: Record<string, number>;
  }> => {
    const response = await apiClient.get<MaterialApiResponse<Record<string, unknown>>>(
      '/materials/summary'
    );
    return response.data.data as {
      total_materials: number;
      total_weight_kg: number;
      by_status: Record<string, number>;
      by_material_type: Record<string, number>;
    };
  },
};
