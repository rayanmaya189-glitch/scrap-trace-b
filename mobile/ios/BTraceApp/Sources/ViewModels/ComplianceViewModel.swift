//
//  ComplianceViewModel.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import Foundation

@MainActor
class ComplianceViewModel: ObservableObject {
    @Published var totalReports: Int = 0
    @Published var pendingActions: Int = 0
    @Published var upcomingDeadlines: [ComplianceDeadline] = []
    
    let reportTypes: [String] = [
        "CBAM Report",
        "EPR Report",
        "Carbon Intensity Report",
        "Mass Balance Report",
        "GST Compliance Report",
        "Audit Trail Export"
    ]
    
    private let networkService: NetworkService
    
    init(networkService: NetworkService = NetworkService.shared) {
        self.networkService = networkService
        Task {
            await loadComplianceData()
        }
    }
    
    func loadComplianceData() async {
        do {
            // Fetch compliance data from API
            let reports = try await networkService.getComplianceReports()
            totalReports = reports.count
            pendingActions = calculatePendingActions(reports)
            upcomingDeadlines = fetchUpcomingDeadlines()
        } catch {
            print("Failed to load compliance data: \(error)")
        }
    }
    
    func generateReport(_ reportType: String) async {
        do {
            let downloadUrl = try await networkService.generateReport(type: reportType)
            // Handle file download
            print("Report generated: \(downloadUrl)")
        } catch {
            print("Failed to generate report: \(error)")
        }
    }
    
    private func calculatePendingActions(_ reports: [ComplianceReport]) -> Int {
        // Count pending actions based on report status
        return reports.filter { $0.status == "pending" }.count
    }
    
    private func fetchUpcomingDeadlines() -> [ComplianceDeadline] {
        // Calculate based on compliance rules
        return [
            ComplianceDeadline(title: "CBAM Q4 Report", dueDate: "2024-12-31", isUrgent: true),
            ComplianceDeadline(title: "EPR Annual Filing", dueDate: "2025-01-31", isUrgent: false),
            ComplianceDeadline(title: "GST Reconciliation", dueDate: "2024-11-30", isUrgent: false)
        ]
    }
}

struct ComplianceDeadline: Identifiable {
    let id = UUID()
    let title: String
    let dueDate: String
    let isUrgent: Bool
}
