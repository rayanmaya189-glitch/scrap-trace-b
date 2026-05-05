package com.btrace.data.domain

data class MaterialPassport(
    val id: String,
    val batchId: String,
    val materialType: String,
    val weight: Double,
    val originSupplierId: String,
    val currentOwnerId: String?,
    val status: MaterialStatus,
    val cbamData: CbamData?,
    val carbonIntensity: Double?,
    val recycledContent: Double?,
    val createdAt: Long,
    val updatedAt: Long,
    val hashCurrent: String,
    val hashPrev: String?
)

enum class MaterialStatus {
    CREATED,
    IN_TRANSIT,
    DELIVERED,
    DISPUTED,
    VERIFIED
}

data class CbamData(
    val emissionFactor: Double,
    val totalEmissions: Double,
    val reportingPeriod: String
)

data class SupplierProfile(
    val id: String,
    val userId: String,
    val businessName: String,
    val gstin: String?,
    val pan: String?,
    val role: SupplierRole,
    val isVerified: Boolean,
    val icsScore: Double?,
    val creditLimit: Double?,
    val kycDocuments: List<String>,
    val createdAt: Long
)

enum class SupplierRole {
    DEALER,
    BUYER,
    EXPORTER,
    NBFC,
    AUDITOR
}

data class DigitalHandshake(
    val id: String,
    val materialId: String,
    val fromSupplierId: String,
    val toSupplierId: String,
    val initiatorSignature: String,
    val recipientSignature: String?,
    val hashCurrent: String,
    val hashPrev: String?,
    val deviceFingerprint: String,
    val timestamp: Long,
    val status: HandshakeStatus,
    val disputeReason: String?,
    val evidenceUrls: List<String>
)

enum class HandshakeStatus {
    PENDING,
    CONFIRMED,
    DISPUTED,
    REJECTED
}

data class ConsentRecord(
    val id: String,
    val userId: String,
    val purpose: String,
    val grantedTo: String,
    val isGranted: Boolean,
    val expiresAt: Long?,
    val createdAt: Long,
    val revokedAt: Long?
)

data class ComplianceReport(
    val id: String,
    val reportType: ReportType,
    val periodStart: Long,
    val periodEnd: Long,
    val generatedAt: Long,
    val data: Map<String, Any>,
    val downloadUrl: String?
)

enum class ReportType {
    CBAM,
    EPR,
    GST,
    AUDIT_TRAIL
}
