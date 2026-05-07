import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Button } from '../components/ui/button';
import { Badge } from '../components/ui/badge';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../components/ui/table';
import { Shield, Download, CheckCircle, AlertTriangle, FileText, Clock } from 'lucide-react';
import { apiClient } from '../api/client';

interface AuditLogEntry {
  id: string;
  event_type: string;
  subject: string;
  payload_hash: string;
  processed_at: string;
  merkle_root?: string;
}

interface MerkleProofResponse {
  leaf_hash: string;
  proof_hashes: Array<{ hash: string; position: string }>;
  root_hash: string;
  verified: boolean;
}

export function AuditTrailPage() {
  const [auditLogs, setAuditLogs] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedEntry, setSelectedEntry] = useState<string | null>(null);
  const [merkleProof, setMerkleProof] = useState<MerkleProofResponse | null>(null);
  const [verifying, setVerifying] = useState(false);

  useEffect(() => {
    fetchAuditLogs();
  }, []);

  const fetchAuditLogs = async () => {
    try {
      const response = await apiClient.get('/audit');
      if (response.ok) {
        const data = await response.json();
        setAuditLogs(data);
      }
    } catch (error) {
      console.error('Failed to fetch audit logs:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchMerkleProof = async (entryId: string) => {
    setVerifying(true);
    try {
      const response = await apiClient.get(`/audit/merkle-proof/${entryId}`);
      if (response.ok) {
        const proof = await response.json();
        setMerkleProof(proof);
        setSelectedEntry(entryId);
      }
    } catch (error) {
      console.error('Failed to fetch Merkle proof:', error);
    } finally {
      setVerifying(false);
    }
  };

  const exportAuditTrail = async () => {
    try {
      const response = await apiClient.get('/audit/export');
      if (response.ok) {
        const data = await response.json();
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `audit-trail-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      }
    } catch (error) {
      console.error('Failed to export audit trail:', error);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString('en-IN', {
      dateStyle: 'medium',
      timeStyle: 'short',
    });
  };

  const truncateHash = (hash: string) => {
    return `${hash.slice(0, 8)}...${hash.slice(-8)}`;
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Shield className="h-8 w-8" />
            Audit Trail
          </h1>
          <p className="text-muted-foreground mt-1">
            Cryptographically verifiable event log with Merkle proofs
          </p>
        </div>
        <Button onClick={exportAuditTrail} variant="outline">
          <Download className="h-4 w-4 mr-2" />
          Export Trail
        </Button>
      </div>

      {loading ? (
        <Card>
          <CardContent className="p-8 text-center">
            <Clock className="h-8 w-8 animate-spin mx-auto mb-2" />
            <p>Loading audit logs...</p>
          </CardContent>
        </Card>
      ) : (
        <>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <FileText className="h-5 w-5" />
                Event Log ({auditLogs.length} entries)
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Timestamp</TableHead>
                    <TableHead>Event Type</TableHead>
                    <TableHead>Payload Hash</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {auditLogs.map((log) => (
                    <TableRow key={log.id}>
                      <TableCell>{formatDate(log.processed_at)}</TableCell>
                      <TableCell>
                        <Badge variant="secondary">{log.event_type}</Badge>
                      </TableCell>
                      <TableCell className="font-mono text-xs">
                        {truncateHash(log.payload_hash)}
                      </TableCell>
                      <TableCell>
                        {merkleProof?.leaf_hash === log.payload_hash && merkleProof.verified ? (
                          <Badge variant="default" className="bg-green-600">
                            <CheckCircle className="h-3 w-3 mr-1" />
                            Verified
                          </Badge>
                        ) : (
                          <Badge variant="outline">Pending</Badge>
                        )}
                      </TableCell>
                      <TableCell>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => fetchMerkleProof(log.id)}
                          disabled={verifying}
                        >
                          {verifying && selectedEntry === log.id ? (
                            <Clock className="h-3 w-3 animate-spin" />
                          ) : (
                            <Shield className="h-3 w-3" />
                          )}
                          <span className="ml-1">Verify</span>
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>

          {merkleProof && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5" />
                  Merkle Proof Verification
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="text-sm font-medium mb-2">Leaf Hash</h4>
                    <code className="block p-2 bg-muted rounded text-xs break-all">
                      {merkleProof.leaf_hash}
                    </code>
                  </div>
                  <div>
                    <h4 className="text-sm font-medium mb-2">Root Hash</h4>
                    <code className="block p-2 bg-muted rounded text-xs break-all">
                      {merkleProof.root_hash}
                    </code>
                  </div>
                </div>

                <div>
                  <h4 className="text-sm font-medium mb-2">Proof Path ({merkleProof.proof_hashes.length} nodes)</h4>
                  <div className="space-y-1">
                    {merkleProof.proof_hashes.map((node, index) => (
                      <div key={index} className="flex items-center gap-2 text-xs">
                        <Badge variant="outline">{node.position}</Badge>
                        <code className="flex-1 bg-muted p-1 rounded break-all">
                          {node.hash}
                        </code>
                      </div>
                    ))}
                  </div>
                </div>

                <div className={`p-4 rounded-lg ${merkleProof.verified ? 'bg-green-100' : 'bg-red-100'}`}>
                  <div className="flex items-center gap-2">
                    {merkleProof.verified ? (
                      <>
                        <CheckCircle className="h-5 w-5 text-green-700" />
                        <span className="font-medium text-green-700">Proof Verified Successfully</span>
                      </>
                    ) : (
                      <>
                        <AlertTriangle className="h-5 w-5 text-red-700" />
                        <span className="font-medium text-red-700">Verification Failed</span>
                      </>
                    )}
                  </div>
                  <p className={`text-sm mt-1 ${merkleProof.verified ? 'text-green-600' : 'text-red-600'}`}>
                    {merkleProof.verified
                      ? 'This event is cryptographically proven to be part of the immutable audit trail.'
                      : 'The proof verification failed. This entry may have been tampered with.'}
                  </p>
                </div>
              </CardContent>
            </Card>
          )}
        </>
      )}
    </div>
  );
}
