import SwiftUI

struct OTPVerificationView: View {
    let phoneNumber: String
    @State private var otp = ""
    @State private var isLoading = false
    @State private var errorMessage: String?
    @Environment(\.dismiss) var dismiss
    
    var body: some View {
        VStack(spacing: 24) {
            Text("Verify OTP")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("Enter the OTP sent to \(phoneNumber)")
                .foregroundColor(.gray)
            
            Spacer().frame(height: 40)
            
            // OTP Input
            VStack(alignment: .leading, spacing: 8) {
                Text("One-Time Password")
                    .font(.headline)
                
                TextField("XXXXXX", text: $otp)
                    .keyboardType(.numberPad)
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(12)
                    .multilineTextAlignment(.center)
                    .font(.title2)
                    .monospacedDigit()
            }
            .padding(.horizontal)
            
            if let error = errorMessage {
                Text(error)
                    .foregroundColor(.red)
                    .font(.caption)
                    .padding(.horizontal)
            }
            
            // Verify Button
            Button(action: handleVerify) {
                HStack {
                    if isLoading {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .white))
                    } else {
                        Text("Verify & Continue")
                            .fontWeight(.semibold)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(otp.count >= 6 ? Color.blue : Color.gray)
                .foregroundColor(.white)
                .cornerRadius(12)
            }
            .disabled(isLoading || otp.count < 6)
            .padding(.horizontal)
            
            // Resend OTP
            Button("Resend OTP") {
                // TODO: Resend OTP logic
            }
            .foregroundColor(.blue)
            
            Spacer()
        }
        .padding(.vertical)
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Button("Back") {
                    dismiss()
                }
            }
        }
    }
    
    private func handleVerify() {
        isLoading = true
        errorMessage = nil
        
        // TODO: Call API to verify OTP
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            isLoading = false
            // On success, navigate to dashboard
        }
    }
}

#Preview {
    NavigationStack {
        OTPVerificationView(phoneNumber: "+91 9876543210")
    }
}
