//
//  HandshakesListView.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import SwiftUI

struct HandshakesListView: View {
    @StateObject private var viewModel = HandshakesViewModel()
    @Binding var selectedHandshakeId: String?
    
    var body: some View {
        NavigationView {
            Group {
                switch viewModel.uiState {
                case .loading:
                    ProgressView("Loading handshakes...")
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                    
                case .success(let handshakes):
                    if handshakes.isEmpty {
                        EmptyHandshakesView()
                    } else {
                        List(handshakes) { handshake in
                            HandshakeRow(handshake: handshake)
                                .contentShape(Rectangle())
                                .onTapGesture {
                                    selectedHandshakeId = handshake.id
                                }
                        }
                        .refreshable {
                            await viewModel.loadHandshakes()
                        }
                    }
                    
                case .error(let message):
                    ErrorView(message: message) {
                        Task {
                            await viewModel.loadHandshakes()
                        }
                    }
                }
            }
            .navigationTitle("Handshakes")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        // Navigate to initiate handshake
                    }) {
                        Image(systemName: "plus")
                    }
                }
            }
        }
    }
}

struct HandshakeRow: View {
    let handshake: Handshake
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading) {
                    Text("Material: \(handshake.materialId.prefix(8))...")
                        .font(.headline)
                    Text("ID: \(handshake.id.prefix(12))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                StatusBadge(status: handshake.status)
            }
            
            HStack {
                VStack(alignment: .leading) {
                    Text("From:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(handshake.sellerName ?? "N/A")
                        .font(.subheadline)
                }
                
                Spacer()
                
                VStack(alignment: .trailing) {
                    Text("To:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(handshake.buyerName ?? "Pending")
                        .font(.subheadline)
                }
            }
            
            HStack {
                Text(handshake.createdAt)
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                Spacer()
                
                if handshake.requiresConfirmation {
                    Button(action: {
                        // Handle confirmation
                    }) {
                        Text("Confirm")
                            .font(.caption)
                            .fontWeight(.semibold)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.small)
                }
            }
        }
        .padding(.vertical, 4)
    }
}

struct StatusBadge: View {
    let status: String
    
    var body: some View {
        Text(status.capitalized)
            .font(.caption)
            .fontWeight(.medium)
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(statusColor.opacity(0.15))
            .foregroundColor(statusColor)
            .cornerRadius(12)
    }
    
    private var statusColor: Color {
        switch status.lowercased() {
        case "pending": return .orange
        case "confirmed": return .green
        case "disputed": return .red
        case "completed": return .blue
        default: return .gray
        }
    }
}

struct EmptyHandshakesView: View {
    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "handshake")
                .font(.system(size: 60))
                .foregroundColor(.secondary)
            
            Text("No Handshakes Yet")
                .font(.title2)
                .fontWeight(.semibold)
            
            Text("Start by initiating a material transfer handshake")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
            
            Button(action: {
                // Initiate handshake
            }) {
                Label("Initiate Handshake", systemImage: "plus")
            }
            .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

struct HandshakesListView_Previews: PreviewProvider {
    static var previews: some View {
        HandshakesListView(selectedHandshakeId: .constant(nil))
    }
}
