import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowLeft, Upload, Camera } from 'lucide-react';
import { apiClient } from '@/stores/useAuthStore';
import { useSyncStore } from '@/stores/useSyncStore';

const MATERIAL_TYPES = [
  'Ferrous Scrap',
  'Non-Ferrous Scrap',
  'Steel Billets',
  'Iron Ore',
  'Coal',
  'Cement',
  'Aluminum',
  'Copper',
  'Other'
];

const MATERIAL_GRADES = [
  'HMS 1&2 (80:20)',
  'HMS 1&2 (70:30)',
  'ISRI 200',
  'ISRI 201',
  'ISRI 202',
  'Grade A',
  'Grade B',
  'Grade C',
  'Custom'
];

export function CreateMaterialForm() {
  const navigate = useNavigate();
  const { addToQueue, saveMaterialOffline, isOnline } = useSyncStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const [formData, setFormData] = useState({
    materialType: '',
    batchWeightKg: '',
    materialGrade: '',
    customGrade: '',
    sourcePincode: '',
    notes: '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);

    try {
      const payload = {
        material_type: formData.materialType,
        batch_weight_kg: parseFloat(formData.batchWeightKg),
        material_grade: formData.materialGrade === 'Custom' ? formData.customGrade : formData.materialGrade,
        source_pincode: formData.sourcePincode,
        notes: formData.notes || undefined,
      };

      if (isOnline) {
        // Online - create via API
        const response = await apiClient.post('/v1/materials', payload);
        const material = response.data.data;
        
        // Also save locally for offline access
        await saveMaterialOffline({
          id: material.id,
          materialType: material.material_type,
          batchWeightKg: material.batch_weight_kg,
          materialGrade: material.material_grade,
          sourcePincode: material.source_pincode,
          supplierId: material.supplier_id,
          status: material.status,
          createdAt: material.created_at,
          updatedAt: material.updated_at,
        });
      } else {
        // Offline - queue for later sync
        const materialId = `offline_${Date.now()}`;
        await addToQueue('CREATE_MATERIAL', {
          id: materialId,
          ...payload,
        });
        
        // Save locally immediately
        await saveMaterialOffline({
          id: materialId,
          materialType: payload.material_type,
          batchWeightKg: payload.batch_weight_kg,
          materialGrade: payload.material_grade,
          sourcePincode: payload.source_pincode,
          supplierId: 'pending',
          status: 'PENDING',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        });
      }

      navigate('/materials');
    } catch (err: any) {
      console.error('Failed to create material:', err);
      setError(err.response?.data?.message || 'Failed to create material. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center">
        <button
          onClick={() => navigate('/materials')}
          className="text-gray-500 hover:text-gray-700"
        >
          <ArrowLeft className="h-6 w-6" />
        </button>
        <div className="ml-4">
          <h1 className="text-2xl font-bold text-gray-900">Create Material Batch</h1>
          <p className="mt-1 text-sm text-gray-500">
            Log a new industrial material batch to start tracking
          </p>
        </div>
      </div>

      {!isOnline && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <p className="text-sm text-yellow-800">
            ⚠️ You're offline. This material will be synced when you reconnect.
          </p>
        </div>
      )}

      {/* Form */}
      <form onSubmit={handleSubmit} className="bg-white rounded-xl shadow-sm border border-gray-200 p-6 space-y-6">
        {error && (
          <div className="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-lg text-sm">
            {error}
          </div>
        )}

        {/* Material Type */}
        <div>
          <label htmlFor="materialType" className="block text-sm font-medium text-gray-700 mb-1">
            Material Type <span className="text-red-500">*</span>
          </label>
          <select
            id="materialType"
            name="materialType"
            required
            value={formData.materialType}
            onChange={handleChange}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          >
            <option value="">Select material type</option>
            {MATERIAL_TYPES.map(type => (
              <option key={type} value={type}>{type}</option>
            ))}
          </select>
        </div>

        {/* Batch Weight */}
        <div>
          <label htmlFor="batchWeightKg" className="block text-sm font-medium text-gray-700 mb-1">
            Batch Weight (kg) <span className="text-red-500">*</span>
          </label>
          <input
            type="number"
            id="batchWeightKg"
            name="batchWeightKg"
            required
            min="0.1"
            step="0.1"
            value={formData.batchWeightKg}
            onChange={handleChange}
            placeholder="e.g., 1000.5"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          />
        </div>

        {/* Material Grade */}
        <div>
          <label htmlFor="materialGrade" className="block text-sm font-medium text-gray-700 mb-1">
            Material Grade <span className="text-red-500">*</span>
          </label>
          <select
            id="materialGrade"
            name="materialGrade"
            required
            value={formData.materialGrade}
            onChange={handleChange}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          >
            <option value="">Select grade</option>
            {MATERIAL_GRADES.map(grade => (
              <option key={grade} value={grade}>{grade}</option>
            ))}
          </select>
        </div>

        {formData.materialGrade === 'Custom' && (
          <div>
            <label htmlFor="customGrade" className="block text-sm font-medium text-gray-700 mb-1">
              Custom Grade Specification <span className="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="customGrade"
              name="customGrade"
              required={formData.materialGrade === 'Custom'}
              value={formData.customGrade}
              onChange={handleChange}
              placeholder="Enter grade specification"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
            />
          </div>
        )}

        {/* Source Pincode */}
        <div>
          <label htmlFor="sourcePincode" className="block text-sm font-medium text-gray-700 mb-1">
            Source Pincode <span className="text-red-500">*</span>
          </label>
          <input
            type="text"
            id="sourcePincode"
            name="sourcePincode"
            required
            pattern="[0-9]{6}"
            maxLength={6}
            value={formData.sourcePincode}
            onChange={handleChange}
            placeholder="6-digit pincode"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          />
        </div>

        {/* Notes */}
        <div>
          <label htmlFor="notes" className="block text-sm font-medium text-gray-700 mb-1">
            Additional Notes
          </label>
          <textarea
            id="notes"
            name="notes"
            rows={3}
            value={formData.notes}
            onChange={handleChange}
            placeholder="Any additional information about this batch..."
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          />
        </div>

        {/* Photo Upload Placeholder */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Slip Photo (Optional)
          </label>
          <div className="mt-1 flex justify-center px-6 pt-5 pb-6 border-2 border-gray-300 border-dashed rounded-lg hover:border-indigo-500 transition-colors cursor-pointer">
            <div className="space-y-1 text-center">
              <Camera className="mx-auto h-12 w-12 text-gray-400" />
              <div className="flex text-sm text-gray-600 justify-center">
                <span className="relative cursor-pointer bg-white rounded-md font-medium text-indigo-600 hover:text-indigo-500">
                  Upload a photo
                </span>
                <p className="pl-1">or drag and drop</p>
              </div>
              <p className="text-xs text-gray-500">PNG, JPG up to 10MB</p>
            </div>
          </div>
        </div>

        {/* Submit Button */}
        <div className="flex items-center justify-end space-x-4 pt-4 border-t border-gray-200">
          <button
            type="button"
            onClick={() => navigate('/materials')}
            className="px-4 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isLoading}
            className="px-6 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading ? 'Creating...' : 'Create Material'}
          </button>
        </div>
      </form>
    </div>
  );
}
