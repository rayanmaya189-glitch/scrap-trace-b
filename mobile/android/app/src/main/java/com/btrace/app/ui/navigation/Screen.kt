package com.btrace.app.ui.navigation

sealed class Screen(val route: String) {
    object Splash : Screen("splash")
    object Login : Screen("login")
    object OTPVerification : Screen("otp_verification/{phoneNumber}") {
        fun createRoute(phoneNumber: String) = "otp_verification/$phoneNumber"
    }
    object Dashboard : Screen("dashboard")
    object Materials : Screen("materials")
    object MaterialDetail : Screen("material_detail/{materialId}") {
        fun createRoute(materialId: String) = "material_detail/$materialId"
    }
    object CreateMaterial : Screen("create_material")
    object HandshakeInitiate : Screen("handshake_initiate")
    object HandshakeConfirm : Screen("handshake_confirm/{handshakeId}") {
        fun createRoute(handshakeId: String) = "handshake_confirm/$handshakeId"
    }
    object HandshakeDispute : Screen("handshake_dispute/{handshakeId}") {
        fun createRoute(handshakeId: String) = "handshake_dispute/$handshakeId"
    }
    object Scores : Screen("scores")
    object Compliance : Screen("compliance")
    object Profile : Screen("profile")
    object Settings : Screen("settings")
    object ConsentManagement : Screen("consent_management")
}
