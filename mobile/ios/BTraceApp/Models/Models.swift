import Foundation

// MARK: - Base Response Models
struct BaseResponse<T: Codable>: Codable {
    let success: Bool
    let message: String?
    let data: T?
}

struct PagedResponse<T: Codable>: Codable {
    let items: [T]
    let total: Int
    let page: Int
    let limit: Int
}

// MARK: - Auth Models
struct OtpRequest: Codable {
    let phoneNumber: String
    
    enum CodingKeys: String, CodingKey {
        case phoneNumber = "phone_number"
    }
}

struct VerifyOtpRequest: Codable {
    let phoneNumber: String
    let otp: String
    let deviceFingerprint: String
    
    enum CodingKeys: String, CodingKey {
        case phoneNumber = "phone_number"
        case otp
        case deviceFingerprint = "device_fingerprint"
    }
}

struct RefreshTokenRequest: Codable {
    let refreshToken: String
    
    enum CodingKeys: String, CodingKey {
        case refreshToken = "refresh_token"
    }
}

struct AuthResponse: Codable {
    let accessToken: String
    let refreshToken: String
    let expiresIn: Int
    let user: SupplierProfile?
    
    enum CodingKeys: String, CodingKey {
        case accessToken = "access_token"
        case refreshToken = "refresh_token"
        case expiresIn = "expires_in"
        case user
    }
}

// MARK: - Supplier Profile Models
struct SupplierProfile: Codable {
    let id: String
    let name: String
    let phoneNumber: String
    let email: String?
    let role: String
    let gstNumber: String?
    let panNumber: String?
    let address: String?
    let city: String?
    let state: String?
    let pincode: String?
    let isVerified: Bool
    let kycStatus: String?
    let createdAt: String
    let updatedAt: String
    
    enum CodingKeys: String, CodingKey {
        case id, name
        case phoneNumber = "phone_number"
        case email, role
        case gstNumber = "gst_number"
        case panNumber = "pan_number"
        case address, city, state, pincode
        case isVerified = "is_verified"
        case kycStatus = "kyc_status"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct UpdateSupplierRequest: Codable {
    let name: String?
    let email: String?
    let gstNumber: String?
    let panNumber: String?
    let address: String?
    let city: String?
    let state: String?
    let pincode: String?
    
    enum CodingKeys: String, CodingKey {
        case name, email
        case gstNumber = "gst_number"
        case panNumber = "pan_number"
        case address, city, state, pincode
    }
}

// MARK: - Material Passport Models
struct MaterialPassport: Codable, Identifiable {
    let id: String
    let batchId: String
    let supplierId: String
    let supplierName: String
    let materialType: String
    let grade: String?
    let quantityKg: Double
    let unit: String
    let status: String
    let currentOwnerId: String
    let cbamCategory: String?
    let carbonIntensity: Double?
    let recycledContentPct: Double?
    let emissionFactor: Double?
    let totalEmissionsKgCo2: Double?
    let manufacturingDate: String?
    let createdAt: String
    let updatedAt: String
    let handshakeHistory: [DigitalHandshake]
    
    enum CodingKeys: String, CodingKey {
        case id
        case batchId = "batch_id"
        case supplierId = "supplier_id"
        case supplierName = "supplier_name"
        case materialType = "material_type"
        case grade, quantityKg = "quantity_kg", unit, status
        case currentOwnerId = "current_owner_id"
        case cbamCategory = "cbam_category"
        case carbonIntensity = "carbon_intensity"
        case recycledContentPct = "recycled_content_pct"
        case emissionFactor = "emission_factor"
        case totalEmissionsKgCo2 = "total_emissions_kg_co2"
        case manufacturingDate = "manufacturing_date"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case handshakeHistory = "handshake_history"
    }
}

struct CreateMaterialRequest: Codable {
    let batchId: String
    let materialType: String
    let grade: String?
    let quantityKg: Double
    let unit: String
    let cbamCategory: String?
    let carbonIntensity: Double?
    let recycledContentPct: Double?
    
    enum CodingKeys: String, CodingKey {
        case batchId = "batch_id"
        case materialType = "material_type"
        case grade
        case quantityKg = "quantity_kg"
        case unit
        case cbamCategory = "cbam_category"
        case carbonIntensity = "carbon_intensity"
        case recycledContentPct = "recycled_content_pct"
    }
}

struct MaterialSummary: Codable {
    let totalMaterials: Int
    let totalWeightKg: Double
    let byStatus: [String: Int]
    let byType: [String: Int]
    
