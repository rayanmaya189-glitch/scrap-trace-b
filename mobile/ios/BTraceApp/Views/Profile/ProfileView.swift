//
//  ProfileView.swift
//  BTraceApp
//
//  Created by BTrace Team
//  Copyright © 2024 BTrace. All rights reserved.
//

import SwiftUI

struct ProfileView: View {
    @StateObject private var viewModel = ProfileViewModel()
    var onLogout: () -> Void
    
    var body: some View {
        NavigationView {
            List {
                // Profile Header
                Section {
                    VStack(spacing: 12) {
                        switch viewModel.uiState {
                        case .success(let user):
                            AvatarView(name: user.name)
                            
                            Text(user.name)
                                .font(.headline)
                            
                            Text(user.email)
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                            
                            Text(user.phone)
                                .font(.caption)
                                .foregroundColor(.secondary)
                            
                            Button(action: {
                                // Edit profile
                            }) {
                                Label("Edit Profile", systemImage: "pencil")
                            }
                            .buttonStyle(.bordered)
                            
                        case .loading:
                            ProgressView("Loading profile...")
                                .frame(maxWidth: .infinity)
                                .padding()
                            
                        case .error(let message):
                            Text(message)
                                .foregroundColor(.red)
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 16)
                }
                
                // Stats
                Section(header: Text("Statistics")) {
                    HStack(spacing: 16) {
                        switch viewModel.uiState {
                        case .success(let user):
                            StatItem(label: "Materials", value: "\(user.materialCount)")
                            StatItem(label: "Handshakes", value: "\(user.handshakeCount)")
                            StatItem(label: "Score", value: "\(user.complianceScore)")
                        default:
                            StatItem(label: "Materials", value: "-")
                            StatItem(label: "Handshakes", value: "-")
                            StatItem(label: "Score", value: "-")
                        }
                    }
                    .padding(.vertical, 8)
                }
                
                // Settings
                Section(header: Text("Settings")) {
                    NavigationLink(destination: Text("Consent Management")) {
                        Label("Consent Management", systemImage: "shield")
                    }
                    
                    Button(action: {
                        Task {
                            await viewModel.downloadMyData()
                        }
                    }) {
                        Label("Download My Data", systemImage: "arrow.down.doc")
                    }
                    
                    NavigationLink(destination: Text("About")) {
                        Label("About B-Trace", systemImage: "info.circle")
                    }
                }
                
                // Danger Zone
                Section(header: Text("Account")) {
                    Button(role: .destructive) {
                        Task {
                            await viewModel.requestAccountDeletion()
                        }
                    } label: {
                        Label("Delete Account", systemImage: "trash")
                    }
                    
                    Button(role: .destructive, action: onLogout) {
                        Label("Logout", systemImage: "rectangle.portrait.and.arrow.right")
                    }
                }
            }
            .navigationTitle("Profile")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        // Settings
                    }) {
                        Image(systemName: "gear")
                    }
                }
            }
        }
    }
}

struct AvatarView: View {
    let name: String
    
    var body: some View {
        ZStack {
            Circle()
                .fill(Color.blue.opacity(0.15))
                .frame(width: 100, height: 100)
            
            Text(String(name.prefix(1)).uppercased())
                .font(.title)
                .fontWeight(.bold)
                .foregroundColor(.blue)
        }
    }
}

struct StatItem: View {
    let label: String
    let value: String
    
    var body: some View {
        VStack(spacing: 4) {
            Text(value)
                .font(.title2)
                .fontWeight(.bold)
                .foregroundColor(.blue)
            
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
    }
}

struct ProfileView_Previews: PreviewProvider {
    static var previews: some View {
        ProfileView(onLogout: {})
    }
}
