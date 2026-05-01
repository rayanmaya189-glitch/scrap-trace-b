import React, { createContext, useContext, useState } from 'react';

interface Toast {
  id: string;
  title?: string;
  description?: string;
  variant?: 'default' | 'destructive' | 'success' | 'warning';
  duration?: number;
}

interface ToastContextType {
  toasts: Toast[];
  toast: (toast: Omit<Toast, 'id'>) => string;
  dismiss: (id: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = ({ title, description, variant = 'default', duration = 5000 }: Omit<Toast, 'id'>) => {
    const id = Math.random().toString(36).substr(2, 9);
    const newToast: Toast = { id, title, description, variant, duration };
    
    setToasts((prev) => [...prev, newToast]);

    if (duration > 0) {
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, duration);
    }

    return id;
  };

  const dismiss = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  return (
    <ToastContext.Provider value={{ toasts, toast, dismiss }}>
      {children}
      <div className="fixed bottom-0 right-0 z-50 p-4 space-y-4 max-w-md">
        {toasts.map((toastItem) => (
          <div
            key={toastItem.id}
            className={`
              rounded-lg shadow-lg p-4 border transition-all animate-in slide-in-from-right-full
              ${toastItem.variant === 'destructive' ? 'bg-red-50 border-red-200 text-red-800' : ''}
              ${toastItem.variant === 'success' ? 'bg-green-50 border-green-200 text-green-800' : ''}
              ${toastItem.variant === 'warning' ? 'bg-yellow-50 border-yellow-200 text-yellow-800' : ''}
              ${toastItem.variant === 'default' ? 'bg-white border-gray-200 text-gray-800' : ''}
            `}
          >
            {toastItem.title && (
              <div className="font-medium text-sm">{toastItem.title}</div>
            )}
            {toastItem.description && (
              <div className="text-sm opacity-90 mt-1">{toastItem.description}</div>
            )}
            <button
              onClick={() => dismiss(toastItem.id)}
              className="absolute top-2 right-2 text-xs opacity-50 hover:opacity-100"
            >
              ✕
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}

export function toast(toastData: Omit<Toast, 'id'>) {
  // This is a convenience function for using toast outside components
  // It will work once the provider is set up
  const event = new CustomEvent('btrace-toast', { detail: toastData });
  window.dispatchEvent(event);
  return '';
}
