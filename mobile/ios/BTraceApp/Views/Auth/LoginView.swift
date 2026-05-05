import SwiftUI

struct LoginView: View {
    @State private var phoneNumber = ""
    @State private var isLoading = false
    @State private var errorMessage: String?
    @State private var navigateToOTP = false
    
    var body: some View {
        VStack(spacing: 24) {
            // Logo
            Image(systemName: "shield.checkered")
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 100, height: 100)
                .foregroundColor(.blue)
            
            Text("B-Trace Protocol")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("Secure Material Tracking & Verification")
                .font(.subheadline)
                .foregroundColor(.gray)
                .multilineTextAlignment(.center)
            
            Spacer().frame(height: 40)
            
            // Phone Number Input
            VStack(alignment: .leading, spacing: 8) {
                Text("Phone Number")
                    .font(.headline)
                
                TextField("+91 XXXXX XXXXX", text: $phoneNumber)
                    .keyboardType(.phonePad)
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(12)
                    .autocapitalization(.none)
                    .disableAutocorrection(true)
            }
            .padding(.horizontal)
            
            if let error = errorMessage {
                Text(error)
                    .foregroundColor(.red)
                    .font(.caption)
                    .padding(.horizontal)
            }
            
            // Login Button
            Button(action: handleLogin) {
                HStack {
                    if isLoading {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .white))
                    } else {
                        Text("Request OTP")
                            .fontWeight(.semibold)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(phoneNumber.count >= 10 ? Color.blue : Color.gray)
                .foregroundColor(.white)
                .cornerRadius(12)
            }
            .disabled(isLoading || phoneNumber.count < 10)
            .padding(.horizontal)
            
            Spacer()
            
            // Footer
            Text("By continuing, you agree to our Terms & Privacy Policy")
                .font(.caption)
                .foregroundColor(.gray)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
        }
        .padding(.vertical)
        .navigationDestination(isPresented: $navigateToOTP) {
            OTPVerificationView(phoneNumber: phoneNumber)
        }
    }
    
    private func handleLogin() {
        isLoading = true
        errorMessage = nil
        
        // TODO: Call API to request OTP
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            isLoading = false
            navigateToOTP = true
        }
    }
}

#Preview {
    NavigationStack {
        LoginView()
    }
}
