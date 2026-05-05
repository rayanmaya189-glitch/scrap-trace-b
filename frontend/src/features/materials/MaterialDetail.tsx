import React, { useEffect, useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { 
  Package, Calendar, MapPin, Weight, User, Hash, 
  ArrowRight, Clock, CheckCircle, AlertTriangle, XCircle,
  Download, Share2, MoreVertical, Activity
} from 'lucide-react';
import { apiClient } from '../../stores/useAuthStore';
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card';
import { Button } from '../../components/ui/button';
import { Badge } from '../../components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../components/ui/tabs';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../../components/ui/table';
import { toast } from '../../components/ui/toast';

interface MaterialPassport {
  id: string;
  supplier_id: string;
  material_type: string;
  material_grade: string;
  batch_weight_kg: string;
  source_pincode: string;
  destination_pincode?: string;
  manufacturing_date: string;
  expiry_date?: string;
  status: string;
  cbam_category?: string;
  carbon_intensity?: number;
  recycled_content_pct?: number;
  hazard_class?: string;
  slip_photo_url?: string;
  created_at: string;
  updated_at: string;
  buyer_id?: string;
}

interface Handshake {
  id: string;
  material_id: string;
  supplier_id: string;
  buyer_id: string;
  payload_hash: string;
  hash_prev: string;
  hash_current: string;
  sync_status: string;
  timestamp_utc: string;
  supplier_name?: string;
  buyer_name?: string;
}

export function MaterialDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [material, setMaterial] = useState<MaterialPassport | null>(null);
  const [handshakes, setHandshakes] = useState<Handshake[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadMaterialDetails();
  }, [id]);

  async function loadMaterialDetails() {
    if (!id) return;
    
    try {
      setLoading(true);
      const [materialRes, handshakesRes] = await Promise.all([
        apiClient.get(`/v1/materials/${id}`),
        apiClient.get(`/v1/handshakes?material_id=${id}`)
      ]);

      setMaterial(materialRes.data.data);
      setHandshakes(handshakesRes.data.data?.data || []);
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to load material details');
      toast({
        title: 'Error',
        description: 'Failed to load material details',
        variant: 'destructive'
      });
    } finally {
      setLoading(false);
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  if (error || !material) {
    return (
      <div className="text-center py-12">
        <AlertTriangle className="h-12 w-12 text-red-500 mx-auto mb-4" />
        <h3 className="text-lg font-semibold text-gray-900 mb-2">Material Not Found</h3>
        <p className="text-gray-500 mb-4">{error || 'The requested material does not exist'}</p>
        <Button onClick={() => navigate('/materials')}>
          Back to Materials
        </Button>
      </div>
    );
  }

  const getStatusBadge = (status: string) => {
    const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
      PENDING: 'secondary',
      CONFIRMED: 'default',
      IN_TRANSIT: 'default',
      DELIVERED: 'default',
      DISPUTED: 'destructive',
      CANCELLED: 'outline'
    };

    return (
      <Badge variant={variants[status] || 'secondary'}>
        {status}
      </Badge>
    );
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="outline" onClick={() => navigate('/materials')}>
            ← Back
          </Button>
          <div>
            <h1 className="text-2xl font-bold text-gray-900">
              Material Passport #{material.id.slice(0, 8)}
            </h1>
            <p className="text-sm text-gray-500">
              Created {new Date(material.created_at).toLocaleDateString()}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
          <Button variant="outline" size="sm">
            <Share2 className="h-4 w-4 mr-2" />
            Share
          </Button>
          {material.status === 'PENDING' && (
            <Link to={`/materials/${id}/handshake`}>
              <Button size="sm">
                Initiate Handshake
                <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </Link>
          )}
        </div>
      </div>

      {/* Status Banner */}
      <div className={`rounded-lg p-4 ${
        material.status === 'DISPUTED' ? 'bg-red-50 border border-red-200' :
        material.status === 'CONFIRMED' ? 'bg-green-50 border border-green-200' :
        'bg-blue-50 border border-blue-200'
      }`}>
        <div className="flex items-center gap-3">
          {material.status === 'DISPUTED' ? (
            <XCircle className="h-6 w-6 text-red-600" />
          ) : material.status === 'CONFIRMED' ? (
            <CheckCircle className="h-6 w-6 text-green-600" />
          ) : (
            <Clock className="h-6 w-6 text-blue-600" />
          )}
          <div>
            <p className="font-semibold text-gray-900">Status: {material.status}</p>
            <p className="text-sm text-gray-600">
              {material.status === 'CONFIRMED' ? 'Ownership transferred successfully' :
               material.status === 'DISPUTED' ? 'This material is under dispute' :
               'Awaiting confirmation'}
            </p>
          </div>
        </div>
      </div>

      <Tabs defaultValue="details" className="space-y-4">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="compliance">Compliance</TabsTrigger>
          <TabsTrigger value="history">Handshake History</TabsTrigger>
        </TabsList>

        <TabsContent value="details" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-gray-500">Material Type</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <Package className="h-4 w-4 text-gray-400" />
                  <span className="text-lg font-semibold">{material.material_type}</span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-gray-500">Grade</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <Hash className="h-4 w-4 text-gray-400" />
                  <span className="text-lg font-semibold">{material.material_grade}</span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-gray-500">Batch Weight</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <Weight className="h-4 w-4 text-gray-400" />
                  <span className="text-lg font-semibold">{material.batch_weight_kg} kg</span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-gray-500">Source Location</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <MapPin className="h-4 w-4 text-gray-400" />
                  <span className="text-lg font-semibold">PIN: {material.source_pincode}</span>
                </div>
              </CardContent>
            </Card>

            {material.destination_pincode && (
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium text-gray-500">Destination</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="flex items-center gap-2">
                    <MapPin className="h-4 w-4 text-gray-400" />
                    <span className="text-lg font-semibold">PIN: {material.destination_pincode}</span>
                  </div>
                </CardContent>
              </Card>
            )}

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium text-gray-500">Manufacturing Date</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <Calendar className="h-4 w-4 text-gray-400" />
                  <span className="text-lg font-semibold">
                    {new Date(material.manufacturing_date).toLocaleDateString()}
                  </span>
                </div>
              </CardContent>
            </Card>
          </div>

          {material.slip_photo_url && (
            <Card>
              <CardHeader>
                <CardTitle>Slip Photo</CardTitle>
              </CardHeader>
              <CardContent>
                <img 
                  src={material.slip_photo_url} 
                  alt="Material slip" 
                  className="w-full max-w-md rounded-lg border"
                />
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="compliance" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  CBAM Information
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-gray-500">Category:</span>
                  <span className="font-medium">{material.cbam_category || 'N/A'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-500">Carbon Intensity:</span>
                  <span className="font-medium">
                    {material.carbon_intensity ? `${material.carbon_intensity} kg CO2e` : 'N/A'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-500">Recycled Content:</span>
                  <span className="font-medium">
                    {material.recycled_content_pct ? `${material.recycled_content_pct}%` : 'N/A'}
                  </span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <AlertTriangle className="h-5 w-5" />
                  Safety Information
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-gray-500">Hazard Class:</span>
                  <Badge variant={material.hazard_class ? 'destructive' : 'secondary'}>
                    {material.hazard_class || 'Non-Hazardous'}
                  </Badge>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-500">Expiry Date:</span>
                  <span className="font-medium">
                    {material.expiry_date ? new Date(material.expiry_date).toLocaleDateString() : 'N/A'}
                  </span>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="history" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Handshake History</CardTitle>
              <p className="text-sm text-gray-500">
                Complete ownership transfer timeline
              </p>
            </CardHeader>
            <CardContent>
              {handshakes.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  <Activity className="h-12 w-12 mx-auto mb-2 opacity-50" />
                  <p>No handshake history yet</p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Date</TableHead>
                      <TableHead>From</TableHead>
                      <TableHead>To</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead>Hash</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {handshakes.map((handshake) => (
                      <TableRow key={handshake.id}>
                        <TableCell>
                          {new Date(handshake.timestamp_utc).toLocaleString()}
                        </TableCell>
                        <TableCell className="truncate max-w-[150px]">
                          {handshake.supplier_name || handshake.supplier_id.slice(0, 8)}
                        </TableCell>
                        <TableCell className="truncate max-w-[150px]">
                          {handshake.buyer_name || handshake.buyer_id.slice(0, 8)}
                        </TableCell>
                        <TableCell>
                          <Badge variant={
                            handshake.sync_status === 'SYNCED' ? 'default' :
                            handshake.sync_status === 'DISPUTED' ? 'destructive' : 'secondary'
                          }>
                            {handshake.sync_status}
                          </Badge>
                        </TableCell>
                        <TableCell className="font-mono text-xs truncate max-w-[200px]">
                          {handshake.hash_current.slice(0, 16)}...
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