    enum CodingKeys: String, CodingKey {
        case totalMaterials = "total_materials"
        case totalWeightKg = "total_weight_kg"
        case byStatus = "by_status"
        case byType = "by_type"
    }
}

// MARK: - Digital Handshake Models
struct DigitalHandshake: Codable, Identifiable {
    let id: String
    let materialId: String
    let initiatorId: String
    let initiatorName: String
    let receiverId: String
    let receiverName: String
    let status: String
    let initiatedAt: String
    let confirmedAt: String?
    let hashPrev: String?
    let hashCurrent: String
    let signatureInitiator: String?
    let signatureReceiver: String?
    let disputeReason: String?
    let evidenceUrls: [String]?
    
    enum CodingKeys: String, CodingKey {
        case id
        case materialId = "material_id"
        case initiatorId = "initiator_id"
        case initiatorName = "initiator_name"
        case receiverId = "receiver_id"
        case receiverName = "receiver_name"
        case status
        case initiatedAt = "initiated_at"
        case confirmedAt = "confirmed_at"
        case hashPrev = "hash_prev"
        case hashCurrent = "hash_current"
        case signatureInitiator = "signature_initiator"
        case signatureReceiver = "signature_receiver"
        case disputeReason = "dispute_reason"
        case evidenceUrls = "evidence_urls"
    }
}

struct InitiateHandshakeRequest: Codable {
    let materialId: String
    let receiverId: String
    let initiatorSignature: String
    let deviceFingerprint: String
    
    enum CodingKeys: String, CodingKey {
        case materialId = "material_id"
        case receiverId = "receiver_id"
        case initiatorSignature = "initiator_signature"
        case deviceFingerprint = "device_fingerprint"
    }
}

struct ConfirmHandshakeRequest: Codable {
    let handshakeId: String
    let receiverSignature: String
    let deviceFingerprint: String
    
    enum CodingKeys: String, CodingKey {
        case handshakeId = "handshake_id"
        case receiverSignature = "receiver_signature"
        case deviceFingerprint = "device_fingerprint"
    }
}

struct DisputeRequest: Codable {
    let handshakeId: String
    let reason: String
    let evidenceUrls: [String]
    let deviceFingerprint: String
    
    enum CodingKeys: String, CodingKey {
        case handshakeId = "handshake_id"
        case reason
        case evidenceUrls = "evidence_urls"
        case deviceFingerprint = "device_fingerprint"
    }
}

// MARK: - Scoring Models
struct ScoringOutput: Codable {
    let supplierId: String
    let icsScore: Int
    let probabilityOfDefault: Double
    let stabilityIndex: Double
    let creditLimitRecommendation: Double
    let riskCategory: String
    let factors: [ScoreFactor]
    let calculatedAt: String
    
    enum CodingKeys: String, CodingKey {
        case supplierId = "supplier_id"
        case icsScore = "ics_score"
        case probabilityOfDefault = "probability_of_default"
        case stabilityIndex = "stability_index"
        case creditLimitRecommendation = "credit_limit_recommendation"
        case riskCategory = "risk_category"
        case factors
        case calculatedAt = "calculated_at"
    }
}

struct ScoreFactor: Codable {
    let name: String
    let weight: Double
    let score: Double
    let impact: String
}

// MARK: - Consent Models
struct ConsentLog: Codable, Identifiable {
    let id: String
    let userId: String
    let grantedTo: String
    let purpose: String
    let dataTypes: [String]
    let isActive: Bool
    let grantedAt: String
    let revokedAt: String?
    
    enum CodingKeys: String, CodingKey {
        case id
        case userId = "user_id"
        case grantedTo = "granted_to"
        case purpose
        case dataTypes = "data_types"
        case isActive = "is_active"
        case grantedAt = "granted_at"
        case revokedAt = "revoked_at"
    }
}

struct CreateConsentRequest: Codable {
    let grantedTo: String
    let purpose: String
    let dataTypes: [String]
    
    enum CodingKeys: String, CodingKey {
        case grantedTo = "granted_to"
        case purpose
        case dataTypes = "data_types"
    }
}

// MARK: - Report Models
struct GenerateReportRequest: Codable {
    let reportType: String
    let startDate: String
    let endDate: String
    let format: String
    
    enum CodingKeys: String, CodingKey {
        case reportType = "report_type"
        case startDate = "start_date"
        case endDate = "end_date"
        case format
    }
}

struct ReportData: Codable {
    let reportId: String
    let reportType: String
    let downloadUrl: String
    let generatedAt: String
    
    enum CodingKeys: String, CodingKey {
        case reportId = "report_id"
        case reportType = "report_type"
        case downloadUrl = "download_url"
        case generatedAt = "generated_at"
    }
}

// MARK: - Upload Models
struct UploadResponse: Codable {
    let fileId: String
    let fileName: String
    let fileUrl: String
    let fileSize: Int
    let uploadedAt: String
    
    enum CodingKeys: String, CodingKey {
        case fileId = "file_id"
        case fileName = "file_name"
        case fileUrl = "file_url"
        case fileSize = "file_size"
        case uploadedAt = "uploaded_at"
    }
}
