import { create } from 'zustand';
import PouchDB from 'pouchdb-browser';

// Initialize local databases for offline-first architecture
const offlineMaterialsDb = new PouchDB('btrace_materials');
const offlineHandshakesDb = new PouchDB('btrace_handshakes');
const offlineQueueDb = new PouchDB('btrace_sync_queue');

// Types
export interface Material {
  id: string;
  materialType: string;
  batchWeightKg: number;
  materialGrade: string;
  sourcePincode: string;
  supplierId: string;
  buyerId?: string;
  status: 'PENDING' | 'CONFIRMED' | 'DISPUTED' | 'COMPLETED' | 'CANCELLED';
  createdAt: string;
  updatedAt: string;
}

export interface Handshake {
  id: string;
  materialId: string;
  supplierSig: string;
  buyerSig: string;
  payloadHash: string;
  hashPrev: string;
  hashCurrent: string;
  versionVector: Record<string, number>;
  syncStatus: 'LOCAL' | 'SYNCING' | 'SYNCED' | 'CONFLICT' | 'DISPUTED';
  timestampUtc: string;
}

interface SyncQueueItem {
  _id: string;
  _rev?: string;
  action: 'CREATE_MATERIAL' | 'UPDATE_MATERIAL' | 'CREATE_HANDSHAKE' | 'CONFIRM_HANDSHAKE';
  data: any;
  timestamp: number;
  retryCount: number;
  lastError?: string;
}

interface SyncState {
  // Connection state
  isOnline: boolean;
  isSyncing: boolean;
  syncProgress: number;
  
  // Queue stats
  pendingItems: number;
  failedItems: number;
  
  // Actions
  checkConnectivity: () => void;
  addToQueue: (action: SyncQueueItem['action'], data: any) => Promise<string>;
  processQueue: () => Promise<void>;
  getPendingItems: () => Promise<SyncQueueItem[]>;
  removeFromQueue: (id: string) => Promise<void>;
  updateSyncProgress: (progress: number) => void;
  
  // Material operations (offline-first)
  saveMaterialOffline: (material: Material) => Promise<void>;
  getOfflineMaterials: () => Promise<Material[]>;
  getMaterialById: (id: string) => Promise<Material | undefined>;
  
  // Handshake operations (offline-first)
  saveHandshakeOffline: (handshake: Handshake) => Promise<void>;
  getOfflineHandshakes: () => Promise<Handshake[]>;
  updateHandshakeStatus: (id: string, status: Handshake['syncStatus']) => Promise<void>;
}

