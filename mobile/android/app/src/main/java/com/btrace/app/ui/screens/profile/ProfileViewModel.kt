package com.btrace.app.ui.screens.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.btrace.app.data.api.BTraceApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ProfileViewModel @Inject constructor(
    private val api: BTraceApi
) : ViewModel() {

    private val _uiState = MutableStateFlow<ProfileUiState>(ProfileUiState.Loading)
    val uiState: StateFlow<ProfileUiState> = _uiState

    init {
        loadUserProfile()
    }

    private fun loadUserProfile() {
        viewModelScope.launch {
            _uiState.value = ProfileUiState.Loading
            try {
                val user = api.getUserProfile()
                _uiState.value = ProfileUiState.Success(user)
            } catch (e: Exception) {
                _uiState.value = ProfileUiState.Error(
                    e.message ?: "Failed to load profile"
                )
            }
        }
    }

    fun downloadMyData() {
        viewModelScope.launch {
            try {
                val downloadUrl = api.downloadMyData()
                // Handle file download
            } catch (e: Exception) {
                // Show error
            }
        }
    }

    fun requestAccountDeletion() {
        viewModelScope.launch {
            try {
                api.requestAccountDeletion()
                // Show success message
            } catch (e: Exception) {
                // Show error
            }
        }
    }

    suspend fun updateProfile(name: String, email: String, phone: String): Result<Unit> {
        return try {
            api.updateUserProfile(name, email, phone)
            loadUserProfile() // Refresh data
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
