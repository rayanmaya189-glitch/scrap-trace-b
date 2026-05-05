package com.btrace.app.data.model

import com.google.gson.annotations.SerializedName

// Base Response Models
data class BaseResponse<T>(
    @SerializedName("success") val success: Boolean,
    @SerializedName("message") val message: String?,
    @SerializedName("data") val data: T?
)

data class PagedResponse<T>(
    @SerializedName("items") val items: List<T>,
    @SerializedName("total") val total: Int,
    @SerializedName("page") val page: Int,
    @SerializedName("limit") val limit: Int
)

// Auth Models
data class OtpRequest(
    @SerializedName("phone_number") val phoneNumber: String
)

data class VerifyOtpRequest(
    @SerializedName("phone_number") val phoneNumber: String,
    @SerializedName("otp") val otp: String,
    @SerializedName("device_fingerprint") val deviceFingerprint: String
)

data class RefreshTokenRequest(
    @SerializedName("refresh_token") val refreshToken: String
)

data class AuthResponse(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("refresh_token") val refreshToken: String,
    @SerializedName("expires_in") val expiresIn: Long,
    @SerializedName("user") val user: SupplierProfile?
)

// Supplier Profile Models
data class SupplierProfile(
    @SerializedName("id") val id: String,
    @SerializedName("name") val name: String,
    @SerializedName("phone_number") val phoneNumber: String,
    @SerializedName("email") val email: String?,
    @SerializedName("role") val role: String,
    @SerializedName("gst_number") val gstNumber: String?,
    @SerializedName("pan_number") val panNumber: String?,
    @SerializedName("address") val address: String?,
    @SerializedName("city") val city: String?,
    @SerializedName("state") val state: String?,
    @SerializedName("pincode") val pincode: String?,
    @SerializedName("is_verified") val isVerified: Boolean,
    @SerializedName("kyc_status") val kycStatus: String?,
    @SerializedName("created_at") val createdAt: String,
    @SerializedName("updated_at") val updatedAt: String
)

data class UpdateSupplierRequest(
    @SerializedName("name") val name: String?,
    @SerializedName("email") val email: String?,
    @SerializedName("gst_number") val gstNumber: String?,
    @SerializedName("pan_number") val panNumber: String?,
    @SerializedName("address") val address: String?,
    @SerializedName("city") val city: String?,
    @SerializedName("state") val state: String?,
    @SerializedName("pincode") val pincode: String?
)

// Material Passport Models
data class MaterialPassport(
    @SerializedName("id") val id: String,
    @SerializedName("batch_id") val batchId: String,
    @SerializedName("supplier_id") val supplierId: String,
    @SerializedName("supplier_name") val supplierName: String,
    @SerializedName("material_type") val materialType: String,
    @SerializedName("grade") val grade: String?,
    @SerializedName("quantity_kg") val quantityKg: Double,
    @SerializedName("unit") val unit: String,
    @SerializedName("status") val status: String,
    @SerializedName("current_owner_id") val currentOwnerId: String,
    @SerializedName("cbam_category") val cbamCategory: String?,
    @SerializedName("carbon_intensity") val carbonIntensity: Double?,
    @SerializedName("recycled_content_pct") val recycledContentPct: Double?,
    @SerializedName("emission_factor") val emissionFactor: Double?,
    @SerializedName("total_emissions_kg_co2") val totalEmissionsKgCo2: Double?,
    @SerializedName("manufacturing_date") val manufacturingDate: String?,
    @SerializedName("created_at") val createdAt: String,
    @SerializedName("updated_at") val updatedAt: String,
    @SerializedName("handshake_history") val handshakeHistory: List<DigitalHandshake>
)

data class CreateMaterialRequest(
    @SerializedName("batch_id") val batchId: String,
    @SerializedName("material_type") val materialType: String,
    @SerializedName("grade") val grade: String?,
    @SerializedName("quantity_kg") val quantityKg: Double,
    @SerializedName("unit") val unit: String = "KG",
    @SerializedName("cbam_category") val cbamCategory: String?,
    @SerializedName("carbon_intensity") val carbonIntensity: Double?,
    @SerializedName("recycled_content_pct") val recycledContentPct: Double?
)

