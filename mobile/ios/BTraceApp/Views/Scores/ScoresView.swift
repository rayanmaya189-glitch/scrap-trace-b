//
//  ScoresView.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import SwiftUI

struct ScoresView: View {
    @StateObject private var viewModel = ScoresViewModel()
    
    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Overall Score Card
                    OverallScoreCard(uiState: viewModel.uiState)
                    
                    // Score Breakdown
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Score Breakdown")
                            .font(.title2)
                            .fontWeight(.bold)
                            .padding(.horizontal)
                        
                        switch viewModel.uiState {
                        case .success(let scores):
                            ScoreItem(
                                icon: "lock.shield",
                                title: "Security Compliance",
                                score: scores.securityScore,
                                maxScore: 30
                            )
                            ScoreItem(
                                icon: "chart.bar",
                                title: "Reporting Accuracy",
                                score: scores.reportingScore,
                                maxScore: 25
                            )
                            ScoreItem(
                                icon: "checkmark.seal",
                                title: "Verification Rate",
                                score: scores.verificationScore,
                                maxScore: 25
                            )
                            ScoreItem(
                                icon: "clock",
                                title: "Timeliness",
                                score: scores.timelinessScore,
                                maxScore: 20
                            )
                        default:
                            ProgressView()
                                .frame(maxWidth: .infinity)
                                .padding()
                        }
                    }
                    
                    // Recommendations
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Recommendations")
                            .font(.title2)
                            .fontWeight(.bold)
                            .padding(.horizontal)
                        
                        switch viewModel.uiState {
                        case .success(let scores):
                            ForEach(scores.recommendations, id: \.self) { recommendation in
                                RecommendationCard(recommendation: recommendation)
                            }
                        default:
                            ProgressView()
                                .frame(maxWidth: .infinity)
                                .padding()
                        }
                    }
                }
                .padding(.vertical)
            }
            .navigationTitle("Compliance Scores")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        // Show info
                    }) {
                        Image(systemName: "info.circle")
                    }
                }
            }
        }
    }
}

struct OverallScoreCard: View {
    let uiState: ScoresUiState
    
    var body: some View {
        VStack(spacing: 16) {
            if case .success(let scores) = uiState {
                ZStack {
                    Circle()
                        .stroke(Color.gray.opacity(0.2), lineWidth: 12)
                        .frame(width: 160, height: 160)
                    
                    Circle()
                        .trim(from: 0, to: CGFloat(scores.overallScore) / 100.0)
                        .stroke(scoreColor(scores.overallScore), style: StrokeStyle(lineWidth: 12, lineCap: .round))
                        .frame(width: 160, height: 160)
                        .rotationEffect(.degrees(-90))
                    
                    VStack {
                        Text("\(scores.overallScore)")
                            .font(.system(size: 48, weight: .bold))
                            .foregroundColor(scoreColor(scores.overallScore))
                        Text("out of 100")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                
                Text(scoreRating(scores.overallScore))
                    .font(.title3)
                    .fontWeight(.semibold)
                    .foregroundColor(scoreColor(scores.overallScore))
                
                Text("Last updated: \(scores.lastUpdated)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                ProgressView("Calculating score...")
                    .padding()
            }
        }
        .padding()
        .background(Color.white)
        .cornerRadius(16)
        .shadow(color: Color.black.opacity(0.1), radius: 8, x: 0, y: 2)
        .padding(.horizontal)
    }
    
    private func scoreColor(_ score: Int) -> Color {
        if score >= 80 { return .green }
        if score >= 60 { return .orange }
        if score >= 40 { return Color.red.opacity(0.8) }
        return .red
    }
    
    private func scoreRating(_ score: Int) -> String {
        if score >= 90 { return "Excellent" }
        if score >= 75 { return "Good" }
        if score >= 60 { return "Fair" }
        if score >= 40 { return "Poor" }
        return "Critical"
    }
}

struct ScoreItem: View {
    let icon: String
    let title: String
    let score: Int
    let maxScore: Int
    
    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(.blue)
                .frame(width: 40)
            
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.body)
                    .fontWeight(.medium)
                
                GeometryReader { geometry in
                    ZStack(alignment: .leading) {
                        RoundedRectangle(cornerRadius: 3)
                            .fill(Color.gray.opacity(0.2))
                            .frame(height: 6)
                        
                        RoundedRectangle(cornerRadius: 3)
                            .fill(scoreColor(score, maxScore: maxScore))
                            .frame(width: geometry.size.width * CGFloat(score) / CGFloat(maxScore), height: 6)
                    }
                }
                .frame(height: 6)
            }
            
            Spacer()
            
            Text("\(score)/\(maxScore)")
                .font(.body)
                .fontWeight(.bold)
        }
        .padding(.horizontal)
    }
    
    private func scoreColor(_ score: Int, maxScore: Int) -> Color {
        let percentage = maxScore > 0 ? score * 100 / maxScore : 0
        if percentage >= 80 { return .green }
        if percentage >= 60 { return .orange }
        return .red
    }
}

struct RecommendationCard: View {
    let recommendation: String
    
    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: "lightbulb")
                .foregroundColor(.yellow)
                .font(.body)
            
            Text(recommendation)
                .font(.body)
                .foregroundColor(.secondary)
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.yellow.opacity(0.1))
        .cornerRadius(12)
        .padding(.horizontal)
    }
}

struct ScoresView_Previews: PreviewProvider {
    static var previews: some View {
        ScoresView()
    }
}
