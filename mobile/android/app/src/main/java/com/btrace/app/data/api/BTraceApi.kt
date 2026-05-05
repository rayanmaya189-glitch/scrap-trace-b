package com.btrace.app.data.api

import retrofit2.http.*
import com.btrace.app.data.model.*

interface BTraceApi {
    
    // Auth Endpoints
    @POST("auth/request-otp")
    suspend fun requestOtp(@Body request: OtpRequest): Response<BaseResponse<Unit>>
    
    @POST("auth/verify-otp")
    suspend fun verifyOtp(@Body request: VerifyOtpRequest): Response<AuthResponse>
    
    @POST("auth/refresh")
    suspend fun refreshToken(@Body request: RefreshTokenRequest): Response<AuthResponse>
    
    @POST("auth/logout")
    suspend fun logout(): Response<BaseResponse<Unit>>
    
    // Material Endpoints
    @GET("materials")
    suspend fun getMaterials(
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 20,
        @Query("status") status: String? = null
    ): Response<PagedResponse<MaterialPassport>>
    
    @GET("materials/{id}")
    suspend fun getMaterialById(@Path("id") id: String): Response<MaterialPassport>
    
    @POST("materials")
    suspend fun createMaterial(@Body material: CreateMaterialRequest): Response<MaterialPassport>
    
    @PATCH("materials/{id}/status/{status}")
    suspend fun updateMaterialStatus(
        @Path("id") id: String,
        @Path("status") status: String
    ): Response<MaterialPassport>
    
    @PATCH("materials/{id}/buyer/{buyerId}")
    suspend fun assignBuyer(
        @Path("id") id: String,
        @Path("buyerId") buyerId: String
    ): Response<MaterialPassport>
    
    @GET("materials/summary")
    suspend fun getMaterialSummary(): Response<MaterialSummary>
    
    // Supplier Endpoints
    @GET("suppliers/me")
    suspend fun getCurrentUserProfile(): Response<SupplierProfile>
    
    @PUT("suppliers/{id}")
    suspend fun updateSupplierProfile(
        @Path("id") id: String,
        @Body profile: UpdateSupplierRequest
    ): Response<SupplierProfile>
    
    // Handshake Endpoints
    @POST("handshakes/initiate")
    suspend fun initiateHandshake(@Body request: InitiateHandshakeRequest): Response<DigitalHandshake>
    
    @POST("handshakes/confirm")
    suspend fun confirmHandshake(@Body request: ConfirmHandshakeRequest): Response<DigitalHandshake>
    
    @POST("handshakes/dispute")
    suspend fun raiseDispute(@Body request: DisputeRequest): Response<DigitalHandshake>
    
    @GET("handshakes")
    suspend fun getHandshakes(
        @Query("material_id") materialId: String? = null,
        @Query("status") status: String? = null
    ): Response<List<DigitalHandshake>>
    
    // Score Endpoints
    @GET("scores/{supplierId}")
    suspend fun getSupplierScore(@Path("supplierId") supplierId: String): Response<ScoringOutput>
    
    // Consent Endpoints
    @GET("consent/my")
    suspend fun getUserConsents(): Response<List<ConsentLog>>
    
    @POST("consent")
    suspend fun createConsent(@Body request: CreateConsentRequest): Response<ConsentLog>
    
    @POST("consent/{id}/revoke")
    suspend fun revokeConsent(@Path("id") consentId: String): Response<ConsentLog>
    
    // Report Endpoints
    @POST("reports/generate")
    suspend fun generateReport(@Body request: GenerateReportRequest): Response<ReportData>
    
    // Upload Endpoints
    @Multipart
    @POST("upload/evidence")
    suspend fun uploadEvidence(
        @Part file: okhttp3.MultipartBody.Part,
        @Part("handshake_id") handshakeId: okhttp3.RequestBody,
        @Part("description") description: okhttp3.RequestBody
    ): Response<UploadResponse>
}
