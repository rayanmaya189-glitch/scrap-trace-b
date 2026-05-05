//
//  ComplianceView.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import SwiftUI

struct ComplianceView: View {
    @StateObject private var viewModel = ComplianceViewModel()
    
    var body: some View {
        NavigationView {
            List {
                // Summary Section
                Section(header: Text("Summary")) {
                    HStack(spacing: 16) {
                        SummaryCard(
                            title: "Reports",
                            value: viewModel.totalReports,
                            icon: "doc.text"
                        )
                        SummaryCard(
                            title: "Pending",
                            value: viewModel.pendingActions,
                            icon: "exclamationmark.triangle"
                        )
                    }
                    .padding(.vertical, 8)
                }
                
                // Available Reports
                Section(header: Text("Available Reports")) {
                    ForEach(viewModel.reportTypes, id: \.self) { reportType in
                        ReportRow(
                            reportType: reportType,
                            onGenerate: {
                                Task {
                                    await viewModel.generateReport(reportType)
                                }
                            }
                        )
                    }
                }
                
                // Upcoming Deadlines
                Section(header: Text("Upcoming Deadlines")) {
                    ForEach(viewModel.upcomingDeadlines) { deadline in
                        DeadlineRow(deadline: deadline)
                    }
                }
                
                // Actions
                Section {
                    Button(action: {
                        // Navigate to consent management
                    }) {
                        Label("Consent Management", systemImage: "shield")
                    }
                }
            }
            .navigationTitle("Compliance")
            .refreshable {
                await viewModel.loadComplianceData()
            }
        }
    }
}

struct SummaryCard: View {
    let title: String
    let value: Int
    let icon: String
    
    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(.blue)
            
            Text("\(value)")
                .font(.title)
                .fontWeight(.bold)
            
            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(Color.blue.opacity(0.1))
        .cornerRadius(12)
    }
}

struct ReportRow: View {
    let reportType: String
    let onGenerate: () -> Void
    
    var description: String {
        switch reportType {
        case "CBAM Report": return "Carbon Border Adjustment Mechanism"
        case "EPR Report": return "Extended Producer Responsibility"
        case "Carbon Intensity Report": return "CI calculation per material"
        case "Mass Balance Report": return "Input-output mass tracking"
        case "GST Compliance Report": return "Tax compliance summary"
        case "Audit Trail Export": return "Complete transaction history"
        default: return "Generate compliance report"
        }
    }
    
    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                Text(reportType)
                    .font(.headline)
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            Button(action: onGenerate) {
                Label("Generate", systemImage: "arrow.down.doc")
            }
            .buttonStyle(.bordered)
        }
        .padding(.vertical, 4)
    }
}

struct DeadlineRow: View {
    let deadline: ComplianceDeadline
    
    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                Text(deadline.title)
                    .font(.body)
                    .fontWeight(.medium)
                Text("Due: \(deadline.dueDate)")
                    .font(.caption)
                    .foregroundColor(deadline.isUrgent ? .red : .secondary)
            }
            
            Spacer()
            
            Label(
                deadline.isUrgent ? "Urgent" : "Upcoming",
                systemImage: deadline.isUrgent ? "exclamationmark.circle" : "clock"
            )
            .font(.caption)
            .foregroundColor(deadline.isUrgent ? .red : .orange)
        }
        .padding(.vertical, 4)
    }
}

struct ComplianceView_Previews: PreviewProvider {
    static var previews: some View {
        ComplianceView()
    }
}
