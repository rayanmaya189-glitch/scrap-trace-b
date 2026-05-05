//
//  BTraceApp.swift
//  BTraceApp
//
//  Created by BTrace Team on 2024.
//

import SwiftUI

@main
struct BTraceApp: App {
    @StateObject private var authManager = AuthManager()
    @StateObject private var syncManager = SyncManager()
    
    var body: some Scene {
        WindowGroup {
            if authManager.isAuthenticated {
                MainTabView()
                    .environmentObject(authManager)
                    .environmentObject(syncManager)
            } else {
                LoginView()
                    .environmentObject(authManager)
            }
        }
    }
}
