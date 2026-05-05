package com.btrace.app.ui.screens.scores

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScoresScreen(
    viewModel: ScoresViewModel = hiltViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Compliance Scores") },
                actions = {
                    IconButton(onClick = { /* Show info */ }) {
                        Icon(Icons.Default.Info, contentDescription = "Score Information")
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .verticalScroll(rememberScrollState())
        ) {
            Spacer(modifier = Modifier.height(16.dp))
            
            // Overall Score Card
            OverallScoreCard(uiState)
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Score Breakdown
            Text(
                text = "Score Breakdown",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(horizontal = 16.dp)
            )
            Spacer(modifier = Modifier.height(12.dp))
            
            ScoreBreakdown(uiState)
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Improvement Recommendations
            Text(
                text = "Recommendations",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(horizontal = 16.dp)
            )
            Spacer(modifier = Modifier.height(12.dp))
            
            RecommendationsList(uiState)
            
            Spacer(modifier = Modifier.height(16.dp))
        }
    }
}

@Composable
private fun OverallScoreCard(uiState: ScoresUiState) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(16.dp),
        shape = MaterialTheme.shapes.large
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            when (uiState) {
                is ScoresUiState.Success -> {
                    val score = uiState.overallScore
                    val scoreColor = getScoreColor(score)
                    
                    // Circular Progress Indicator
                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier.size(160.dp)
                    ) {
                        CircularProgressIndicator(
                            progress = score / 100f,
                            modifier = Modifier.fillMaxSize(),
                            strokeWidth = 12.dp,
                            color = scoreColor
                        )
                        Column(
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Text(
                                text = score.toString(),
                                style = MaterialTheme.typography.displayLarge,
                                fontWeight = FontWeight.Bold,
                                color = scoreColor
                            )
                            Text(
                                text = "out of 100",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Text(
                        text = getScoreRating(score),
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.SemiBold,
                        color = scoreColor
                    )
                    
                    Spacer(modifier = Modifier.height(8.dp))
                    
                    Text(
                        text = "Last updated: ${uiState.lastUpdated}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                else -> {
                    CircularProgressIndicator()
                    Spacer(modifier = Modifier.height(16.dp))
                    Text("Calculating score...")
                }
            }
        }
    }
}

@Composable
private fun ScoreBreakdown(uiState: ScoresUiState) {
    Column(
        modifier = Modifier.padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        when (uiState) {
            is ScoresUiState.Success -> {
                ScoreItem(
                    icon = Icons.Default.Security,
                    title = "Security Compliance",
                    score = uiState.securityScore,
                    maxScore = 30
                )
                ScoreItem(
                    icon = Icons.Default.Assessment,
                    title = "Reporting Accuracy",
                    score = uiState.reportingScore,
                    maxScore = 25
                )
                ScoreItem(
                    icon = Icons.Default.Verified,
                    title = "Verification Rate",
                    score = uiState.verificationScore,
                    maxScore = 25
                )
                ScoreItem(
                    icon = Icons.Default.Timeline,
                    title = "Timeliness",
                    score = uiState.timelinessScore,
                    maxScore = 20
                )
            }
            else -> {
                repeat(4) {
                    ScoreItem(
                        icon = Icons.Default.Security,
                        title = "Loading...",
                        score = 0,
                        maxScore = 0
                    )
                }
            }
        }
    }
}

@Composable
private fun ScoreItem(
    icon: ImageVector,
    title: String,
    score: Int,
    maxScore: Int
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.size(32.dp)
        )
        Spacer(modifier = Modifier.width(16.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            LinearProgressIndicator(
                progress = if (maxScore > 0) score.toFloat() / maxScore else 0f,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(6.dp),
                color = getScoreColor(if (maxScore > 0) (score * 100 / maxScore) else 0)
            )
        }
        Spacer(modifier = Modifier.width(16.dp))
        Text(
            text = "$score/$maxScore",
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
private fun RecommendationsList(uiState: ScoresUiState) {
    Column(
        modifier = Modifier.padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        when (uiState) {
            is ScoresUiState.Success -> {
                uiState.recommendations.forEach { recommendation ->
                    RecommendationCard(recommendation)
                }
            }
            else -> {
                Text("Loading recommendations...")
            }
        }
    }
}

@Composable
private fun RecommendationCard(recommendation: String) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.small,
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.Top
        ) {
            Icon(
                imageVector = Icons.Default.Lightbulb,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.secondary,
                modifier = Modifier.size(20.dp)
            )
            Spacer(modifier = Modifier.width(12.dp))
            Text(
                text = recommendation,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

private fun getScoreColor(score: Int): Color {
    return when {
        score >= 80 -> Color(0xFF4CAF50) // Green
        score >= 60 -> Color(0xFFFFA000) // Orange
        score >= 40 -> Color(0xFFFF9800) // Deep Orange
        else -> Color(0xFFF44336) // Red
    }
}

private fun getScoreRating(score: Int): String {
    return when {
        score >= 90 -> "Excellent"
        score >= 75 -> "Good"
        score >= 60 -> "Fair"
        score >= 40 -> "Poor"
        else -> "Critical"
    }
}

sealed class ScoresUiState {
    object Loading : ScoresUiState()
    data class Success(
        val overallScore: Int = 0,
        val securityScore: Int = 0,
        val reportingScore: Int = 0,
        val verificationScore: Int = 0,
        val timelinessScore: Int = 0,
        val lastUpdated: String = "",
        val recommendations: List<String> = emptyList()
    ) : ScoresUiState()
    data class Error(val message: String) : ScoresUiState()
}
