import { useState, useCallback, useEffect } from 'react';

export interface NetworkStatus {
  isOnline: boolean;
  isOffline: boolean;
  connectionType?: ConnectionType;
  downlink?: number;
  rtt?: number;
}

type ConnectionType = 'bluetooth' | 'cellular' | 'ethernet' | 'none' | 'wifi' | 'wimax' | 'other' | 'unknown';

/**
 * Custom hook for monitoring network connectivity status
 * Provides detailed connection information when available
 */
export function useOnlineStatus(): NetworkStatus {
  const [status, setStatus] = useState<NetworkStatus>({
    isOnline: typeof navigator !== 'undefined' ? navigator.onLine : true,
    isOffline: typeof navigator !== 'undefined' ? !navigator.onLine : false,
  });

  const updateStatus = useCallback(() => {
    const newStatus: NetworkStatus = {
      isOnline: navigator.onLine,
      isOffline: !navigator.onLine,
    };

    // Get detailed connection info if available
    const connection = (navigator as any).connection || 
                       (navigator as any).mozConnection || 
                       (navigator as any).webkitConnection;
    
    if (connection) {
      newStatus.connectionType = connection.type as ConnectionType;
      newStatus.downlink = connection.downlink;
      newStatus.rtt = connection.rtt;
    }

    setStatus(newStatus);
  }, []);

  useEffect(() => {
    // Initial status
    updateStatus();

    // Listen for online/offline events
    window.addEventListener('online', updateStatus);
    window.addEventListener('offline', updateStatus);

    return () => {
      window.removeEventListener('online', updateStatus);
      window.removeEventListener('offline', updateStatus);
    };
  }, [updateStatus]);

  return status;
}

/**
 * Hook to check if user is currently online
 * Simplified version of useOnlineStatus
 */
export function useIsOnline(): boolean {
  const { isOnline } = useOnlineStatus();
  return isOnline;
}

/**
 * Hook to check if user is currently offline
 * Simplified version of useOnlineStatus
 */
export function useIsOffline(): boolean {
  const { isOffline } = useOnlineStatus();
  return isOffline;
}
