package com.btrace.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class OtpRequest(
    @SerialName("phone") val phone: String
)

@Serializable
data class OtpVerificationRequest(
    @SerialName("phone") val phone: String,
    @SerialName("otp") val otp: String
)

@Serializable
data class AuthResponse(
    @SerialName("access_token") val accessToken: String,
    @SerialName("refresh_token") val refreshToken: String,
    @SerialName("expires_in") val expiresIn: Long,
    @SerialName("user") val user: UserInfo
)

@Serializable
data class UserInfo(
    @SerialName("id") val id: String,
    @SerialName("phone") val phone: String,
    @SerialName("role") val role: String,
    @SerialName("is_verified") val isVerified: Boolean
)

@Serializable
data class MaterialListResponse(
    @SerialName("materials") val materials: List<MaterialDto>,
    @SerialName("total") val total: Int
)

@Serializable
data class MaterialDto(
    @SerialName("id") val id: String,
    @SerialName("batch_id") val batchId: String,
    @SerialName("material_type") val materialType: String,
    @SerialName("weight") val weight: Double,
    @SerialName("status") val status: String,
    @SerialName("created_at") val createdAt: Long
)

@Serializable
data class CreateMaterialRequest(
    @SerialName("batch_id") val batchId: String,
    @SerialName("material_type") val materialType: String,
    @SerialName("weight") val weight: Double,
    @SerialName("cbam_data") val cbamData: CbamDataDto? = null,
    @SerialName("carbon_intensity") val carbonIntensity: Double? = null,
    @SerialName("recycled_content") val recycledContent: Double? = null
)

@Serializable
data class CbamDataDto(
    @SerialName("emission_factor") val emissionFactor: Double,
    @SerialName("reporting_period") val reportingPeriod: String
)

@Serializable
data class HandshakeInitRequest(
    @SerialName("material_id") val materialId: String,
    @SerialName("recipient_id") val recipientId: String,
    @SerialName("initiator_signature") val initiatorSignature: String,
    @SerialName("device_fingerprint") val deviceFingerprint: String
)

@Serializable
data class HandshakeConfirmRequest(
    @SerialName("handshake_id") val handshakeId: String,
    @SerialName("recipient_signature") val recipientSignature: String,
    @SerialName("device_fingerprint") val deviceFingerprint: String
)

@Serializable
data class DisputeRequest(
    @SerialName("handshake_id") val handshakeId: String,
    @SerialName("reason") val reason: String,
    @SerialName("evidence_urls") val evidenceUrls: List<String> = emptyList()
)

@Serializable
data class ConsentRequest(
    @SerialName("purpose") val purpose: String,
    @SerialName("granted_to") val grantedTo: String,
    @SerialName("expires_at") val expiresAt: Long? = null
)

@Serializable
data class ReportGenerationRequest(
    @SerialName("report_type") val reportType: String,
    @SerialName("period_start") val periodStart: Long,
    @SerialName("period_end") val periodEnd: Long
)

@Serializable
data class ApiResponse<T>(
    @SerialName("success") val success: Boolean,
    @SerialName("data") val data: T?,
    @SerialName("error") val error: String? = null
)
