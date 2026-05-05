//
//  MaterialsListView.swift
//  BTraceApp
//

import SwiftUI

struct MaterialsListView: View {
    @StateObject private var viewModel = MaterialsListViewModel()
    @State private var showingCreateSheet = false
    
    var body: some View {
        NavigationView {
            ZStack {
                if viewModel.isLoading {
                    ProgressView("Loading materials...")
                } else if viewModel.materials.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "box")
                            .font(.system(size: 48))
                            .foregroundColor(.gray)
                        Text("No materials found")
                            .font(.headline)
                        Button("Add Material") {
                            showingCreateSheet = true
                        }
                        .buttonStyle(.borderedProminent)
                    }
                } else {
                    List(viewModel.materials) { material in
                        NavigationLink(destination: MaterialDetailView(material: material)) {
                            MaterialRow(material: material)
                        }
                    }
                }
            }
            .navigationTitle("Materials")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { showingCreateSheet = true }) {
                        Image(systemName: "plus")
                    }
                }
            }
            .sheet(isPresented: $showingCreateSheet) {
                CreateMaterialView()
            }
        }
    }
}

struct MaterialRow: View {
    let material: MaterialPassport
    
    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(material.materialType)
                    .font(.headline)
                Text("Batch: \(material.batchId)")
                    .font(.caption)
                    .foregroundColor(.gray)
                Text("\(material.quantityKg) \(material.unit)")
                    .font(.subheadline)
            }
            
            Spacer()
            
            VStack(alignment: .trailing, spacing: 4) {
                StatusBadge(status: material.status)
                if let carbon = material.carbonIntensity {
                    Text("\(carbon, specifier: "%.2f") kg CO₂")
                        .font(.caption)
                        .foregroundColor(.green)
                }
            }
        }
        .padding(.vertical, 4)
    }
}

struct StatusBadge: View {
    let status: String
    
    var statusColor: Color {
        switch status.lowercased() {
        case "verified", "completed": return .green
        case "pending": return .orange
        case "disputed": return .red
        default: return .gray
        }
    }
    
    var body: some View {
        Text(status.capitalized)
            .font(.caption)
            .fontWeight(.semibold)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(statusColor.opacity(0.2))
            .foregroundColor(statusColor)
            .cornerRadius(4)
    }
}

struct MaterialDetailView: View {
    let material: MaterialPassport
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // Basic Info Card
                InfoCard(title: "Basic Information") {
                    DetailRow(label: "Batch ID", value: material.batchId)
                    DetailRow(label: "Material Type", value: material.materialType)
                    DetailRow(label: "Grade", value: material.grade ?? "N/A")
                    DetailRow(label: "Quantity", value: "\(material.quantityKg) \(material.unit)")
                    DetailRow(label: "Status", value: material.status)
                }
                
                // Compliance Info Card
                if let cbam = material.cbamCategory ?? nil {
                    InfoCard(title: "Compliance Information") {
                        DetailRow(label: "CBAM Category", value: cbam)
                        if let carbon = material.carbonIntensity {
                            DetailRow(label: "Carbon Intensity", value: "\(carbon, specifier: "%.2f") kg CO₂/kg")
                        }
                        if let recycled = material.recycledContentPct {
                            DetailRow(label: "Recycled Content", value: "\(recycled, specifier: "%.1f")%")
                        }
                    }
                }
                
                // Handshake History
                InfoCard(title: "Ownership History") {
                    ForEach(material.handshakeHistory, id: \.id) { handshake in
                        HandshakeTimelineItem(handshake: handshake)
                    }
                }
            }
            .padding()
        }
        .navigationTitle("Material Details")
    }
}

struct InfoCard<Content: View>: View {
    let title: String
    @ViewBuilder let content: Content
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .font(.headline)
                .foregroundColor(.primary)
            
            Divider()
            
            content
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.05), radius: 4, x: 0, y: 2)
    }
}

struct DetailRow: View {
    let label: String
    let value: String
    
    var body: some View {
        HStack {
            Text(label)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
                .fontWeight(.medium)
        }
        .padding(.vertical, 4)
    }
}

struct HandshakeTimelineItem: View {
    let handshake: DigitalHandshake
    
    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Circle()
                .fill(handshake.status == "confirmed" ? Color.green : Color.orange)
                .frame(width: 12, height: 12)
            
            VStack(alignment: .leading, spacing: 4) {
                Text("\(handshake.initiatorName) → \(handshake.receiverName)")
                    .font(.subheadline)
                    .fontWeight(.medium)
                Text(handshake.initiatedAt.formatted())
                    .font(.caption)
                    .foregroundColor(.gray)
            }
            
            Spacer()
        }
        .padding(.vertical, 4)
    }
}

#Preview {
    MaterialsListView()
}
