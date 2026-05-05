//
//  HandshakesViewModel.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import Foundation

@MainActor
class HandshakesViewModel: ObservableObject {
    @Published var uiState: HandshakesUiState = .loading
    
    private let networkService: NetworkService
    
    init(networkService: NetworkService = NetworkService.shared) {
        self.networkService = networkService
        Task {
            await loadHandshakes()
        }
    }
    
    func loadHandshakes() async {
        uiState = .loading
        
        do {
            let handshakes = try await networkService.getHandshakes()
            uiState = .success(handshakes)
        } catch {
            uiState = .error(error.localizedDescription)
        }
    }
    
    func confirmHandshake(handshakeId: String) async throws {
        try await networkService.confirmHandshake(id: handshakeId)
        await loadHandshakes() // Refresh list
    }
    
    func disputeHandshake(handshakeId: String, reason: String, evidenceUrls: [String]) async throws {
        try await networkService.disputeHandshake(id: handshakeId, reason: reason, evidenceUrls: evidenceUrls)
        await loadHandshakes() // Refresh list
    }
}

enum HandshakesUiState: Equatable {
    case loading
    case success([Handshake])
    case error(String)
}
