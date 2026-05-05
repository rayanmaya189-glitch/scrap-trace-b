package com.btrace.app.ui.screens.handshakes

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.btrace.app.data.api.BTraceApi
import com.btrace.app.data.model.Handshake
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HandshakesScreen(
    viewModel: HandshakesViewModel = hiltViewModel(),
    onHandshakeClick: (String) -> Unit,
    onInitiateHandshake: () -> Unit
) {
    val uiState by viewModel.uiState.collectAsState()
    val scope = rememberCoroutineScope()
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Handshakes") },
                actions = {
                    IconButton(onClick = onInitiateHandshake) {
                        Icon(Icons.Default.Add, contentDescription = "Initiate Handshake")
                    }
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            when (uiState) {
                is HandshakesUiState.Loading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
                is HandshakesUiState.Success -> {
                    val handshakes = (uiState as HandshakesUiState.Success).handshakes
                    
                    if (handshakes.isEmpty()) {
                        EmptyHandshakesView(
                            modifier = Modifier.align(Alignment.Center),
                            onInitiateClick = onInitiateHandshake
                        )
                    } else {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(16.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            items(handshakes, key = { it.id }) { handshake ->
                                HandshakeCard(
                                    handshake = handshake,
                                    onClick = { onHandshakeClick(handshake.id) }
                                )
                            }
                        }
                    }
                }
                is HandshakesUiState.Error -> {
                    ErrorView(
                        message = (uiState as HandshakesUiState.Error).message,
                        onRetry = { viewModel.loadHandshakes() },
                        modifier = Modifier.align(Alignment.Center)
                    )
                }
            }
        }
    }
}

@Composable
private fun HandshakeCard(
    handshake: Handshake,
    onClick: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
        shape = RoundedCornerShape(12.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(
                        text = "Material: ${handshake.materialId.take(8)}...",
                        fontWeight = FontWeight.Bold,
                        style = MaterialTheme.typography.titleMedium
                    )
                    Text(
                        text = "ID: ${handshake.id.take(12)}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                HandshakeStatusChip(status = handshake.status)
            }
            
            Spacer(modifier = Modifier.height(12.dp))
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column {
                    Text(
                        text = "From:",
                        style = MaterialTheme.typography.labelSmall
                    )
                    Text(
                        text = handshake.sellerName ?: "N/A",
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
                
                Column(horizontalAlignment = Alignment.End) {
                    Text(
                        text = "To:",
                        style = MaterialTheme.typography.labelSmall
                    )
                    Text(
                        text = handshake.buyerName ?: "Pending",
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }
            
            Spacer(modifier = Modifier.height(12.dp))
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = handshake.createdAt,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                
                if (handshake.requiresConfirmation) {
                    Button(
                        onClick = { /* Handle confirmation */ },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary
                        ),
                        modifier = Modifier.height(32.dp)
                    ) {
                        Text("Confirm", style = MaterialTheme.typography.labelLarge)
                    }
                }
            }
        }
    }
}

@Composable
private fun HandshakeStatusChip(status: String) {
    val (color, label) = when (status.lowercase()) {
        "pending" -> Color(0xFFFFA000) to "Pending"
        "confirmed" -> Color(0xFF4CAF50) to "Confirmed"
        "disputed" -> Color(0xFFF44336) to "Disputed"
        "completed" -> Color(0xFF2196F3) to "Completed"
        else -> Color.Gray to status
    }
    
    Surface(
        color = color.copy(alpha = 0.1f),
        shape = RoundedCornerShape(16.dp),
        modifier = Modifier.padding(start = 8.dp)
    ) {
        Text(
            text = label,
            color = color,
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp)
        )
    }
}

@Composable
private fun EmptyHandshakesView(
    modifier: Modifier = Modifier,
    onInitiateClick: () -> Unit
) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = Icons.Default.Shake,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Spacer(modifier = Modifier.height(16.dp))
        Text(
            text = "No Handshakes Yet",
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onSurface
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "Start by initiating a material transfer handshake",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.widthIn(max = 250.dp)
        )
        Spacer(modifier = Modifier.height(24.dp))
        Button(onClick = onInitiateClick) {
            Icon(Icons.Default.Add, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(8.dp))
            Text("Initiate Handshake")
        }
    }
}

@Composable
private fun ErrorView(
    message: String,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = Icons.Default.ErrorOutline,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.error
        )
        Spacer(modifier = Modifier.height(16.dp))
        Text(
            text = "Failed to Load Handshakes",
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onSurface
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.widthIn(max = 250.dp)
        )
        Spacer(modifier = Modifier.height(24.dp))
        Button(onClick = onRetry) {
            Text("Retry")
        }
    }
}

sealed class HandshakesUiState {
    object Loading : HandshakesUiState()
    data class Success(val handshakes: List<Handshake>) : HandshakesUiState()
    data class Error(val message: String) : HandshakesUiState()
}
