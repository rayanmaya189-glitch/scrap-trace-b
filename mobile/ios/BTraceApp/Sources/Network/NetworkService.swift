//
//  NetworkService.swift
//  BTraceApp
//
//  API client for B-Trace backend communication
//

import Foundation
import Combine

class NetworkService: ObservableObject {
    static let shared = NetworkService()
    
    private let session: URLSession
    private let baseURL: URL
    @Published private(set) var isAuthenticated: Bool = false
    
    init() {
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 30
        config.timeoutIntervalForResource = 300
        self.session = URLSession(configuration: config)
        self.baseURL = URL(string: "https://api.btrace.io")!
        
        // Check existing auth state
        isAuthenticated = AccessTokenStore.shared.accessToken != nil
    }
    
    // MARK: - Authentication
    
    func requestOTP(phone: String) async throws {
        let endpoint = baseURL.appendingPathComponent("/v1/auth/request-otp")
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let body = ["phone": phone]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
    }
    
    func verifyOTP(phone: String, otp: String) async throws -> AuthResponse {
        let endpoint = baseURL.appendingPathComponent("/v1/auth/verify-otp")
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let body = ["phone": phone, "otp": otp]
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        let authResponse = try JSONDecoder().decode(AuthResponse.self, from: data)
        AccessTokenStore.shared.save(tokens: authResponse)
        isAuthenticated = true
        
        return authResponse
    }
    
    func refreshToken() async throws -> AuthResponse {
        guard let refreshToken = AccessTokenStore.shared.refreshToken else {
            throw NetworkError.unauthorized
        }
        
        let endpoint = baseURL.appendingPathComponent("/v1/auth/refresh")
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("Bearer \(refreshToken)", forHTTPHeaderField: "Authorization")
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        let authResponse = try JSONDecoder().decode(AuthResponse.self, from: data)
        AccessTokenStore.shared.save(tokens: authResponse)
        
        return authResponse
    }
    
    func logout() async throws {
        guard let accessToken = AccessTokenStore.shared.accessToken else { return }
        
        let endpoint = baseURL.appendingPathComponent("/v1/auth/logout")
        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("Bearer \(accessToken)", forHTTPHeaderField: "Authorization")
        
        let (_, response) = try await session.data(for: request)
        try handleResponse(nil, response)
        
        AccessTokenStore.shared.clear()
        isAuthenticated = false
    }
    
    // MARK: - Materials
    
    func fetchMaterials() async throws -> [MaterialPassport] {
        let endpoint = baseURL.appendingPathComponent("/v1/materials")
        var request = try authenticatedRequest(url: endpoint)
        request.httpMethod = "GET"
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        let result = try JSONDecoder().decode(MaterialListResponse.self, from: data)
        return result.materials
    }
    
    func createMaterial(_ material: CreateMaterialRequest) async throws -> MaterialPassport {
        let endpoint = baseURL.appendingPathComponent("/v1/materials")
        var request = try authenticatedRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(material)
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode(MaterialPassport.self, from: data)
    }
    
    // MARK: - Handshakes
    
    func initiateHandshake(_ request: HandshakeInitRequest) async throws -> DigitalHandshake {
        let endpoint = baseURL.appendingPathComponent("/v1/handshakes/initiate")
        var urlRequest = try authenticatedRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(request)
        
        let (data, response) = try await session.data(for: urlRequest)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode(DigitalHandshake.self, from: data)
    }
    
    func confirmHandshake(_ request: HandshakeConfirmRequest) async throws -> DigitalHandshake {
        let endpoint = baseURL.appendingPathComponent("/v1/handshakes/confirm")
        var urlRequest = try authenticatedRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(request)
        
        let (data, response) = try await session.data(for: urlRequest)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode(DigitalHandshake.self, from: data)
    }
    
    func raiseDispute(_ request: DisputeRequest) async throws -> DigitalHandshake {
        let endpoint = baseURL.appendingPathComponent("/v1/handshakes/dispute")
        var urlRequest = try authenticatedRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(request)
        
        let (data, response) = try await session.data(for: urlRequest)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode(DigitalHandshake.self, from: data)
    }
    
