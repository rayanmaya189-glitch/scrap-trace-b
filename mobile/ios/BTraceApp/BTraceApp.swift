import SwiftUI

@main
struct BTraceApp: App {
    @StateObject private var appState = AppState()
    
    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(appState)
        }
    }
}

// MARK: - App State
class AppState: ObservableObject {
    @Published var isAuthenticated = false
    @Published var accessToken: String?
    @Published var refreshToken: String?
    @Published var currentUser: SupplierProfile?
    
    init() {
        loadTokens()
    }
    
    func loadTokens() {
        if let token = UserDefaults.standard.string(forKey: "accessToken"),
           let refresh = UserDefaults.standard.string(forKey: "refreshToken") {
            self.accessToken = token
            self.refreshToken = refresh
            self.isAuthenticated = true
        }
    }
    
    func saveTokens(accessToken: String, refreshToken: String) {
        self.accessToken = accessToken
        self.refreshToken = refreshToken
        UserDefaults.standard.set(accessToken, forKey: "accessToken")
        UserDefaults.standard.set(refreshToken, forKey: "refreshToken")
        self.isAuthenticated = true
    }
    
    func logout() {
        accessToken = nil
        refreshToken = nil
        currentUser = nil
        UserDefaults.standard.removeObject(forKey: "accessToken")
        UserDefaults.standard.removeObject(forKey: "refreshToken")
        isAuthenticated = false
    }
}

// MARK: - Root View
struct RootView: View {
    @EnvironmentObject var appState: AppState
    
    var body: some View {
        Group {
            if appState.isAuthenticated {
                MainTabView()
            } else {
                AuthNavigationView()
            }
        }
    }
}

// MARK: - Main Tab View
struct MainTabView: View {
    var body: some View {
        TabView {
            DashboardView()
                .tabItem {
                    Label("Dashboard", systemImage: "house")
                }
            
            MaterialsListView()
                .tabItem {
                    Label("Materials", systemImage: "box")
                }
            
            HandshakeView()
                .tabItem {
                    Label("Handshakes", systemImage: "handshake")
                }
            
            ScoresView()
                .tabItem {
                    Label("Scores", systemImage: "chart.bar")
                }
            
            ComplianceView()
                .tabItem {
                    Label("Compliance", systemImage: "doc.text")
                }
            
            ProfileView()
                .tabItem {
                    Label("Profile", systemImage: "person")
                }
        }
    }
}

// MARK: - Placeholder Views
struct AuthNavigationView: View {
    var body: some View {
        NavigationStack {
            LoginView()
        }
    }
}

struct DashboardView: View {
    var body: some View {
        NavigationStack {
            Text("Dashboard")
                .navigationTitle("Dashboard")
        }
    }
}

struct MaterialsListView: View {
    var body: some View {
        NavigationStack {
            Text("Materials")
                .navigationTitle("Materials")
        }
    }
}

struct HandshakeView: View {
    var body: some View {
        NavigationStack {
            Text("Handshakes")
                .navigationTitle("Handshakes")
        }
    }
}

struct ScoresView: View {
    var body: some View {
        NavigationStack {
            Text("Scores")
                .navigationTitle("Credit Scores")
        }
    }
}

struct ComplianceView: View {
    var body: some View {
        NavigationStack {
            Text("Compliance")
                .navigationTitle("Compliance")
        }
    }
}

struct ProfileView: View {
    @EnvironmentObject var appState: AppState
    
    var body: some View {
        NavigationStack {
            VStack {
                Text("Profile")
                Button("Logout") {
                    appState.logout()
                }
                .buttonStyle(.borderedProminent)
            }
            .navigationTitle("Profile")
        }
    }
}

#Preview {
    BTraceApp()
}
