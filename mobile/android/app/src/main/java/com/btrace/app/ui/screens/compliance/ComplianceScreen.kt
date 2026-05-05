package com.btrace.app.ui.screens.compliance

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.btrace.app.data.model.ComplianceReport
import com.btrace.app.data.model.ConsentRecord

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ComplianceScreen(
    viewModel: ComplianceViewModel = hiltViewModel(),
    onGenerateReport: (String) -> Unit,
    onViewConsents: () -> Unit
) {
    val uiState by viewModel.uiState.collectAsState()
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Compliance") },
                actions = {
                    IconButton(onClick = onViewConsents) {
                        Icon(Icons.Default.Shield, contentDescription = "Consent Management")
                    }
                }
            )
        }
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            item {
                ComplianceSummaryCards(uiState)
            }
            
            item {
                Text(
                    text = "Available Reports",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(8.dp))
            }
            
            items(getReportTypes()) { reportType ->
                ReportCard(
                    reportType = reportType,
                    onGenerate = { onGenerateReport(reportType) }
                )
            }
            
            item {
                Spacer(modifier = Modifier.height(16.dp))
                
                Text(
                    text = "Upcoming Deadlines",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.height(8.dp))
            }
            
            if (uiState is ComplianceUiState.Success) {
                val deadlines = (uiState as ComplianceUiState.Success).upcomingDeadlines
                items(deadlines) { deadline ->
                    DeadlineCard(deadline = deadline)
                }
            }
        }
    }
}

@Composable
private fun ComplianceSummaryCards(uiState: ComplianceUiState) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        when (uiState) {
            is ComplianceUiState.Success -> {
                SummaryCard(
                    title = "Total Reports",
                    value = uiState.totalReports.toString(),
                    icon = Icons.Default.Description,
                    modifier = Modifier.weight(1f)
                )
                SummaryCard(
                    title = "Pending Actions",
                    value = uiState.pendingActions.toString(),
                    icon = Icons.Default.Warning,
                    modifier = Modifier.weight(1f)
                )
            }
            else -> {
                SummaryCard(
                    title = "Total Reports",
                    value = "-",
                    icon = Icons.Default.Description,
                    modifier = Modifier.weight(1f)
                )
                SummaryCard(
                    title = "Pending Actions",
                    value = "-",
                    icon = Icons.Default.Warning,
                    modifier = Modifier.weight(1f)
                )
            }
        }
    }
}

@Composable
private fun SummaryCard(
    title: String,
    value: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier,
        shape = MaterialTheme.shapes.medium,
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(32.dp)
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = value,
                style = MaterialTheme.typography.headlineMedium,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun ReportCard(
    reportType: String,
    onGenerate: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.medium
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    text = reportType,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    text = getReportDescription(reportType),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Button(onClick = onGenerate) {
                Icon(
                    imageVector = Icons.Default.Download,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Generate")
            }
        }
    }
}

@Composable
private fun DeadlineCard(deadline: ComplianceDeadline) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.small
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    text = deadline.title,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    text = "Due: ${deadline.dueDate}",
                    style = MaterialTheme.typography.bodySmall,
                    color = if (deadline.isUrgent) MaterialTheme.colorScheme.error 
                            else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            AssistChip(
                onClick = { },
                label = { Text(if (deadline.isUrgent) "Urgent" else "Upcoming") },
                leadingIcon = {
                    Icon(
                        imageVector = if (deadline.isUrgent) Icons.Default.Error else Icons.Default.Schedule,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp)
                    )
                }
            )
        }
    }
}

private fun getReportTypes(): List<String> {
    return listOf(
        "CBAM Report",
        "EPR Report",
        "Carbon Intensity Report",
        "Mass Balance Report",
        "GST Compliance Report",
        "Audit Trail Export"
    )
}

private fun getReportDescription(reportType: String): String {
    return when (reportType) {
        "CBAM Report" -> "Carbon Border Adjustment Mechanism"
        "EPR Report" -> "Extended Producer Responsibility"
        "Carbon Intensity Report" -> "CI calculation per material"
        "Mass Balance Report" -> "Input-output mass tracking"
        "GST Compliance Report" -> "Tax compliance summary"
        "Audit Trail Export" -> "Complete transaction history"
        else -> "Generate compliance report"
    }
}

data class ComplianceDeadline(
    val title: String,
    val dueDate: String,
    val isUrgent: Boolean
)

sealed class ComplianceUiState {
    object Loading : ComplianceUiState()
    data class Success(
        val totalReports: Int = 0,
        val pendingActions: Int = 0,
        val upcomingDeadlines: List<ComplianceDeadline> = emptyList()
    ) : ComplianceUiState()
    data class Error(val message: String) : ComplianceUiState()
}