    func fetchHandshakes() async throws -> [DigitalHandshake] {
        let endpoint = baseURL.appendingPathComponent("/v1/handshakes")
        var request = try authenticatedRequest(url: endpoint)
        request.httpMethod = "GET"
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        let result = try JSONDecoder().decode([DigitalHandshake].self, from: data)
        return result
    }
    
    // MARK: - Consent
    
    func fetchConsents() async throws -> [ConsentRecord] {
        let endpoint = baseURL.appendingPathComponent("/v1/consent/my")
        var request = try authenticatedRequest(url: endpoint)
        request.httpMethod = "GET"
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode([ConsentRecord].self, from: data)
    }
    
    func grantConsent(_ request: ConsentRequest) async throws -> ConsentRecord {
        let endpoint = baseURL.appendingPathComponent("/v1/consent")
        var urlRequest = try authenticatedRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(request)
        
        let (data, response) = try await session.data(for: urlRequest)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode(ConsentRecord.self, from: data)
    }
    
    func revokeConsent(id: String) async throws {
        let endpoint = baseURL.appendingPathComponent("/v1/consent/\(id)/revoke")
        var request = try authenticatedRequest(url: endpoint)
        request.httpMethod = "POST"
        
        let (_, response) = try await session.data(for: request)
        try handleResponse(nil, response)
    }
    
    // MARK: - Reports
    
    func generateReport(_ request: ReportGenerationRequest) async throws -> ComplianceReport {
        let endpoint = baseURL.appendingPathComponent("/v1/reports/generate")
        var urlRequest = try authenticatedRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(request)
        
        let (data, response) = try await session.data(for: urlRequest)
        try handleResponse(data, response)
        
        return try JSONDecoder().decode(ComplianceReport.self, from: data)
    }
    
    // MARK: - Upload
    
    func uploadEvidence(fileData: Data, fileName: String, mimeType: String) async throws -> String {
        let endpoint = baseURL.appendingPathComponent("/v1/upload/evidence")
        var request = try authenticatedRequest(url: endpoint)
        request.httpMethod = "POST"
        
        let boundary = UUID().uuidString
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        
        var body = Data()
        body.append("--\(boundary)\r\n".data(using: .utf8)!)
        body.append("Content-Disposition: form-data; name=\"file\"; filename=\"\(fileName)\"\r\n".data(using: .utf8)!)
        body.append("Content-Type: \(mimeType)\r\n\r\n".data(using: .utf8)!)
        body.append(fileData)
        body.append("\r\n--\(boundary)--\r\n".data(using: .utf8)!)
        
        request.httpBody = body
        
        let (data, response) = try await session.data(for: request)
        try handleResponse(data, response)
        
        let result = try JSONDecoder().decode(UploadResponse.self, from: data)
        return result.fileUrl
    }
    
    // MARK: - Helpers
    
    private func authenticatedRequest(url: URL) throws -> URLRequest {
        guard let token = AccessTokenStore.shared.accessToken else {
            throw NetworkError.unauthorized
        }
        
        var request = URLRequest(url: url)
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        return request
    }
    
    private func handleResponse(_ data: Data?, _ response: URLResponse) throws {
        guard let httpResponse = response as? HTTPURLResponse else {
            throw NetworkError.invalidResponse
        }
        
        switch httpResponse.statusCode {
        case 200..<300:
            return
        case 401:
            throw NetworkError.unauthorized
        case 403:
            throw NetworkError.forbidden
        case 404:
            throw NetworkError.notFound
        case 500..<600:
            throw NetworkError.serverError
        default:
            throw NetworkError.unknown
        }
    }
}

// MARK: - Models

struct AuthResponse: Codable {
    let accessToken: String
    let refreshToken: String
    let expiresIn: Int
    let user: UserInfo
    
    enum CodingKeys: String, CodingKey {
        case accessToken = "access_token"
        case refreshToken = "refresh_token"
        case expiresIn = "expires_in"
        case user
    }
}

