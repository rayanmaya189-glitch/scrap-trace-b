package com.btrace.app.ui.screens.scores

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.btrace.app.data.api.BTraceApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ScoresViewModel @Inject constructor(
    private val api: BTraceApi
) : ViewModel() {

    private val _uiState = MutableStateFlow<ScoresUiState>(ScoresUiState.Loading)
    val uiState: StateFlow<ScoresUiState> = _uiState

    init {
        loadScores()
    }

    private fun loadScores() {
        viewModelScope.launch {
            _uiState.value = ScoresUiState.Loading
            try {
                val scores = api.getComplianceScores()
                _uiState.value = ScoresUiState.Success(
                    overallScore = scores.overallScore,
                    securityScore = scores.securityScore,
                    reportingScore = scores.reportingScore,
                    verificationScore = scores.verificationScore,
                    timelinessScore = scores.timelinessScore,
                    lastUpdated = scores.lastUpdated,
                    recommendations = generateRecommendations(scores)
                )
            } catch (e: Exception) {
                _uiState.value = ScoresUiState.Error(
                    e.message ?: "Failed to load scores"
                )
            }
        }
    }

    private fun generateRecommendations(scores: ComplianceScores): List<String> {
        val recommendations = mutableListOf<String>()
        
        if (scores.securityScore < 25) {
            recommendations.add("Improve device fingerprinting and signature verification rates")
        }
        if (scores.reportingScore < 20) {
            recommendations.add("Submit pending compliance reports (CBAM/EPR)")
        }
        if (scores.verificationScore < 20) {
            recommendations.add("Complete pending handshake confirmations")
        }
        if (scores.timelinessScore < 15) {
            recommendations.add("Reduce response time for material transfer requests")
        }
        
        if (recommendations.isEmpty()) {
            recommendations.add("Excellent compliance! Maintain current practices")
        }
        
        return recommendations
    }
}

data class ComplianceScores(
    val overallScore: Int,
    val securityScore: Int,
    val reportingScore: Int,
    val verificationScore: Int,
    val timelinessScore: Int,
    val lastUpdated: String
)