data class MaterialSummary(
    @SerializedName("total_materials") val totalMaterials: Int,
    @SerializedName("total_weight_kg") val totalWeightKg: Double,
    @SerializedName("by_status") val byStatus: Map<String, Int>,
    @SerializedName("by_type") val byType: Map<String, Int>
)

// Digital Handshake Models
data class DigitalHandshake(
    @SerializedName("id") val id: String,
    @SerializedName("material_id") val materialId: String,
    @SerializedName("initiator_id") val initiatorId: String,
    @SerializedName("initiator_name") val initiatorName: String,
    @SerializedName("receiver_id") val receiverId: String,
    @SerializedName("receiver_name") val receiverName: String,
    @SerializedName("status") val status: String,
    @SerializedName("initiated_at") val initiatedAt: String,
    @SerializedName("confirmed_at") val confirmedAt: String?,
    @SerializedName("hash_prev") val hashPrev: String?,
    @SerializedName("hash_current") val hashCurrent: String,
    @SerializedName("signature_initiator") val signatureInitiator: String?,
    @SerializedName("signature_receiver") val signatureReceiver: String?,
    @SerializedName("dispute_reason") val disputeReason: String?,
    @SerializedName("evidence_urls") val evidenceUrls: List<String>?
)

data class InitiateHandshakeRequest(
    @SerializedName("material_id") val materialId: String,
    @SerializedName("receiver_id") val receiverId: String,
    @SerializedName("initiator_signature") val initiatorSignature: String,
    @SerializedName("device_fingerprint") val deviceFingerprint: String
)

data class ConfirmHandshakeRequest(
    @SerializedName("handshake_id") val handshakeId: String,
    @SerializedName("receiver_signature") val receiverSignature: String,
    @SerializedName("device_fingerprint") val deviceFingerprint: String
)

data class DisputeRequest(
    @SerializedName("handshake_id") val handshakeId: String,
    @SerializedName("reason") val reason: String,
    @SerializedName("evidence_urls") val evidenceUrls: List<String>,
    @SerializedName("device_fingerprint") val deviceFingerprint: String
)

// Scoring Models
data class ScoringOutput(
    @SerializedName("supplier_id") val supplierId: String,
    @SerializedName("ics_score") val icsScore: Int,
    @SerializedName("probability_of_default") val probabilityOfDefault: Double,
    @SerializedName("stability_index") val stabilityIndex: Double,
    @SerializedName("credit_limit_recommendation") val creditLimitRecommendation: Double,
    @SerializedName("risk_category") val riskCategory: String,
    @SerializedName("factors") val factors: List<ScoreFactor>,
    @SerializedName("calculated_at") val calculatedAt: String
)

data class ScoreFactor(
    @SerializedName("name") val name: String,
    @SerializedName("weight") val weight: Double,
    @SerializedName("score") val score: Double,
    @SerializedName("impact") val impact: String
)

// Consent Models
data class ConsentLog(
    @SerializedName("id") val id: String,
    @SerializedName("user_id") val userId: String,
    @SerializedName("granted_to") val grantedTo: String,
    @SerializedName("purpose") val purpose: String,
    @SerializedName("data_types") val dataTypes: List<String>,
    @SerializedName("is_active") val isActive: Boolean,
    @SerializedName("granted_at") val grantedAt: String,
    @SerializedName("revoked_at") val revokedAt?
)

data class CreateConsentRequest(
    @SerializedName("granted_to") val grantedTo: String,
    @SerializedName("purpose") val purpose: String,
    @SerializedName("data_types") val dataTypes: List<String>
)

// Report Models
data class GenerateReportRequest(
    @SerializedName("report_type") val reportType: String,
    @SerializedName("start_date") val startDate: String,
    @SerializedName("end_date") val endDate: String,
    @SerializedName("format") val format: String = "PDF"
)

data class ReportData(
    @SerializedName("report_id") val reportId: String,
    @SerializedName("report_type") val reportType: String,
    @SerializedName("download_url") val downloadUrl: String,
    @SerializedName("generated_at") val generatedAt: String
)

// Upload Models
data class UploadResponse(
    @SerializedName("file_id") val fileId: String,
    @SerializedName("file_name") val fileName: String,
    @SerializedName("file_url") val fileUrl: String,
    @SerializedName("file_size") val fileSize: Long,
    @SerializedName("uploaded_at") val uploadedAt: String
)
