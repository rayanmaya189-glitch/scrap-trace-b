import { apiClient } from './client';

export interface ReportData {
  report_type: string;
  generated_at: string;
  data: Record<string, unknown>;
  row_count: number;
}

export interface GenerateReportRequest {
  report_type: 'cbam' | 'epr' | 'gst_audit' | 'consent_export';
  supplier_id?: string;
  start_date?: string;
  end_date?: string;
  format?: 'json' | 'csv';
}

export interface ReportApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
  timestamp: string;
}

export const reportApi = {
  /**
   * Generate a compliance report
   */
  generateReport: async (request: GenerateReportRequest): Promise<ReportData> => {
    const response = await apiClient.post<ReportApiResponse<ReportData>>(
      '/reports/generate',
      request
    );
    return response.data.data;
  },

  /**
   * Generate CBAM report
   */
  generateCbamReport: async (
    supplierId?: string,
    startDate?: string,
    endDate?: string
  ): Promise<ReportData> => {
    return reportApi.generateReport({
      report_type: 'cbam',
      supplier_id: supplierId,
      start_date: startDate,
      end_date: endDate,
      format: 'json',
    });
  },

  /**
   * Generate EPR report
   */
  generateEprReport: async (
    supplierId?: string,
    startDate?: string,
    endDate?: string
  ): Promise<ReportData> => {
    return reportApi.generateReport({
      report_type: 'epr',
      supplier_id: supplierId,
      start_date: startDate,
      end_date: endDate,
      format: 'json',
    });
  },

  /**
   * Generate GST audit trail report
   */
  generateGstAuditReport: async (
    supplierId?: string,
    startDate?: string,
    endDate?: string
  ): Promise<ReportData> => {
    return reportApi.generateReport({
      report_type: 'gst_audit',
      supplier_id: supplierId,
      start_date: startDate,
      end_date: endDate,
      format: 'json',
    });
  },

  /**
   * Generate consent export for DPDP compliance
   */
  generateConsentExport: async (
    supplierId?: string,
    startDate?: string,
    endDate?: string
  ): Promise<ReportData> => {
    return reportApi.generateReport({
      report_type: 'consent_export',
      supplier_id: supplierId,
      start_date: startDate,
      end_date: endDate,
      format: 'json',
    });
  },

  /**
   * Export report as CSV
   */
  exportAsCsv: async (reportType: string, reportData: Record<string, unknown>): Promise<string> => {
    // Convert JSON data to CSV format
    const rows: string[] = [];
    
    if (Array.isArray(reportData['batches']) || Array.isArray(reportData['transactions']) || Array.isArray(reportData['consents'])) {
      const items = reportData['batches'] || reportData['transactions'] || reportData['consents'];
      if (items.length === 0) return '';
      
      const headers = Object.keys(items[0]);
      rows.push(headers.join(','));
      
      items.forEach((item: Record<string, unknown>) => {
        const values = headers.map(header => {
          const value = item[header];
          if (value === null || value === undefined) return '';
          const stringValue = String(value);
          // Escape quotes and wrap in quotes if contains comma
          if (stringValue.includes(',') || stringValue.includes('"')) {
            return `"${stringValue.replace(/"/g, '""')}"`;
          }
          return stringValue;
        });
        rows.push(values.join(','));
      });
    }
    
    return rows.join('\n');
  },

  /**
   * Download report as file
   */
  downloadReport: async (reportData: ReportData, format: 'json' | 'csv' = 'json'): Promise<void> => {
    let content: string;
    let mimeType: string;
    let extension: string;

    if (format === 'csv') {
      content = await reportApi.exportAsCsv(reportData.report_type, reportData.data);
      mimeType = 'text/csv';
      extension = 'csv';
    } else {
      content = JSON.stringify(reportData.data, null, 2);
      mimeType = 'application/json';
      extension = 'json';
    }

    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${reportData.report_type}_report_${new Date().toISOString().split('T')[0]}.${extension}`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  },
};
