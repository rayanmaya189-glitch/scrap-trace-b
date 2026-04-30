import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '@/stores/useAuthStore';

export function LoginPage() {
  const navigate = useNavigate();
  const { requestOtp, login, isLoading, error, clearError } = useAuthStore();
  
  const [step, setStep] = useState<'phone' | 'otp'>('phone');
  const [phone, setPhone] = useState('');
  const [otp, setOtp] = useState('');

  const handleRequestOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await requestOtp(phone);
      setStep('otp');
    } catch (err) {
      console.error('Failed to request OTP:', err);
    }
  };

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await login(phone, otp);
      navigate('/dashboard');
    } catch (err) {
      console.error('Login failed:', err);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <div className="max-w-md w-full space-y-8">
        <div>
          <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
            B-Trace Protocol
          </h2>
          <p className="mt-2 text-center text-sm text-gray-600">
            Industrial Traceability & Credit Protocol
          </p>
        </div>

        {step === 'phone' ? (
          <form className="mt-8 space-y-6" onSubmit={handleRequestOtp}>
            <div className="rounded-md shadow-sm -space-y-px">
              <div>
                <label htmlFor="phone" className="sr-only">
                  Phone Number
                </label>
                <input
                  id="phone"
                  name="phone"
                  type="tel"
                  required
                  value={phone}
                  onChange={(e) => setPhone(e.target.value)}
                  className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                  placeholder="Enter your phone number"
                  pattern="[0-9]{10,15}"
                />
              </div>
            </div>

            {error && (
              <div className="text-red-500 text-sm text-center">{error}</div>
            )}

            <div>
              <button
                type="submit"
                disabled={isLoading}
                className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
              >
                {isLoading ? 'Sending...' : 'Send OTP'}
              </button>
            </div>
          </form>
        ) : (
          <form className="mt-8 space-y-6" onSubmit={handleLogin}>
            <div className="rounded-md shadow-sm -space-y-px">
              <div>
                <label htmlFor="otp" className="sr-only">
                  OTP
                </label>
                <input
                  id="otp"
                  name="otp"
                  type="text"
                  inputMode="numeric"
                  required
                  value={otp}
                  onChange={(e) => setOtp(e.target.value)}
                  className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm text-center tracking-widest text-2xl"
                  placeholder="Enter OTP"
                  pattern="[0-9]{4,8}"
                  maxLength={8}
                />
              </div>
            </div>

            {error && (
              <div className="text-red-500 text-sm text-center">{error}</div>
            )}

            <div className="flex space-x-4">
              <button
                type="button"
                onClick={() => {
                  setStep('phone');
                  clearError();
                }}
                className="flex-1 py-2 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none"
              >
                Back
              </button>
              <button
                type="submit"
                disabled={isLoading}
                className="flex-1 group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
              >
                {isLoading ? 'Verifying...' : 'Verify & Login'}
              </button>
            </div>
          </form>
        )}

        <div className="mt-4 text-center">
          <p className="text-xs text-gray-500">
            By continuing, you agree to our Terms of Service and Privacy Policy
          </p>
        </div>
      </div>
    </div>
  );
}
