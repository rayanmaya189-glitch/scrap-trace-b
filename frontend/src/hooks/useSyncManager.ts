import { useEffect } from 'react';
import { syncService } from '../services/syncService';
import { useSyncStore } from '../store/syncStore';

/**
 * Hook to manage sync service lifecycle and provide sync status
 */
export function useSyncManager() {
  const syncStatus = useSyncStore((state) => ({
    isSyncing: state.isSyncing,
    pendingCount: state.pendingQueue.filter((item) => item.status === 'pending').length,
    conflictCount: state.pendingQueue.filter((item) => item.status === 'conflict').length,
    failedCount: state.pendingQueue.filter((item) => item.status === 'failed').length,
    lastSyncTime: state.lastSyncTime,
    syncError: state.syncError,
  }));

  useEffect(() => {
    // Start sync loop on mount
    syncService.startSyncLoop();

    // Stop sync loop on unmount
    return () => {
      syncService.stopSyncLoop();
    };
  }, []);

  // Listen to online/offline events
  useEffect(() => {
    const handleOnline = () => {
      console.log('[useSyncManager] Device came online, triggering sync...');
      syncService.startSyncLoop();
    };

    const handleOffline = () => {
      console.log('[useSyncManager] Device went offline, pausing sync...');
      syncService.stopSyncLoop();
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  return {
    ...syncStatus,
    isOnline: navigator.onLine,
    queueStatus: syncService.getSyncStatus(),
  };
}

/**
 * Hook to get current sync status without managing lifecycle
 */
export function useSyncStatus() {
  return useSyncStore((state) => ({
    isSyncing: state.isSyncing,
    pendingCount: state.pendingQueue.length,
    lastSyncTime: state.lastSyncTime,
    syncError: state.syncError,
    hasPending: state.pendingQueue.some((item) => item.status === 'pending'),
    hasConflicts: state.pendingQueue.some((item) => item.status === 'conflict'),
  }));
}
