//
//  DashboardView.swift
//  BTraceApp
//
//  Created by B-Trace Team
//

import SwiftUI

struct DashboardView: View {
    @State private var selectedTab = 0
    
    var body: some View {
        TabView(selection: $selectedTab) {
            HomeTabView()
                .tabItem {
                    Image(systemName: "house.fill")
                    Text("Home")
                }
                .tag(0)
            
            MaterialsListView()
                .tabItem {
                    Image(systemName: "box.fill")
                    Text("Materials")
                }
                .tag(1)
            
            HandshakesListView()
                .tabItem {
                    Image(systemName: "handshake.fill")
                    Text("Handshakes")
                }
                .tag(2)
            
            ComplianceView()
                .tabItem {
                    Image(systemName: "chart.bar.fill")
                    Text("Compliance")
                }
                .tag(3)
            
            ProfileView()
                .tabItem {
                    Image(systemName: "person.fill")
                    Text("Profile")
                }
                .tag(4)
        }
    }
}

struct HomeTabView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                Text("Welcome to B-Trace")
                    .font(.title)
                    .fontWeight(.bold)
                    .padding(.bottom, 10)
                
                // Summary Cards
                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 16) {
                    SummaryCard(
                        title: "Total Materials",
                        value: "156",
                        iconName: "box.fill"
                    )
                    
                    SummaryCard(
                        title: "Pending Handshakes",
                        value: "12",
                        iconName: "handshake.fill"
                    )
                    
                    SummaryCard(
                        title: "Compliance Score",
                        value: "94%",
                        iconName: "chart.bar.fill"
                    )
                    
                    SummaryCard(
                        title: "Carbon Saved",
                        value: "2.4T",
                        iconName: "leaf.fill"
                    )
                }
                
                // Recent Activity
                Text("Recent Activity")
                    .font(.title2)
                    .fontWeight(.semibold)
                    .padding(.top, 20)
                
                ForEach(0..<5) { index in
                    ActivityRow(index: index)
                }
            }
            .padding()
        }
    }
}

struct SummaryCard: View {
    let title: String
    let value: String
    let iconName: String
    
    var body: some View {
        VStack(alignment: .center, spacing: 8) {
            Image(systemName: iconName)
                .font(.title2)
                .foregroundColor(.blue)
            
            Text(value)
                .font(.title2)
                .fontWeight(.bold)
                .foregroundColor(.blue)
            
            Text(title)
                .font(.caption)
                .foregroundColor(.gray)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

struct ActivityRow: View {
    let index: Int
    
    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "info.circle.fill")
                .foregroundColor(.secondary)
            
            VStack(alignment: .leading, spacing: 4) {
                Text("Activity Item \(index + 1)")
                    .font(.body)
                
                Text("\(index + 1) hours ago")
                    .font(.caption)
                    .foregroundColor(.gray)
            }
            
            Spacer()
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(8)
        .shadow(color: .black.opacity(0.05), radius: 2, x: 0, y: 1)
    }
}

#Preview {
    DashboardView()
}
