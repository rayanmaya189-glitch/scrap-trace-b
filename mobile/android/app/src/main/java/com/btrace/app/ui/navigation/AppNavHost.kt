package com.btrace.app.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.btrace.app.ui.screens.auth.LoginScreen
import com.btrace.app.ui.screens.auth.OTPVerificationScreen
import com.btrace.app.ui.screens.dashboard.DashboardScreen
import com.btrace.app.ui.screens.materials.MaterialListScreen
import com.btrace.app.ui.screens.materials.MaterialDetailScreen
import com.btrace.app.ui.screens.materials.CreateMaterialScreen
import com.btrace.app.ui.screens.handshakes.HandshakeInitiateScreen
import com.btrace.app.ui.scores.ScoresScreen
import com.btrace.app.ui.compliance.ComplianceScreen
import com.btrace.app.ui.profile.ProfileScreen

@Composable
fun AppNavHost(
    navController: NavHostController = rememberNavController()
) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route

    NavHost(
        navController = navController,
        startDestination = Screen.Splash.route
    ) {
        // Splash Screen
        composable(Screen.Splash.route) {
            SplashScreen(
                onNavigateToLogin = {
                    navController.navigate(Screen.Login.route) {
                        popUpTo(Screen.Splash.route) { inclusive = true }
                    }
                }
            )
        }

        // Auth Screens
        composable(Screen.Login.route) {
            LoginScreen(
                onLoginSuccess = { phoneNumber ->
                    navController.navigate(Screen.OTPVerification.createRoute(phoneNumber))
                }
            )
        }

        composable(
            route = Screen.OTPVerification.route,
            arguments = listOf(navArgument("phoneNumber") { type = NavType.StringType })
        ) { backStackEntry ->
            val phoneNumber = backStackEntry.arguments?.getString("phoneNumber") ?: ""
            OTPVerificationScreen(
                phoneNumber = phoneNumber,
                onVerificationSuccess = {
                    navController.navigate(Screen.Dashboard.route) {
                        popUpTo(Screen.Login.route) { inclusive = true }
                    }
                },
                onBackPress = { navController.popBackStack() }
            )
        }

        // Main App Screens
        composable(Screen.Dashboard.route) {
            DashboardScreen(
                onNavigateToMaterials = { navController.navigate(Screen.Materials.route) },
                onNavigateToHandshake = { navController.navigate(Screen.HandshakeInitiate.route) },
                onNavigateToScores = { navController.navigate(Screen.Scores.route) },
                onNavigateToCompliance = { navController.navigate(Screen.Compliance.route) },
                onNavigateToProfile = { navController.navigate(Screen.Profile.route) }
            )
        }

        composable(Screen.Materials.route) {
            MaterialListScreen(
                onMaterialClick = { materialId ->
                    navController.navigate(Screen.MaterialDetail.createRoute(materialId))
                },
                onCreateMaterialClick = { navController.navigate(Screen.CreateMaterial.route) },
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(
            route = Screen.MaterialDetail.route,
            arguments = listOf(navArgument("materialId") { type = NavType.StringType })
        ) { backStackEntry ->
            val materialId = backStackEntry.arguments?.getString("materialId") ?: ""
            MaterialDetailScreen(
                materialId = materialId,
                onBackPress = { navController.popBackStack() },
                onInitiateHandshake = { navController.navigate(Screen.HandshakeInitiate.route) }
            )
        }

        composable(Screen.CreateMaterial.route) {
            CreateMaterialScreen(
                onMaterialCreated = { navController.popBackStack() },
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(Screen.HandshakeInitiate.route) {
            HandshakeInitiateScreen(
                onHandshakeInitiated = { navController.popBackStack() },
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(Screen.Scores.route) {
            ScoresScreen(
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(Screen.Compliance.route) {
            ComplianceScreen(
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(Screen.Profile.route) {
            ProfileScreen(
                onNavigateToSettings = { navController.navigate(Screen.Settings.route) },
                onNavigateToConsent = { navController.navigate(Screen.ConsentManagement.route) },
                onLogout = {
                    navController.navigate(Screen.Login.route) {
                        popUpTo(0) { inclusive = true }
                    }
                },
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(Screen.Settings.route) {
            SettingsScreen(
                onBackPress = { navController.popBackStack() }
            )
        }

        composable(Screen.ConsentManagement.route) {
            ConsentManagementScreen(
                onBackPress = { navController.popBackStack() }
            )
        }
    }
}

// Placeholder screens for compilation
@Composable
fun SplashScreen(onNavigateToLogin: () -> Unit) {
    androidx.compose.runtime.LaunchedEffect(Unit) {
        kotlinx.coroutines.delay(2000)
        onNavigateToLogin()
    }
    androidx.compose.material3.CircularProgressIndicator()
}

@Composable
fun SettingsScreen(onBackPress: () -> Unit) {
    androidx.compose.material3.Text("Settings")
}

@Composable
fun ConsentManagementScreen(onBackPress: () -> Unit) {
    androidx.compose.material3.Text("Consent Management")
}
