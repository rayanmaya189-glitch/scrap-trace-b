package com.btrace.app.ui.screens.profile

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.btrace.app.data.model.UserProfile

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel = hiltViewModel(),
    onLogout: () -> Unit,
    onEditProfile: () -> Unit,
    onViewConsents: () -> Unit,
    onSettingsClick: () -> Unit
) {
    val uiState by viewModel.uiState.collectAsState()
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Profile") },
                actions = {
                    IconButton(onClick = onSettingsClick) {
                        Icon(Icons.Default.Settings, contentDescription = "Settings")
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
            // Profile Header
            ProfileHeader(
                uiState = uiState,
                onEditClick = onEditProfile
            )
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Profile Stats
            ProfileStats(uiState)
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Menu Items
            ProfileMenuItems(
                onViewConsents = onViewConsents,
                onDownloadData = { viewModel.downloadMyData() },
                onDeleteAccount = { viewModel.requestAccountDeletion() },
                onLogout = onLogout
            )
        }
    }
}

@Composable
private fun ProfileHeader(
    uiState: ProfileUiState,
    onEditClick: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Avatar
        Surface(
            shape = CircleShape,
            color = MaterialTheme.colorScheme.primaryContainer,
            modifier = Modifier.size(100.dp)
        ) {
            Box(contentAlignment = Alignment.Center) {
                if (uiState is ProfileUiState.Success && uiState.user.avatarUrl != null) {
                    // Load image from URL (implement with Coil or similar)
                    Text(
                        text = uiState.user.name.first().toString(),
                        style = MaterialTheme.typography.headlineLarge,
                        color = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                } else {
                    Icon(
                        imageVector = Icons.Default.Person,
                        contentDescription = null,
                        modifier = Modifier.size(60.dp),
                        tint = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                }
            }
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        when (uiState) {
            is ProfileUiState.Success -> {
                Text(
                    text = uiState.user.name,
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = uiState.user.email,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    text = uiState.user.phone,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            else -> {
                Text(
                    text = "Loading...",
                    style = MaterialTheme.typography.bodyLarge
                )
            }
        }
        
        Spacer(modifier = Modifier.height(12.dp))
        
        OutlinedButton(
            onClick = onEditClick,
            modifier = Modifier.height(40.dp)
        ) {
            Icon(Icons.Default.Edit, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(8.dp))
            Text("Edit Profile")
        }
    }
}

@Composable
private fun ProfileStats(uiState: ProfileUiState) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly
    ) {
        when (uiState) {
            is ProfileUiState.Success -> {
                StatItem("Materials", uiState.user.materialCount.toString())
                StatItem("Handshakes", uiState.user.handshakeCount.toString())
                StatItem("Score", uiState.user.complianceScore.toString())
            }
            else -> {
                StatItem("Materials", "-")
                StatItem("Handshakes", "-")
                StatItem("Score", "-")
            }
        }
    }
}

@Composable
private fun StatItem(label: String, value: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(
            text = value,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
private fun ProfileMenuItems(
    onViewConsents: () -> Unit,
    onDownloadData: () -> Unit,
    onDeleteAccount: () -> Unit,
    onLogout: () -> Unit
) {
    Column {
        ProfileMenuItem(
            icon = Icons.Default.Shield,
            title = "Consent Management",
            subtitle = "Manage your data sharing consents",
            onClick = onViewConsents
        )
        
        ProfileMenuItem(
            icon = Icons.Default.Download,
            title = "Download My Data",
            subtitle = "Export all your data in JSON format",
            onClick = onDownloadData
        )
        
        ProfileMenuItem(
            icon = Icons.Default.Info,
            title = "About B-Trace",
            subtitle = "Version 1.0.0",
            onClick = { }
        )
        
        HorizontalDivider()
        
        ProfileMenuItem(
            icon = Icons.Default.DeleteForever,
            title = "Delete Account",
            subtitle = "Permanently delete your account and data",
            onClick = onDeleteAccount,
            textColor = MaterialTheme.colorScheme.error
        )
        
        ProfileMenuItem(
            icon = Icons.Default.Logout,
            title = "Logout",
            subtitle = "Sign out from your account",
            onClick = onLogout,
            textColor = MaterialTheme.colorScheme.error
        )
    }
}

@Composable
private fun ProfileMenuItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    subtitle: String,
    onClick: () -> Unit,
    textColor: androidx.compose.ui.graphics.Color = MaterialTheme.colorScheme.onSurface
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = textColor,
            modifier = Modifier.size(24.dp)
        )
        Spacer(modifier = Modifier.width(16.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
                color = textColor
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        Icon(
            imageVector = Icons.Default.ChevronRight,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

sealed class ProfileUiState {
    object Loading : ProfileUiState()
    data class Success(val user: UserProfile) : ProfileUiState()
    data class Error(val message: String) : ProfileUiState()
}
