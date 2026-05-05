import React, { useState, useEffect } from 'react';
import { useAuthStore } from '../auth/AuthProvider';
import { Card } from '../../components/ui/Card';
import { Button } from '../../components/ui/Button';
import { Input } from '../../components/ui/Input';
import { Badge } from '../../components/ui/Badge';
import { Toast } from '../../components/ui/Toast';
import { Avatar } from '../../components/ui/Avatar';
import { Tabs } from '../../components/ui/Tabs';
import api from '../../lib/api';

interface ProfileData {
  id: string;
  phone: string;
  name: string | null;
  business_name: string | null;
  role: string;
  pincode: string | null;
  gst_number: string | null;
  is_verified: boolean;
  created_at: string;
}

interface ConsentRecord {
  id: string;
  supplier_id: string;
  purpose: string;
  granted: boolean;
  revoked_at: string | null;
  created_at: string;
}

export function ProfileSettings() {
  const { user, token } = useAuthStore();
  const [profile, setProfile] = useState<ProfileData | null>(null);
  const [consents, setConsents] = useState<ConsentRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  
  const [formData, setFormData] = useState({
    name: '',
    business_name: '',
    pincode: '',
    gst_number: '',
  });

  useEffect(() => {
    loadProfile();
    loadConsents();
  }, []);

  const loadProfile = async () => {
    try {
      const response = await api.get('/suppliers/me', {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (response.data.success && response.data.data) {
        setProfile(response.data.data);
        setFormData({
          name: response.data.data.name || '',
          business_name: response.data.data.business_name || '',
          pincode: response.data.data.pincode || '',
          gst_number: response.data.data.gst_number || '',
        });
      }
    } catch (error) {
      setToast({ message: 'Failed to load profile', type: 'error' });
    } finally {
      setLoading(false);
    }
  };

  const loadConsents = async () => {
    try {
      const response = await api.get('/consent/my', {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (response.data.success) {
        setConsents(response.data.data || []);
      }
    } catch (error) {
      console.error('Failed to load consents:', error);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      const response = await api.put(
        `/suppliers/${user?.id}`,
        formData,
        { headers: { Authorization: `Bearer ${token}` } }
      );
      if (response.data.success) {
        setToast({ message: 'Profile updated successfully', type: 'success' });
        loadProfile();
      }
    } catch (error) {
      setToast({ message: 'Failed to update profile', type: 'error' });
    } finally {
      setSaving(false);
    }
  };

  const handleRevokeConsent = async (consentId: string) => {
    try {
      const response = await api.post(
        `/consent/${consentId}/revoke`,
        {},
        { headers: { Authorization: `Bearer ${token}` } }
      );
      if (response.data.success) {
        setToast({ message: 'Consent revoked successfully', type: 'success' });
        loadConsents();
      }
    } catch (error) {
      setToast({ message: 'Failed to revoke consent', type: 'error' });
    }
  };

  const handleGrantConsent = async (purpose: string) => {
    try {
      const response = await api.post(
        '/consent',
        {
          supplier_id: user?.id,
          purpose,
          granted: true,
        },
        { headers: { Authorization: `Bearer ${token}` } }
      );
      if (response.data.success) {
        setToast({ message: 'Consent granted successfully', type: 'success' });
        loadConsents();
      }
    } catch (error) {
      setToast({ message: 'Failed to grant consent', type: 'error' });
    }
  };

  if (loading) {
    return <div className="flex items-center justify-center p-8">Loading...</div>;
  }

  return (
    <div className="max-w-4xl mx-auto p-4 space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Profile & Settings</h1>
      
      <Tabs
        tabs={[
          { id: 'profile', label: 'Profile' },
          { id: 'verification', label: 'Verification' },
          { id: 'consent', label: 'Consent Management' },
          { id: 'devices', label: 'Devices' },
        ]}
        renderTab={(tabId) => {
          switch (tabId) {
            case 'profile':
              return (
                <Card className="p-6">
                  <h2 className="text-xl font-semibold mb-4">Business Information</h2>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Full Name
                      </label>
                      <Input
                        value={formData.name}
                        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        placeholder="Enter your full name"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Business Name
                      </label>
                      <Input
                        value={formData.business_name}
                        onChange={(e) => setFormData({ ...formData, business_name: e.target.value })}
                        placeholder="Enter business name"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Pincode
                        </label>
                        <Input
                          value={formData.pincode}
                          onChange={(e) => setFormData({ ...formData, pincode: e.target.value })}
                          placeholder="110001"
                          maxLength={6}
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          GST Number
                        </label>
                        <Input
                          value={formData.gst_number}
                          onChange={(e) => setFormData({ ...formData, gst_number: e.target.value.toUpperCase() })}
                          placeholder="22AAAAA0000A1Z5"
                          maxLength={15}
                        />
                      </div>
                    </div>
                    <div className="flex items-center gap-4 pt-4">
                      <Button onClick={handleSave} disabled={saving}>
                        {saving ? 'Saving...' : 'Save Changes'}
                      </Button>
                      {profile?.is_verified && (
                        <Badge variant="success">Verified</Badge>
                      )}
                    </div>
                  </div>
                </Card>
              );
            
            case 'verification':
              return (
                <Card className="p-6">
                  <h2 className="text-xl font-semibold mb-4">KYC Verification Status</h2>
                  <div className="space-y-4">
                    <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                      <div>
                        <p className="font-medium">Business Verification</p>
                        <p className="text-sm text-gray-500">GST and business documents</p>
                      </div>
                      {profile?.is_verified ? (
                        <Badge variant="success">Verified</Badge>
                      ) : (
                        <Badge variant="warning">Pending</Badge>
                      )}
                    </div>
                    
                    {!profile?.is_verified && (
                      <div className="pt-4">
                        <Button variant="secondary">Upload Documents</Button>
                        <p className="text-sm text-gray-500 mt-2">
                          Upload GST certificate and business proof for verification
                        </p>
                      </div>
                    )}
                  </div>
                </Card>
              );
            
            case 'consent':
              return (
                <Card className="p-6">
                  <h2 className="text-xl font-semibold mb-4">Data Consent Management</h2>
                  <p className="text-sm text-gray-600 mb-4">
                    Manage your consent for data sharing as per DPDP 2023
                  </p>
                  
                  <div className="space-y-4">
                    <div className="flex items-center justify-between p-4 border rounded-lg">
                      <div>
                        <p className="font-medium">Credit Scoring</p>
                        <p className="text-sm text-gray-500">Allow NBFCs to access your transaction history for scoring</p>
                      </div>
                      {consents.some(c => c.purpose === 'credit_scoring' && c.granted && !c.revoked_at) ? (
                        <Button 
                          variant="secondary" 
                          size="sm"
                          onClick={() => {
                            const consent = consents.find(c => c.purpose === 'credit_scoring' && c.granted && !c.revoked_at);
                            if (consent) handleRevokeConsent(consent.id);
                          }}
                        >
                          Revoke
                        </Button>
                      ) : (
                        <Button 
                          size="sm"
                          onClick={() => handleGrantConsent('credit_scoring')}
                        >
                          Grant
                        </Button>
                      )}
                    </div>
                    
                    <div className="flex items-center justify-between p-4 border rounded-lg">
                      <div>
                        <p className="font-medium">Compliance Reporting</p>
                        <p className="text-sm text-gray-500">Generate CBAM/EPR reports for regulatory compliance</p>
                      </div>
                      {consents.some(c => c.purpose === 'compliance_reporting' && c.granted && !c.revoked_at) ? (
                        <Button 
                          variant="secondary" 
                          size="sm"
                          onClick={() => {
                            const consent = consents.find(c => c.purpose === 'compliance_reporting' && c.granted && !c.revoked_at);
                            if (consent) handleRevokeConsent(consent.id);
                          }}
                        >
                          Revoke
                        </Button>
                      ) : (
                        <Button 
                          size="sm"
                          onClick={() => handleGrantConsent('compliance_reporting')}
                        >
                          Grant
                        </Button>
                      )}
                    </div>
                  </div>
                  
                  {consents.length > 0 && (
                    <div className="mt-6">
                      <h3 className="font-medium mb-2">Consent History</h3>
                      <div className="space-y-2 max-h-64 overflow-y-auto">
                        {consents.map((consent) => (
                          <div key={consent.id} className="flex items-center justify-between p-3 bg-gray-50 rounded">
                            <div>
                              <p className="text-sm font-medium">{consent.purpose}</p>
                              <p className="text-xs text-gray-500">
                                {new Date(consent.created_at).toLocaleDateString()}
                                {consent.revoked_at && ` • Revoked: ${new Date(consent.revoked_at).toLocaleDateString()}`}
                              </p>
                            </div>
                            <Badge variant={consent.granted && !consent.revoked_at ? 'success' : 'secondary'}>
                              {consent.revoked_at ? 'Revoked' : consent.granted ? 'Active' : 'Denied'}
                            </Badge>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </Card>
              );
            
            case 'devices':
              return (
                <Card className="p-6">
                  <h2 className="text-xl font-semibold mb-4">Registered Devices</h2>
                  <div className="space-y-4">
                    <div className="p-4 border rounded-lg">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <div className="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
                            📱
                          </div>
                          <div>
                            <p className="font-medium">Current Device</p>
                            <p className="text-sm text-gray-500">Last active: Just now</p>
                          </div>
                        </div>
                        <Badge variant="success">Active</Badge>
                      </div>
                    </div>
                    <p className="text-sm text-gray-500">
                      You can revoke access to other registered devices from here.
                    </p>
                  </div>
                </Card>
              );
            
            default:
              return null;
          }
        }}
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
