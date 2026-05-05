import React, { useState, useEffect } from 'react';
import { Card } from '../../components/ui/card';
import { Button } from '../../components/ui/button';
import { Badge } from '../../components/ui/badge';
import { Table } from '../../components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../components/ui/tabs';
import { BarChart3, FileText, Shield, AlertTriangle, CheckCircle, Download, RefreshCw } from 'lucide-react';
import { consentApi, reportApi, type ConsentRecord, type ReportData } from '../../api';
import { useAuthStore } from '../../stores/useAuthStore';

interface ComplianceReport {
  id: string;
  type: 'CBAM' | 'EPR' | 'GST' | 'AUDIT';
  status: 'PENDING' | 'GENERATING' | 'READY' | 'FAILED';
  period: string;
  generatedAt?: string;
  downloadUrl?: string;
  data?: ReportData;
}

interface ComplianceDeadline {
  id: string;
  type: string;
  dueDate: string;
  status: 'UPCOMING' | 'OVERDUE' | 'COMPLETED';
  description: string;
}

interface AuditLogEntry {
  timestamp: string;
  action: string;
  resource: string;
  actor: string;
  hash: string;
}

export function ComplianceDashboard() {
  const { user } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [reports, setReports] = useState<ComplianceReport[]>([]);
  const [consents, setConsents] = useState<ConsentRecord[]>([]);
  const [auditLogs, setAuditLogs] = useState<AuditLogEntry[]>([]);
  const [generatingReport, setGeneratingReport] = useState<string | null>(null);

  const deadlines: ComplianceDeadline[] = [
    { id: '1', type: 'CBAM', dueDate: '2024-05-31', status: 'UPCOMING', description: 'Quarterly CBAM report submission' },
    { id: '2', type: 'EPR', dueDate: '2024-04-30', status: 'OVERDUE', description: 'Annual EPR compliance filing' },
    { id: '3', type: 'GST', dueDate: '2024-04-20', status: 'COMPLETED', description: 'Monthly GST return' },
  ];

  // Fetch consents on mount
  useEffect(() => {
    loadConsents();
    loadAuditLogs();
  }, []);

  const loadConsents = async () => {
    try {
      const data = await consentApi.getMyConsents();
      setConsents(data);
    } catch (error) {
      console.error('Failed to load consents:', error);
    }
  };

  const loadAuditLogs = async () => {
    // Mock audit logs - will be replaced with API call
    setAuditLogs([
      { timestamp: '2024-04-20 14:32:15 UTC', action: 'material.created', resource: 'MAT-2024-001', actor: '9876543210', hash: 'a1b2c3d4...' },
      { timestamp: '2024-04-20 15:45:22 UTC', action: 'handshake.confirmed', resource: 'MAT-2024-001', actor: '9123456789', hash: 'e5f6g7h8...' },
      { timestamp: '2024-04-21 09:15:00 UTC', action: 'score.calculated', resource: 'SUP-2024-042', actor: 'SYSTEM', hash: 'i9j0k1l2...' },
    ]);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'READY':
      case 'COMPLETED':
        return 'bg-green-100 text-green-800 border-green-300';
      case 'GENERATING':
      case 'UPCOMING':
        return 'bg-blue-100 text-blue-800 border-blue-300';
      case 'PENDING':
        return 'bg-yellow-100 text-yellow-800 border-yellow-300';
      case 'FAILED':
      case 'OVERDUE':
        return 'bg-red-100 text-red-800 border-red-300';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-300';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'CBAM':
        return <BarChart3 className="w-4 h-4" />;
      case 'EPR':
        return <Shield className="w-4 h-4" />;
      case 'GST':
        return <FileText className="w-4 h-4" />;
      case 'AUDIT':
        return <CheckCircle className="w-4 h-4" />;
      default:
        return <FileText className="w-4 h-4" />;
    }
  };

  const handleGenerateReport = async (type: 'CBAM' | 'EPR' | 'GST' | 'ALL') => {
    if (type === 'ALL') {
      await Promise.all(['CBAM', 'EPR', 'GST'].map(t => handleGenerateReport(t as 'CBAM' | 'EPR' | 'GST')));
      return;
    }

    setGeneratingReport(type);
    try {
      const reportType = type === 'CBAM' ? 'cbam' : type === 'EPR' ? 'epr' : 'gst_audit';
      const reportData = await reportApi.generateReport({
        report_type: reportType,
        format: 'json',
      });

      const newReport: ComplianceReport = {
        id: Date.now().toString(),
        type,
        status: 'READY',
        period: 'Current Period',
        generatedAt: new Date().toISOString(),
        data: reportData,
      };

      setReports(prev => [newReport, ...prev]);
    } catch (error) {
      console.error(`Failed to generate ${type} report:`, error);
      const failedReport: ComplianceReport = {
        id: Date.now().toString(),
        type,
        status: 'FAILED',
        period: 'Current Period',
      };
      setReports(prev => [failedReport, ...prev]);
    } finally {
      setGeneratingReport(null);
    }
  };

  const handleDownloadReport = async (report: ComplianceReport) => {
    if (!report.data) return;
    
    try {
      await reportApi.downloadReport(report.data, 'json');
    } catch (error) {
      console.error('Failed to download report:', error);
    }
  };

  const handleToggleConsent = async (consent: ConsentRecord) => {
    try {
      if (consent.granted) {
        await consentApi.revokeConsent(consent.id);
      } else {
        await consentApi.createConsent({
          supplier_id: user?.id || '',
          purpose: consent.purpose,
          granted: true,
        });
      }
      await loadConsents();
    } catch (error) {
      console.error('Failed to update consent:', error);
    }
  };

  const handleExportConsent = async () => {
    try {
      const reportData = await reportApi.generateConsentExport();
      await reportApi.downloadReport(reportData, 'csv');
    } catch (error) {
      console.error('Failed to export consent:', error);
    }
  };

  const handleExportAuditLog = async () => {
    try {
      const reportData = await reportApi.generateGstAuditReport();
      await reportApi.downloadReport(reportData, 'csv');
    } catch (error) {
      console.error('Failed to export audit log:', error);
    }
  };

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Compliance Dashboard</h1>
          <p className="text-gray-600 mt-1">Manage regulatory reports and deadlines</p>
        </div>
        <Button onClick={() => handleGenerateReport('ALL')} disabled={loading}>
          <RefreshCw className={`w-4 h-4 mr-2 ${generatingReport ? 'animate-spin' : ''}`} />
          Generate All Reports
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Total Reports</p>
              <p className="text-2xl font-bold">{reports.length + 4}</p>
            </div>
            <FileText className="w-8 h-8 text-blue-600" />
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Ready for Download</p>
              <p className="text-2xl font-bold text-green-600">{reports.filter(r => r.status === 'READY').length + 8}</p>
            </div>
            <Download className="w-8 h-8 text-green-600" />
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Upcoming Deadlines</p>
              <p className="text-2xl font-bold text-blue-600">{deadlines.filter(d => d.status === 'UPCOMING').length}</p>
            </div>
            <AlertTriangle className="w-8 h-8 text-blue-600" />
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-600">Overdue</p>
              <p className="text-2xl font-bold text-red-600">{deadlines.filter(d => d.status === 'OVERDUE').length}</p>
            </div>
            <AlertTriangle className="w-8 h-8 text-red-600" />
          </div>
        </Card>
      </div>

      <Tabs defaultValue="reports" className="space-y-4">
        <TabsList>
          <TabsTrigger value="reports">Reports</TabsTrigger>
          <TabsTrigger value="deadlines">Deadlines</TabsTrigger>
          <TabsTrigger value="audit-trail">Audit Trail</TabsTrigger>
          <TabsTrigger value="consent">Consent Management</TabsTrigger>
        </TabsList>

        {/* Reports Tab */}
        <TabsContent value="reports">
          <Card className="p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">Compliance Reports</h2>
              <div className="space-x-2">
                <Button variant="outline" size="sm" onClick={() => handleGenerateReport('CBAM')} disabled={generatingReport === 'CBAM'}>
                  {generatingReport === 'CBAM' ? <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> : null}
                  CBAM Report
                </Button>
                <Button variant="outline" size="sm" onClick={() => handleGenerateReport('EPR')} disabled={generatingReport === 'EPR'}>
                  {generatingReport === 'EPR' ? <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> : null}
                  EPR Report
                </Button>
                <Button variant="outline" size="sm" onClick={() => handleGenerateReport('GST')} disabled={generatingReport === 'GST'}>
                  {generatingReport === 'GST' ? <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> : null}
                  GST Report
                </Button>
              </div>
            </div>

            <Table>
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Period</th>
                  <th>Status</th>
                  <th>Generated At</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {reports.map((report) => (
                  <tr key={report.id}>
                    <td>
                      <div className="flex items-center gap-2">
                        {getTypeIcon(report.type)}
                        <span className="font-medium">{report.type}</span>
                      </div>
                    </td>
                    <td>{report.period}</td>
                    <td>
                      <Badge className={getStatusColor(report.status)}>
                        {report.status}
                      </Badge>
                    </td>
                    <td>{report.generatedAt ? new Date(report.generatedAt).toLocaleString() : '-'}</td>
                    <td>
                      {report.status === 'READY' && report.data && (
                        <Button size="sm" variant="outline" onClick={() => handleDownloadReport(report)}>
                          <Download className="w-4 h-4 mr-2" />
                          Download
                        </Button>
                      )}
                      {report.status === 'PENDING' && (
                        <Button 
                          size="sm" 
                          variant="outline"
                          onClick={() => handleGenerateReport(report.type)}
                        >
                          Generate
                        </Button>
                      )}
                      {report.status === 'GENERATING' && (
                        <Button size="sm" variant="outline" disabled>
                          <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                          Generating...
                        </Button>
                      )}
                      {report.status === 'FAILED' && (
                        <Button 
                          size="sm" 
                          variant="outline"
                          onClick={() => handleGenerateReport(report.type)}
                        >
                          Retry
                        </Button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </Table>
          </Card>
        </TabsContent>

        {/* Deadlines Tab */}
        <TabsContent value="deadlines">
          <Card className="p-6">
            <h2 className="text-lg font-semibold mb-4">Compliance Deadlines</h2>
            <Table>
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Description</th>
                  <th>Due Date</th>
                  <th>Status</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {deadlines.map((deadline) => (
                  <tr key={deadline.id}>
                    <td>
                      <div className="flex items-center gap-2">
                        {getTypeIcon(deadline.type)}
                        <span className="font-medium">{deadline.type}</span>
                      </div>
                    </td>
                    <td>{deadline.description}</td>
                    <td className={deadline.status === 'OVERDUE' ? 'text-red-600 font-medium' : ''}>
                      {new Date(deadline.dueDate).toLocaleDateString()}
                    </td>
                    <td>
                      <Badge className={getStatusColor(deadline.status)}>
                        {deadline.status}
                      </Badge>
                    </td>
                    <td>
                      {deadline.status !== 'COMPLETED' && (
                        <Button size="sm" variant="outline">
                          Take Action
                        </Button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </Table>
          </Card>
        </TabsContent>

        {/* Audit Trail Tab */}
        <TabsContent value="audit-trail">
          <Card className="p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">Immutable Audit Trail</h2>
              <Button variant="outline" size="sm" onClick={handleExportAuditLog}>
                <Download className="w-4 h-4 mr-2" />
                Export Audit Log
              </Button>
            </div>
            <div className="bg-gray-50 rounded-lg p-4">
              <p className="text-sm text-gray-600 mb-4">
                All transactions are cryptographically secured and immutable. 
                Each entry includes device fingerprint, timestamp, and hash chain validation.
              </p>
              {auditLogs.length > 0 ? (
                <Table>
                  <thead>
                    <tr>
                      <th>Timestamp</th>
                      <th>Action</th>
                      <th>Resource</th>
                      <th>Actor</th>
                      <th>Hash</th>
                    </tr>
                  </thead>
                  <tbody>
                    {auditLogs.map((log, idx) => (
                      <tr key={idx}>
                        <td>{log.timestamp}</td>
                        <td>{log.action}</td>
                        <td>{log.resource}</td>
                        <td>{log.actor}</td>
                        <td className="font-mono text-xs">{log.hash}</td>
                      </tr>
                    ))}
                  </tbody>
                </Table>
              ) : (
                <p className="text-sm text-gray-500">No audit logs available</p>
              )}
            </div>
          </Card>
        </TabsContent>

        {/* Consent Management Tab */}
        <TabsContent value="consent">
          <Card className="p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">Consent Management (DPDP 2023)</h2>
              <Button variant="outline" size="sm" onClick={handleExportConsent}>
                <Download className="w-4 h-4 mr-2" />
                Export Consents
              </Button>
            </div>
            <p className="text-sm text-gray-600 mb-4">
              Manage data sharing permissions as per Digital Personal Data Protection Act, 2023
            </p>
            
            <div className="space-y-4">
              {consents.length > 0 ? (
                consents.map((consent) => (
                  <div key={consent.id} className="flex items-center justify-between p-4 border rounded-lg">
                    <div>
                      <h3 className="font-medium">{consent.purpose}</h3>
                      <p className="text-sm text-gray-600">
                        Status: {consent.granted ? (consent.revoked_at ? 'Revoked' : 'Active') : 'Not Granted'}
                      </p>
                      <p className="text-xs text-gray-500">Granted: {new Date(consent.created_at).toLocaleString()}</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input 
                        type="checkbox" 
                        className="sr-only peer" 
                        checked={consent.granted && !consent.revoked_at}
                        onChange={() => handleToggleConsent(consent)}
                        disabled={loading}
                      />
                      <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                    </label>
                  </div>
                ))
              ) : (
                <div className="text-center py-8">
                  <p className="text-gray-500">No consent records found</p>
                  <p className="text-sm text-gray-400 mt-2">Consent will be recorded when you first use features requiring data access</p>
                </div>
              )}
            </div>

            <div className="mt-6 p-4 bg-blue-50 rounded-lg">
              <h4 className="font-medium text-blue-900 mb-2">Your Rights under DPDP 2023</h4>
              <ul className="text-sm text-blue-800 space-y-1">
                <li>• Right to access your personal data</li>
                <li>• Right to correction and erasure</li>
                <li>• Right to grievance redressal</li>
                <li>• Right to nominate (in case of death/incapacity)</li>
              </ul>
              <Button className="mt-4" variant="outline" size="sm" onClick={handleExportConsent}>
                Request Data Export
              </Button>
            </div>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
