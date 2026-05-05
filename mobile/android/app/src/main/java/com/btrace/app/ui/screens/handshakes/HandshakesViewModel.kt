package com.btrace.app.ui.screens.handshakes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.btrace.app.data.api.BTraceApi
import com.btrace.app.data.model.Handshake
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class HandshakesViewModel @Inject constructor(
    private val api: BTraceApi
) : ViewModel() {

    private val _uiState = MutableStateFlow<HandshakesUiState>(HandshakesUiState.Loading)
    val uiState: StateFlow<HandshakesUiState> = _uiState

    init {
        loadHandshakes()
    }

    fun loadHandshakes() {
        viewModelScope.launch {
            _uiState.value = HandshakesUiState.Loading
            try {
                val handshakes = api.getHandshakes()
                _uiState.value = HandshakesUiState.Success(handshakes)
            } catch (e: Exception) {
                _uiState.value = HandshakesUiState.Error(
                    e.message ?: "Failed to load handshakes"
                )
            }
        }
    }

    suspend fun confirmHandshake(handshakeId: String): Result<Unit> {
        return try {
            api.confirmHandshake(handshakeId)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun disputeHandshake(handshakeId: String, reason: String, evidenceUrls: List<String>): Result<Unit> {
        return try {
            api.disputeHandshake(handshakeId, reason, evidenceUrls)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
