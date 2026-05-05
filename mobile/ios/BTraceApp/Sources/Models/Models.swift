//
//  Models.swift
//  BTraceApp
//
//  Core domain models for B-Trace Protocol
//

import Foundation

// MARK: - Material Passport

struct MaterialPassport: Identifiable, Codable, Hashable {
    let id: String
    let batchId: String
    let materialType: String
    let weight: Double
    let originSupplierId: String
    let currentOwnerId: String?
    let status: MaterialStatus
    let cbamData: CbamData?
    let carbonIntensity: Double?
    let recycledContent: Double?
    let createdAt: Date
    let updatedAt: Date
    let hashCurrent: String
    let hashPrev: String?
    
    enum CodingKeys: String, CodingKey {
        case id, batchId = "batch_id", materialType = "material_type", weight
        case originSupplierId = "origin_supplier_id", currentOwnerId = "current_owner_id"
        case status, cbamData = "cbam_data", carbonIntensity = "carbon_intensity"
        case recycledContent = "recycled_content", createdAt = "created_at"
        case updatedAt = "updated_at", hashCurrent = "hash_current", hashPrev = "hash_prev"
    }
}

enum MaterialStatus: String, Codable, CaseIterable {
    case created = "CREATED"
    case inTransit = "IN_TRANSIT"
    case delivered = "DELIVERED"
    case disputed = "DISPUTED"
    case verified = "VERIFIED"
    
    var displayName: String {
        switch self {
        case .created: return "Created"
        case .inTransit: return "In Transit"
        case .delivered: return "Delivered"
        case .disputed: return "Disputed"
        case .verified: return "Verified"
        }
    }
    
    var color: String {
        switch self {
        case .created: return "blue"
        case .inTransit: return "orange"
        case .delivered: return "green"
        case .disputed: return "red"
        case .verified: return "purple"
        }
    }
}

struct CbamData: Codable, Hashable {
    let emissionFactor: Double
    let totalEmissions: Double
    let reportingPeriod: String
    
    enum CodingKeys: String, CodingKey {
        case emissionFactor = "emission_factor"
        case totalEmissions = "total_emissions"
        case reportingPeriod = "reporting_period"
    }
}

// MARK: - Supplier Profile

struct SupplierProfile: Identifiable, Codable, Hashable {
    let id: String
    let userId: String
    let businessName: String
    let gstin: String?
    let pan: String?
    let role: SupplierRole
    let isVerified: Bool
    let icsScore: Double?
    let creditLimit: Double?
    let kycDocuments: [String]
    let createdAt: Date
    
    enum CodingKeys: String, CodingKey {
        case id, userId = "user_id", businessName = "business_name"
        case gstin, pan, role, isVerified = "is_verified"
        case icsScore = "ics_score", creditLimit = "credit_limit"
        case kycDocuments = "kyc_documents", createdAt = "created_at"
    }
}

enum SupplierRole: String, Codable, CaseIterable {
    case dealer = "DEALER"
    case buyer = "BUYER"
    case exporter = "EXPORTER"
    case nbfc = "NBFC"
    case auditor = "AUDITOR"
    
    var displayName: String {
        rawValue.capitalized
    }
}

// MARK: - Digital Handshake

struct DigitalHandshake: Identifiable, Codable, Hashable {
    let id: String
    let materialId: String
    let fromSupplierId: String
    let toSupplierId: String
    let initiatorSignature: String
    let recipientSignature: String?
    let hashCurrent: String
    let hashPrev: String?
    let deviceFingerprint: String
    let timestamp: Date
    let status: HandshakeStatus
    let disputeReason: String?
    let evidenceUrls: [String]
    
    enum CodingKeys: String, CodingKey {
        case id, materialId = "material_id"
        case fromSupplierId = "from_supplier_id", toSupplierId = "to_supplier_id"
        case initiatorSignature = "initiator_signature"
        case recipientSignature = "recipient_signature"
        case hashCurrent = "hash_current", hashPrev = "hash_prev"
        case deviceFingerprint = "device_fingerprint", timestamp
        case status, disputeReason = "dispute_reason"
        case evidenceUrls = "evidence_urls"
    }
}

enum HandshakeStatus: String, Codable, CaseIterable {
    case pending = "PENDING"
    case confirmed = "CONFIRMED"
    case disputed = "DISPUTED"
    case rejected = "REJECTED"
    
    var displayName: String {
        switch self {
        case .pending: return "Pending"
        case .confirmed: return "Confirmed"
        case .disputed: return "Disputed"
        case .rejected: return "Rejected"
        }
    }
    
    var color: String {
        switch self {
        case .pending: return "orange"
        case .confirmed: return "green"
        case .disputed: return "red"
        case .rejected: return "gray"
        }
    }
}

// MARK: - Consent Record

struct ConsentRecord: Identifiable, Codable, Hashable {
    let id: String
    let userId: String
    let purpose: String
    let grantedTo: String
    let isGranted: Bool
    let expiresAt: Date?
    let createdAt: Date
    let revokedAt: Date?
    
    enum CodingKeys: String, CodingKey {
        case id, userId = "user_id", purpose, grantedTo = "granted_to"
        case isGranted = "is_granted", expiresAt = "expires_at"
        case createdAt = "created_at", revokedAt = "revoked_at"
    }
}

// MARK: - Compliance Report

struct ComplianceReport: Identifiable, Codable, Hashable {
    let id: String
    let reportType: ReportType
    let periodStart: Date
    let periodEnd: Date
    let generatedAt: Date
    let data: [String: AnyCodable]
    let downloadUrl: String?
    
    enum CodingKeys: String, CodingKey {
        case id, reportType = "report_type"
        case periodStart = "period_start", periodEnd = "period_end"
        case generatedAt = "generated_at", data
        case downloadUrl = "download_url"
    }
}

enum ReportType: String, Codable, CaseIterable {
    case cbam = "CBAM"
    case epr = "EPR"
    case gst = "GST"
    case auditTrail = "AUDIT_TRAIL"
    
    var displayName: String {
        rawValue
    }
}

// MARK: - Helper Types

struct AnyCodable: Codable {
    let value: Any
    
    init(_ value: Any) {
        self.value = value
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        
        if container.decodeNil() {
            value = NSNull()
        } else if let bool = try? container.decode(Bool.self) {
            value = bool
        } else if let int = try? container.decode(Int.self) {
            value = int
        } else if let double = try? container.decode(Double.self) {
            value = double
        } else if let string = try? container.decode(String.self) {
            value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            value = array.map { $0.value }
        } else if let dictionary = try? container.decode([String: AnyCodable].self) {
            value = dictionary.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unable to decode value"
            )
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        
        switch value {
        case is NSNull:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map { AnyCodable($0) })
        case let dictionary as [String: Any]:
            try container.encode(dictionary.mapValues { AnyCodable($0) })
        default:
            throw EncodingError.invalidValue(
                value,
                EncodingError.Context(
                    codingPath: encoder.codingPath,
                    debugDescription: "Unable to encode value"
                )
            )
        }
    }
}
