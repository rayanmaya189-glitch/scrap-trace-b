import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useAuth } from '../auth/AuthProvider';

interface HandshakeData {
  materialId: string;
  counterpartyPhone: string;
  quantity: number;
  unit: string;
  notes?: string;
}

export function HandshakeInitiator() {
  const { user } = useAuth();
  const [step, setStep] = useState<'initiate' | 'qr-display' | 'confirm'>('initiate');
  const [handshakeData, setHandshakeData] = useState<HandshakeData>({
    materialId: '',
    counterpartyPhone: '',
    quantity: 0,
    unit: 'kg',
    notes: '',
  });
  const [generatedHash, setGeneratedHash] = useState<string>('');

  const handleInitiate = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // Generate hash for handshake (simplified - in production use Ed25519)
    const hash = await generateHash(JSON.stringify({
      ...handshakeData,
      timestamp: Date.now(),
      initiator: user?.id,
    }));
    
    setGeneratedHash(hash);
    setStep('qr-display');
  };

  const generateHash = async (data: string): Promise<string> => {
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">QR Handshake</h1>
        <p className="text-muted-foreground">
          Initiate and confirm material transfers with cryptographic verification
        </p>
      </div>

      {step === 'initiate' && (
        <Card>
          <CardHeader>
            <CardTitle>Initiate Handshake</CardTitle>
            <CardDescription>
              Enter the details of the material transfer you want to confirm
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleInitiate} className="space-y-4">
              <div>
                <label htmlFor="materialId" className="block text-sm font-medium mb-1">
                  Material Batch ID
                </label>
                <Input
                  id="materialId"
                  value={handshakeData.materialId}
                  onChange={(e) => setHandshakeData({...handshakeData, materialId: e.target.value})}
                  placeholder="Enter batch ID or scan QR"
                  required
                />
              </div>

              <div>
                <label htmlFor="counterparty" className="block text-sm font-medium mb-1">
                  Counterparty Phone
                </label>
                <Input
                  id="counterparty"
                  type="tel"
                  value={handshakeData.counterpartyPhone}
                  onChange={(e) => setHandshakeData({...handshakeData, counterpartyPhone: e.target.value})}
                  placeholder="Enter counterparty phone number"
                  pattern="[0-9]{10,15}"
                  required
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label htmlFor="quantity" className="block text-sm font-medium mb-1">
                    Quantity
                  </label>
                  <Input
                    id="quantity"
                    type="number"
                    value={handshakeData.quantity || ''}
                    onChange={(e) => setHandshakeData({...handshakeData, quantity: parseFloat(e.target.value) || 0})}
                    placeholder="0"
                    min="0"
                    step="0.01"
                    required
                  />
                </div>

                <div>
                  <label htmlFor="unit" className="block text-sm font-medium mb-1">
                    Unit
                  </label>
                  <select
                    id="unit"
                    value={handshakeData.unit}
                    onChange={(e) => setHandshakeData({...handshakeData, unit: e.target.value})}
                    className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  >
                    <option value="kg">Kilograms (kg)</option>
                    <option value="ton">Tonnes (ton)</option>
                    <option value="pcs">Pieces (pcs)</option>
                    <option value="ltr">Liters (ltr)</option>
                  </select>
                </div>
              </div>

              <div>
                <label htmlFor="notes" className="block text-sm font-medium mb-1">
                  Notes (Optional)
                </label>
                <Input
                  id="notes"
                  value={handshakeData.notes || ''}
                  onChange={(e) => setHandshakeData({...handshakeData, notes: e.target.value})}
                  placeholder="Add any additional notes"
                />
              </div>

              <Button type="submit" className="w-full">
                Generate QR Code
              </Button>
            </form>
          </CardContent>
        </Card>
      )}

      {step === 'qr-display' && (
        <Card>
          <CardHeader>
            <CardTitle>Show QR to Counterparty</CardTitle>
            <CardDescription>
              The other party should scan this QR code to confirm the handshake
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-center">
              <div className="bg-white p-4 rounded-lg">
                {/* In production, use qrcode.react library */}
                <div className="w-64 h-64 bg-gray-100 flex items-center justify-center border-2 border-dashed border-gray-300">
                  <div className="text-center text-gray-500">
                    <svg className="w-16 h-16 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" />
                    </svg>
                    <p className="text-sm">QR Code Placeholder</p>
                    <p className="text-xs mt-1">(Install qrcode.react)</p>
                  </div>
                </div>
              </div>
            </div>

            <div className="bg-muted p-4 rounded-md">
              <p className="text-sm font-medium mb-2">Handshake Details:</p>
              <div className="space-y-1 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Material ID:</span>
                  <span className="font-mono">{handshakeData.materialId}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Counterparty:</span>
                  <span>{handshakeData.counterpartyPhone}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Quantity:</span>
                  <span>{handshakeData.quantity} {handshakeData.unit}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Hash:</span>
                  <span className="font-mono text-xs truncate max-w-[200px]">{generatedHash.substring(0, 16)}...</span>
                </div>
              </div>
            </div>

            <div className="flex gap-2">
              <Button variant="outline" onClick={() => setStep('initiate')} className="flex-1">
                Back
              </Button>
              <Button onClick={() => setStep('confirm')} className="flex-1">
                I've Shown QR
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {step === 'confirm' && (
        <Card>
          <CardHeader>
            <CardTitle>Handshake Pending</CardTitle>
            <CardDescription>
              Waiting for counterparty to scan and confirm
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
            </div>
            <p className="text-center text-muted-foreground">
              The handshake will be confirmed once the counterparty scans the QR code and verifies the details.
            </p>
            <Button onClick={() => setStep('initiate')} variant="outline" className="w-full">
              Start New Handshake
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
