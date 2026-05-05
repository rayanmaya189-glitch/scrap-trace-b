// API Service exports
export { apiClient } from './client';
export { authApi } from './auth';
export type { AuthResponse, OtpRequest, OtpVerifyRequest } from './auth';

export { materialApi } from './material';
export type { MaterialPassport, CreateMaterialRequest } from './material';

export { handshakeApi } from './handshake';
export type { DigitalHandshake, InitiateHandshakeRequest, ConfirmHandshakeRequest, RaiseDisputeRequest } from './handshake';

export { consentApi } from './consent';
export type { ConsentRecord, CreateConsentRequest } from './consent';

export { reportApi } from './report';
export type { ReportData, GenerateReportRequest } from './report';
