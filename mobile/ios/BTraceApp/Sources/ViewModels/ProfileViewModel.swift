//
//  ProfileViewModel.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import Foundation

@MainActor
class ProfileViewModel: ObservableObject {
    @Published var uiState: ProfileUiState = .loading
    
    private let networkService: NetworkService
    
    init(networkService: NetworkService = NetworkService.shared) {
        self.networkService = networkService
        Task {
            await loadUserProfile()
        }
    }
    
    func loadUserProfile() async {
        uiState = .loading
        
        do {
            let user = try await networkService.getUserProfile()
            uiState = .success(user)
        } catch {
            uiState = .error(error.localizedDescription)
        }
    }
    
    func downloadMyData() async {
        do {
            let downloadUrl = try await networkService.downloadMyData()
            // Handle file download
            print("Data download URL: \(downloadUrl)")
        } catch {
            print("Failed to download data: \(error)")
        }
    }
    
    func requestAccountDeletion() async {
        do {
            try await networkService.requestAccountDeletion()
            // Show success message
        } catch {
            print("Failed to delete account: \(error)")
        }
    }
    
    func updateProfile(name: String, email: String, phone: String) async throws {
        try await networkService.updateUserProfile(name: name, email: email, phone: phone)
        await loadUserProfile() // Refresh data
    }
}

enum ProfileUiState: Equatable {
    case loading
    case success(UserProfile)
    case error(String)
}