export const useSyncStore = create<SyncState>((set, get) => ({
  isOnline: typeof navigator !== 'undefined' ? navigator.onLine : true,
  isSyncing: false,
  syncProgress: 0,
  pendingItems: 0,
  failedItems: 0,

  checkConnectivity: () => {
    const isOnline = typeof navigator !== 'undefined' ? navigator.onLine : true;
    set({ isOnline });
    
    // If we're back online, trigger sync
    if (isOnline && !get().isSyncing) {
      get().processQueue();
    }
    
    // Update queue stats
    get().updateQueueStats();
  },

  addToQueue: async (action, data) => {
    const id = `queue_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const item: SyncQueueItem = {
      _id: id,
      action,
      data,
      timestamp: Date.now(),
      retryCount: 0,
    };
    
    await offlineQueueDb.put(item);
    await get().updateQueueStats();
    
    // If online, try to process immediately
    if (get().isOnline && !get().isSyncing) {
      get().processQueue();
    }
    
    return id;
  },

  processQueue: async () => {
    if (!get().isOnline || get().isSyncing) {
      return;
    }

    set({ isSyncing: true, syncProgress: 0 });

    try {
      const items = await get().getPendingItems();
      const total = items.length;
      
      for (let i = 0; i < total; i++) {
        const item = items[i];
        set({ syncProgress: ((i + 1) / total) * 100 });

        try {
          await get().executeQueueItem(item);
          // Remove successful item
          await get().removeFromQueue(item._id);
        } catch (error: any) {
          console.error(`Failed to process queue item ${item._id}:`, error);
          
          // Update retry count
          const maxRetries = 5;
          if (item.retryCount >= maxRetries) {
            // Mark as failed, don't retry
            await offlineQueueDb.remove(item._id, item._rev!);
            set((state) => ({ failedItems: state.failedItems + 1 }));
          } else {
            // Increment retry and update
            await offlineQueueDb.put({
              ...item,
              retryCount: item.retryCount + 1,
              lastError: error.message,
            });
          }
        }
      }
    } finally {
      set({ isSyncing: false });
      await get().updateQueueStats();
    }
  },

  executeQueueItem: async (item: SyncQueueItem) => {
    const { apiClient } = await import('./useAuthStore');
    
    switch (item.action) {
      case 'CREATE_MATERIAL':
        await apiClient.post('/v1/materials', item.data);
        // Save to local DB after successful sync
        await offlineMaterialsDb.put({
          _id: item.data.id,
          ...item.data,
          syncedAt: new Date().toISOString(),
        });
        break;
        
      case 'UPDATE_MATERIAL':
        await apiClient.put(`/v1/materials/${item.data.id}`, item.data);
        break;
        
      case 'CREATE_HANDSHAKE':
        await apiClient.post('/v1/handshakes/initiate', item.data);
        break;
        
      case 'CONFIRM_HANDSHAKE':
        await apiClient.post('/v1/handshakes/confirm', item.data);
        break;
        
      default:
        throw new Error(`Unknown queue action: ${item.action}`);
    }
  },

  getPendingItems: async () => {
    const result = await offlineQueueDb.allDocs({ 
      include_docs: true,
      startkey: 'queue_',
      endkey: 'queue_\ufff0',
    });
    return result.rows.map(row => row.doc as SyncQueueItem);
  },

  removeFromQueue: async (id: string) => {
    try {
      const doc = await offlineQueueDb.get(id);
      await offlineQueueDb.remove(doc);
      await get().updateQueueStats();
    } catch (error: any) {
      if (error.status !== 404) {
        throw error;
      }
    }
  },

  updateQueueStats: async () => {
    try {
      const result = await offlineQueueDb.allDocs({ 
        startkey: 'queue_',
        endkey: 'queue_\ufff0',
      });
      set({ pendingItems: result.total_rows });
    } catch (error) {
      console.error('Failed to update queue stats:', error);
    }
  },

  updateSyncProgress: (progress: number) => {
    set({ syncProgress: progress });
  },

  // Material operations
  saveMaterialOffline: async (material: Material) => {
    await offlineMaterialsDb.put({
      _id: material.id,
      ...material,
      isOffline: true,
      savedAt: new Date().toISOString(),
    });
  },

  getOfflineMaterials: async () => {
    const result = await offlineMaterialsDb.allDocs({ 
      include_docs: true,
      startkey: 'material_',
      endkey: 'material_\ufff0',
    });
    return result.rows.map(row => row.doc as Material);
  },

  getMaterialById: async (id: string) => {
    try {
      const doc = await offlineMaterialsDb.get(id);
      return doc as Material;
    } catch (error: any) {
      if (error.status === 404) {
        return undefined;
      }
      throw error;
    }
  },

  // Handshake operations
  saveHandshakeOffline: async (handshake: Handshake) => {
    await offlineHandshakesDb.put({
      _id: handshake.id,
      ...handshake,
      isOffline: true,
      savedAt: new Date().toISOString(),
    });
  },

  getOfflineHandshakes: async () => {
    const result = await offlineHandshakesDb.allDocs({ 
      include_docs: true,
      startkey: 'handshake_',
      endkey: 'handshake_\ufff0',
    });
    return result.rows.map(row => row.doc as Handshake);
  },

  updateHandshakeStatus: async (id: string, status: Handshake['syncStatus']) => {
    try {
      const doc = await offlineHandshakesDb.get(id);
      await offlineHandshakesDb.put({
        ...doc,
        syncStatus: status,
        updatedAt: new Date().toISOString(),
      });
    } catch (error: any) {
      if (error.status !== 404) {
        throw error;
      }
    }
  },
}));

// Listen for online/offline events
if (typeof window !== 'undefined') {
  window.addEventListener('online', () => {
    useSyncStore.getState().checkConnectivity();
  });
  
  window.addEventListener('offline', () => {
    useSyncStore.getState().checkConnectivity();
  });
  
  // Initial connectivity check
  useSyncStore.getState().checkConnectivity();
}
