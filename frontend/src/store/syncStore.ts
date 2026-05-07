import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface PendingHandshake {
  id: string;
  materialId: string;
  supplierSig: string;
  buyerSig?: string;
  payloadHash: string;
  hashPrev: string;
  hashCurrent: string;
  versionVector: Record<string, number>;
  timestamp: number;
  status: 'pending' | 'syncing' | 'synced' | 'conflict' | 'failed';
  retryCount: number;
}

interface SyncState {
  pendingQueue: PendingHandshake[];
  isSyncing: boolean;
  lastSyncTime: number | null;
  syncError: string | null;
  
  // Actions
  addToQueue: (handshake: Omit<PendingHandshake, 'status' | 'retryCount'>) => void;
  updateQueueStatus: (id: string, status: PendingHandshake['status']) => void;
  removeFromQueue: (id: string) => void;
  setSyncing: (isSyncing: boolean) => void;
  setLastSyncTime: (time: number) => void;
  setSyncError: (error: string | null) => void;
  incrementRetry: (id: string) => void;
  getPendingCount: () => number;
  clearCompleted: () => void;
}

export const useSyncStore = create<SyncState>()(
  persist(
    (set, get) => ({
      pendingQueue: [],
      isSyncing: false,
      lastSyncTime: null,
      syncError: null,

      addToQueue: (handshake) => set((state) => ({
        pendingQueue: [
          ...state.pendingQueue,
          { ...handshake, status: 'pending', retryCount: 0 },
        ],
      })),

      updateQueueStatus: (id, status) => set((state) => ({
        pendingQueue: state.pendingQueue.map((item) =>
          item.id === id ? { ...item, status } : item
        ),
      })),

      removeFromQueue: (id) => set((state) => ({
        pendingQueue: state.pendingQueue.filter((item) => item.id !== id),
      })),

      setSyncing: (isSyncing) => set({ isSyncing }),

      setLastSyncTime: (time) => set({ lastSyncTime: time }),

      setSyncError: (error) => set({ syncError: error }),

      incrementRetry: (id) => set((state) => ({
        pendingQueue: state.pendingQueue.map((item) =>
          item.id === id ? { ...item, retryCount: item.retryCount + 1 } : item
        ),
      })),

      getPendingCount: () => get().pendingQueue.length,

      clearCompleted: () => set((state) => ({
        pendingQueue: state.pendingQueue.filter(
          (item) => item.status !== 'synced' && item.status !== 'failed'
        ),
      })),
    }),
    {
      name: 'btrace-sync-storage',
      partialize: (state) => ({
        pendingQueue: state.pendingQueue,
        lastSyncTime: state.lastSyncTime,
      }),
    }
  )
);