struct UserInfo: Codable {
    let id: String
    let phone: String
    let role: String
    let isVerified: Bool
    
    enum CodingKeys: String, CodingKey {
        case id, phone, role
        case isVerified = "is_verified"
    }
}

struct MaterialListResponse: Codable {
    let materials: [MaterialPassport]
    let total: Int
}

struct CreateMaterialRequest: Codable {
    let batchId: String
    let materialType: String
    let weight: Double
    let cbamData: CbamDataDto?
    let carbonIntensity: Double?
    let recycledContent: Double?
    
    enum CodingKeys: String, CodingKey {
        case batchId = "batch_id"
        case materialType = "material_type"
        case weight
        case cbamData = "cbam_data"
        case carbonIntensity = "carbon_intensity"
        case recycledContent = "recycled_content"
    }
}

struct CbamDataDto: Codable {
    let emissionFactor: Double
    let reportingPeriod: String
    
    enum CodingKeys: String, CodingKey {
        case emissionFactor = "emission_factor"
        case reportingPeriod = "reporting_period"
    }
}

struct HandshakeInitRequest: Codable {
    let materialId: String
    let recipientId: String
    let initiatorSignature: String
    let deviceFingerprint: String
    
    enum CodingKeys: String, CodingKey {
        case materialId = "material_id"
        case recipientId = "recipient_id"
        case initiatorSignature = "initiator_signature"
        case deviceFingerprint = "device_fingerprint"
    }
}

struct HandshakeConfirmRequest: Codable {
    let handshakeId: String
    let recipientSignature: String
    let deviceFingerprint: String
    
    enum CodingKeys: String, CodingKey {
        case handshakeId = "handshake_id"
        case recipientSignature = "recipient_signature"
        case deviceFingerprint = "device_fingerprint"
    }
}

struct DisputeRequest: Codable {
    let handshakeId: String
    let reason: String
    let evidenceUrls: [String]
    
    enum CodingKeys: String, CodingKey {
        case handshakeId = "handshake_id"
        case reason
        case evidenceUrls = "evidence_urls"
    }
}

struct ConsentRequest: Codable {
    let purpose: String
    let grantedTo: String
    let expiresAt: Date?
    
    enum CodingKeys: String, CodingKey {
        case purpose, grantedTo = "granted_to"
        case expiresAt = "expires_at"
    }
}

struct ReportGenerationRequest: Codable {
    let reportType: String
    let periodStart: Date
    let periodEnd: Date
    
    enum CodingKeys: String, CodingKey {
        case reportType = "report_type"
        case periodStart = "period_start"
        case periodEnd = "period_end"
    }
}

struct UploadResponse: Codable {
    let success: Bool
    let fileUrl: String
    let fileName: String
    
    enum CodingKeys: String, CodingKey {
        case success, fileUrl = "file_url", fileName = "file_name"
    }
}

// MARK: - Token Storage

class AccessTokenStore {
    static let shared = AccessTokenStore()
    
    private let keychain = KeychainWrapper.standard
    private let accessKey = "btrace.access_token"
    private let refreshKey = "btrace.refresh_token"
    
    var accessToken: String? {
        keychain.string(forKey: accessKey)
    }
    
    var refreshToken: String? {
        keychain.string(forKey: refreshKey)
    }
    
    func save(tokens: AuthResponse) {
        keychain.set(tokens.accessToken, forKey: accessKey)
        keychain.set(tokens.refreshToken, forKey: refreshKey)
    }
    
    func clear() {
        keychain.removeObject(forKey: accessKey)
        keychain.removeObject(forKey: refreshKey)
    }
}

// MARK: - Errors

enum NetworkError: LocalizedError {
    case unauthorized
    case forbidden
    case notFound
    case serverError
    case invalidResponse
    case unknown
    
    var errorDescription: String? {
        switch self {
        case .unauthorized: return "Authentication required"
        case .forbidden: return "Access denied"
        case .notFound: return "Resource not found"
        case .serverError: return "Server error occurred"
        case .invalidResponse: return "Invalid response from server"
        case .unknown: return "An unknown error occurred"
        }
    }
}
