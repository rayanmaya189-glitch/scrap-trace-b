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
import api from '../../lib/api';

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

export function HandshakeConfirm() {
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

  const handleFileUpload = (files: File[]) => {
    files.forEach(file => {
      const reader = new FileReader();
      reader.onload = (e) => {
        if (e.target?.result) {
          setEvidence(prev => [...prev, e.target!.result as string]);
        }
      };
      reader.readAsDataURL(file);
    });
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
      // Upload evidence to MinIO (placeholder - actual implementation would use presigned URLs)
      const evidenceUrls: string[] = [];
      for (const img of evidence) {
        // In production, upload to MinIO and get URL
        evidenceUrls.push(img); // For now, store base64
      }

      const response = await api.post(
        '/handshakes/dispute',
        {
          handshake_id: handshakeId,
          reason: disputeReason,
          evidence_urls: evidenceUrls,
          disputed_by: user?.id,
        },
        { headers: { Authorization: `Bearer ${token}` } }
      );

      if (response.data.success) {
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
