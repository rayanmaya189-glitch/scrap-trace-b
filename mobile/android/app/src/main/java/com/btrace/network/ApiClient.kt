package com.btrace.network

import android.content.Context
import com.btrace.data.domain.ConsentRecord
import com.btrace.data.domain.DigitalHandshake
import com.btrace.data.domain.MaterialPassport
import com.btrace.data.domain.SupplierProfile
import io.ktor.client.*
import io.ktor.client.engine.android.*
import io.ktor.client.plugins.auth.*
import io.ktor.client.plugins.auth.providers.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.logging.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.json.Json

class ApiClient(private val context: Context) {
    
    private val client = HttpClient(Android) {
        install(ContentNegotiation) {
            json(Json {
                ignoreUnknownKeys = true
                isLenient = true
                encodeDefaults = true
            })
        }
        install(Logging) {
            logger = Logger.DEFAULT
            level = LogLevel.INFO
        }
        install(Auth) {
            bearer {
                loadTokens {
                    val prefs = context.getSharedPreferences("btrace_auth", Context.MODE_PRIVATE)
                    val token = prefs.getString("access_token", null)
                    BearerTokens(token ?: "", "")
                }
                refreshTokens {
                    val prefs = context.getSharedPreferences("btrace_auth", Context.MODE_PRIVATE)
                    val refreshToken = prefs.getString("refresh_token", null) ?: ""
                    
                    val response = client.post("$BASE_URL/v1/auth/refresh") {
                        header(HttpHeaders.Authorization, "Bearer $refreshToken")
                    }
                    
                    if (response.status.isSuccess()) {
                        // Parse and save new tokens
                        val prefsEditor = prefs.edit()
                        // Implementation for token refresh
                        BearerTokens("", "")
                    } else {
                        BearerTokens("", "")
                    }
                }
            }
        }
    }
    
    companion object {
        const val BASE_URL = "https://api.btrace.io"
    }
    
    suspend fun requestOtp(phone: String): Result<Unit> {
        return try {
            val response = client.post("$BASE_URL/v1/auth/request-otp") {
                contentType(ContentType.Application.Json)
                setBody(mapOf("phone" to phone))
            }
            if (response.status.isSuccess()) Result.success(Unit)
            else Result.failure(Exception("OTP request failed: ${response.status}"))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun verifyOtp(phone: String, otp: String): Result<AuthResponse> {
        return try {
            val response = client.post("$BASE_URL/v1/auth/verify-otp") {
                contentType(ContentType.Application.Json)
                setBody(mapOf("phone" to phone, "otp" to otp))
            }
            if (response.status.isSuccess()) {
                val authResponse = response.body<AuthResponse>()
                // Save tokens
                val prefs = context.getSharedPreferences("btrace_auth", Context.MODE_PRIVATE)
                prefs.edit().apply {
                    putString("access_token", authResponse.accessToken)
                    putString("refresh_token", authResponse.refreshToken)
                    apply()
                }
                Result.success(authResponse)
            } else {
                Result.failure(Exception("OTP verification failed"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun getMaterials(): Result<List<MaterialPassport>> {
        return try {
            val response = client.get("$BASE_URL/v1/materials")
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to fetch materials"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun createMaterial(request: CreateMaterialRequest): Result<MaterialPassport> {
        return try {
            val response = client.post("$BASE_URL/v1/materials") {
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to create material"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun initiateHandshake(request: HandshakeInitRequest): Result<DigitalHandshake> {
        return try {
            val response = client.post("$BASE_URL/v1/handshakes/initiate") {
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to initiate handshake"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun confirmHandshake(request: HandshakeConfirmRequest): Result<DigitalHandshake> {
        return try {
            val response = client.post("$BASE_URL/v1/handshakes/confirm") {
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to confirm handshake"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun raiseDispute(request: DisputeRequest): Result<DigitalHandshake> {
        return try {
            val response = client.post("$BASE_URL/v1/handshakes/dispute") {
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to raise dispute"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun getConsents(): Result<List<ConsentRecord>> {
        return try {
            val response = client.get("$BASE_URL/v1/consent/my")
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to fetch consents"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun grantConsent(request: ConsentRequest): Result<ConsentRecord> {
        return try {
            val response = client.post("$BASE_URL/v1/consent") {
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                Result.success(response.body())
            } else {
                Result.failure(Exception("Failed to grant consent"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun revokeConsent(consentId: String): Result<Unit> {
        return try {
            val response = client.post("$BASE_URL/v1/consent/$consentId/revoke")
            if (response.status.isSuccess()) {
                Result.success(Unit)
            } else {
                Result.failure(Exception("Failed to revoke consent"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    fun close() {
        client.close()
    }
}
