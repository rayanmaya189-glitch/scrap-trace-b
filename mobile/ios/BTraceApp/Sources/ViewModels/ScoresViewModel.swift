//
//  ScoresViewModel.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import Foundation

@MainActor
class ScoresViewModel: ObservableObject {
    @Published var uiState: ScoresUiState = .loading
    
    private let networkService: NetworkService
    
    init(networkService: NetworkService = NetworkService.shared) {
        self.networkService = networkService
        Task {
            await loadScores()
        }
    }
    
    func loadScores() async {
        uiState = .loading
        
        do {
            let scores = try await networkService.getComplianceScores()
            uiState = .success(
                overallScore: scores.overallScore,
                securityScore: scores.securityScore,
                reportingScore: scores.reportingScore,
                verificationScore: scores.verificationScore,
                timelinessScore: scores.timelinessScore,
                lastUpdated: scores.lastUpdated,
                recommendations: generateRecommendations(scores: scores)
            )
        } catch {
            uiState = .error(error.localizedDescription)
        }
    }
    
    private func generateRecommendations(scores: ComplianceScores) -> [String] {
        var recommendations: [String] = []
        
        if scores.securityScore < 25 {
            recommendations.append("Improve device fingerprinting and signature verification rates")
        }
        if scores.reportingScore < 20 {
            recommendations.append("Submit pending compliance reports (CBAM/EPR)")
        }
        if scores.verificationScore < 20 {
            recommendations.append("Complete pending handshake confirmations")
        }
        if scores.timelinessScore < 15 {
            recommendations.append("Reduce response time for material transfer requests")
        }
        
        if recommendations.isEmpty {
            recommendations.append("Excellent compliance! Maintain current practices")
        }
        
        return recommendations
    }
}

enum ScoresUiState: Equatable {
    case loading
    case success(
        overallScore: Int,
        securityScore: Int,
        reportingScore: Int,
        verificationScore: Int,
        timelinessScore: Int,
        lastUpdated: String,
        recommendations: [String]
    )
    case error(String)
}
