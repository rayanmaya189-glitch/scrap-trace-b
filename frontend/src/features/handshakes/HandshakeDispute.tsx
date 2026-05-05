import React, { useState } from 'react';
import { useAuthStore } from '../auth/AuthProvider';
import { Card } from '../../components/ui/Card';
import { Button } from '../../components/ui/Button';
import { Input } from '../../components/ui/Input';
import { Badge } from '../../components/ui/Badge';
import { Toast } from '../../components/ui/Toast';
import { Dialog } from '../../components/ui/Dialog';
import { Camera } from '../../components/ui/Camera';
import { FileUpload } from '../../components/ui/FileUpload';
import { handshakeApi } from '../../api/handshake';
import { apiClient } from '../../api/client';

interface HandshakeRecord {
  id: string;
  material_id: string;
  supplier_sig: string;
  buyer_sig: string;
  payload_hash: string;
  hash_prev: string;
  hash_current: string;
  sync_status: string;
  timestamp_utc: string;
}

export function HandshakeDispute() {
  const { user, token } = useAuthStore();
  const [handshakeId, setHandshakeId] = useState('');
  const [loading, setLoading] = useState(false);
  const [showCamera, setShowCamera] = useState(false);
  const [showUpload, setShowUpload] = useState(false);
  const [evidence, setEvidence] = useState<string[]>([]);
  const [disputeReason, setDisputeReason] = useState('');
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  const [showDialog, setShowDialog] = useState(false);

  const handleCapturePhoto = (photoData: string) => {
    setEvidence([...evidence, photoData]);
    setShowCamera(false);
  };

  const handleFileUpload = async (files: File[]) => {
    const uploadedUrls: string[] = [];
    setLoading(true);
    
    for (const file of files) {
      try {
        // Create FormData for file upload
        const formData = new FormData();
        formData.append('file', file);
        formData.append('purpose', 'dispute_evidence');
        
        // Upload to backend (which will handle MinIO presigned URL or direct storage)
        const response = await apiClient.post('/upload/evidence', formData, {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
          onUploadProgress: (progressEvent) => {
            const percentCompleted = Math.round(
              (progressEvent.loaded * 100) / (progressEvent.total || file.size)
            );
            console.log(`Upload progress: ${percentCompleted}%`);
          },
        });
        
        if (response.data.success && response.data.data?.files) {
          // Extract URLs from the response
          const fileUrls = response.data.data.files.map((f: any) => f.url);
          uploadedUrls.push(...fileUrls);
        } else if (response.data.success && response.data.data?.url) {
          uploadedUrls.push(response.data.data.url);
        }
      } catch (error: any) {
        console.error('Failed to upload file:', error);
        setToast({ 
          message: `Failed to upload ${file.name}: ${error.response?.data?.message || error.message}`, 
          type: 'error' 
        });
        // Fallback: store as base64 if upload fails
        const reader = new FileReader();
        reader.onload = (e) => {
          if (e.target?.result) {
            setEvidence(prev => [...prev, e.target!.result as string]);
          }
        };
        reader.readAsDataURL(file);
      }
    }
    
    // Add uploaded URLs to evidence
    if (uploadedUrls.length > 0) {
      setEvidence(prev => [...prev, ...uploadedUrls]);
      setToast({ message: `${uploadedUrls.length} file(s) uploaded successfully`, type: 'success' });
    }
    
    setLoading(false);
    setShowUpload(false);
  };

  const handleSubmitDispute = async () => {
    if (!handshakeId || !disputeReason) {
      setToast({ message: 'Please provide handshake ID and dispute reason', type: 'error' });
      return;
    }

    if (evidence.length === 0) {
      setToast({ message: 'Please attach at least one piece of evidence', type: 'error' });
      return;
    }

    setLoading(true);
    try {
      // Separate URLs from base64 data
      const evidenceUrls = evidence.filter(e => e.startsWith('http') || e.startsWith('/'));
      const evidenceBase64 = evidence.filter(e => e.startsWith('data:'));
      
      // For base64 images, we'll include them directly in the request
      // In production, these should be uploaded to MinIO first
      const allEvidence = [...evidenceUrls, ...evidenceBase64];

      const response = await handshakeApi.raiseDispute({
        handshake_id: handshakeId,
        reason: disputeReason,
        evidence: JSON.stringify(allEvidence),
      });

      if (response) {
        setToast({ message: 'Dispute submitted successfully', type: 'success' });
        setHandshakeId('');
        setDisputeReason('');
        setEvidence([]);
        setShowDialog(false);
      }
    } catch (error: any) {
      setToast({
        message: error.response?.data?.message || 'Failed to submit dispute',
        type: 'error'
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto p-4 space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Raise Handshake Dispute</h1>

      <Card className="p-6">
        <p className="text-sm text-gray-600 mb-4">
          If you believe a digital handshake is fraudulent or incorrect, you can raise a dispute with evidence.
          This will flag the transaction for manual review.
        </p>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Handshake ID
            </label>
            <Input
              value={handshakeId}
              onChange={(e) => setHandshakeId(e.target.value)}
              placeholder="Enter handshake UUID"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Dispute Reason
            </label>
            <textarea
              value={disputeReason}
              onChange={(e) => setDisputeReason(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              rows={4}
              placeholder="Describe why you are disputing this handshake..."
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Evidence
            </label>
            <div className="flex gap-2 mb-2">
              <Button
                variant="secondary"
                size="sm"
                onClick={() => setShowCamera(true)}
              >
                📷 Take Photo
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => setShowUpload(true)}
              >
                📎 Upload File
              </Button>
            </div>

            {evidence.length > 0 && (
              <div className="grid grid-cols-3 gap-2 mt-2">
                {evidence.map((img, idx) => (
                  <div key={idx} className="relative aspect-square">
                    <img
                      src={img}
                      alt={`Evidence ${idx + 1}`}
                      className="w-full h-full object-cover rounded-lg"
                    />
                    <button
                      onClick={() => setEvidence(evidence.filter((_, i) => i !== idx))}
                      className="absolute top-1 right-1 w-6 h-6 bg-red-500 text-white rounded-full text-xs flex items-center justify-center"
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          <Button
            onClick={() => setShowDialog(true)}
            disabled={!handshakeId || !disputeReason || evidence.length === 0}
            className="w-full"
          >
            Submit Dispute
          </Button>
        </div>
      </Card>

      {showCamera && (
        <Camera
          onCapture={handleCapturePhoto}
          onClose={() => setShowCamera(false)}
        />
      )}

      {showUpload && (
        <FileUpload
          onUpload={handleFileUpload}
          onClose={() => setShowUpload(false)}
          accept="image/*"
          multiple
        />
      )}

      <Dialog
        isOpen={showDialog}
        onClose={() => setShowDialog(false)}
        title="Confirm Dispute Submission"
        description="Once submitted, this dispute will be reviewed by our team. The handshake will be flagged pending investigation."
        confirmText="Submit Dispute"
        cancelText="Cancel"
        onConfirm={handleSubmitDispute}
        isConfirmLoading={loading}
      />

      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          onClose={() => setToast(null)}
        />
      )}
    </div>
  );
}
