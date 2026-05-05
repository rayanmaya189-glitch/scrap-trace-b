package com.btrace.app.ui.screens.dashboard

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DashboardScreen(
    onNavigateToMaterials: () -> Unit,
    onNavigateToHandshakes: () -> Unit,
    onNavigateToCompliance: () -> Unit,
    onNavigateToProfile: () -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    
    Scaffold(
        bottomBar = {
            NavigationBar {
                NavigationBarItem(
                    icon = { Icon(Icons.Default.Dashboard, contentDescription = "Dashboard") },
                    label = { Text("Home") },
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 }
                )
                NavigationBarItem(
                    icon = { Icon(Icons.Default.Inventory, contentDescription = "Materials") },
                    label = { Text("Materials") },
                    selected = selectedTab == 1,
                    onClick = { 
                        selectedTab = 1
                        onNavigateToMaterials()
                    }
                )
                NavigationBarItem(
                    icon = { Icon(Icons.Default.Shake, contentDescription = "Handshakes") },
                    label = { Text("Handshakes") },
                    selected = selectedTab == 2,
                    onClick = {
                        selectedTab = 2
                        onNavigateToHandshakes()
                    }
                )
                NavigationBarItem(
                    icon = { Icon(Icons.Default.Assessment, contentDescription = "Compliance") },
                    label = { Text("Compliance") },
                    selected = selectedTab == 3,
                    onClick = {
                        selectedTab = 3
                        onNavigateToCompliance()
                    }
                )
                NavigationBarItem(
                    icon = { Icon(Icons.Default.Person, contentDescription = "Profile") },
                    label = { Text("Profile") },
                    selected = selectedTab == 4,
                    onClick = {
                        selectedTab = 4
                        onNavigateToProfile()
                    }
                )
            }
        }
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            item {
                Text(
                    text = "Welcome to B-Trace",
                    style = MaterialTheme.typography.headlineMedium,
                    modifier = Modifier.padding(bottom = 24.dp)
                )
            }
            
            item {
                SummaryCards()
            }
            
            item {
                Spacer(modifier = Modifier.height(24.dp))
                Text(
                    text = "Recent Activity",
                    style = MaterialTheme.typography.titleLarge,
                    modifier = Modifier.padding(bottom = 16.dp)
                )
            }
            
            items(5) { index ->
                ActivityItem(index)
            }
        }
    }
}

@Composable
fun SummaryCards() {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        SummaryCard(
            title = "Total Materials",
            value = "156",
            icon = Icons.Default.Inventory,
            modifier = Modifier.weight(1f)
        )
        SummaryCard(
            title = "Pending Handshakes",
            value = "12",
            icon = Icons.Default.Shake,
            modifier = Modifier.weight(1f)
        )
    }
    
    Spacer(modifier = Modifier.height(16.dp))
    
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        SummaryCard(
            title = "Compliance Score",
            value = "94%",
            icon = Icons.Default.Assessment,
            modifier = Modifier.weight(1f)
        )
        SummaryCard(
            title = "Carbon Saved",
            value = "2.4T",
            icon = Icons.Default.Eco,
            modifier = Modifier.weight(1f)
        )
    }
}

@Composable
fun SummaryCard(
    title: String,
    value: String,
    icon: ImageVector,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier,
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
                tint = MaterialTheme.colorScheme.primary
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = value,
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.primary
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun ActivityItem(index: Int) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Default.Info,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.secondary
            )
            Spacer(modifier = Modifier.width(16.dp))
            Column {
                Text(
                    text = "Activity Item ${index + 1}",
                    style = MaterialTheme.typography.bodyLarge
                )
                Text(
                    text = "${index + 1} hours ago",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}
