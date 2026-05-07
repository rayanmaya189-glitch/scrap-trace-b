import { apiClient } from './client';
import { useSyncStore, PendingHandshake } from '../store/syncStore';
import { incrementClock, compareVersionVectors, resolveHandshakeConflict } from '../lib/crdt';

const MAX_RETRIES = 5;
const SYNC_INTERVAL_MS = 5000;

/**
 * Sync Service for B-Trace Protocol
 * Handles offline queue synchronization with conflict resolution
 */
export class SyncService {
  private static instance: SyncService;
  private syncInterval: NodeJS.Timeout | null = null;
  private isProcessing = false;

  private constructor() {}

  public static getInstance(): SyncService {
    if (!SyncService.instance) {
      SyncService.instance = new SyncService();
    }
    return SyncService.instance;
  }

  /**
   * Start the background sync process
   */
  public startSyncLoop() {
    if (this.syncInterval) {
      console.warn('[SyncService] Sync loop already running');
      return;
    }

    console.log('[SyncService] Starting sync loop');
    this.syncInterval = setInterval(() => {
      this.processQueue();
    }, SYNC_INTERVAL_MS);

    // Process immediately on start
    this.processQueue();
  }

  /**
   * Stop the background sync process
   */
  public stopSyncLoop() {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
      console.log('[SyncService] Sync loop stopped');
    }
  }

  /**
   * Process the pending handshake queue
   */
  private async processQueue() {
    if (this.isProcessing) {
      return;
    }

    const { pendingQueue, updateQueueStatus, setSyncing, setSyncError } = useSyncStore.getState();
    const pendingItems = pendingQueue.filter((item) => item.status === 'pending' || item.status === 'syncing');

    if (pendingItems.length === 0) {
      return;
    }

    this.isProcessing = true;
    setSyncing(true);

    try {
      for (const item of pendingItems) {
        if (item.retryCount >= MAX_RETRIES) {
          updateQueueStatus(item.id, 'failed');
          console.warn(`[SyncService] Item ${item.id} failed after ${MAX_RETRIES} retries`);
          continue;
        }

        updateQueueStatus(item.id, 'syncing');

        try {
          await this.syncHandshake(item);
          updateQueueStatus(item.id, 'synced');
          console.log(`[SyncService] Successfully synced handshake ${item.id}`);
        } catch (error) {
          console.error(`[SyncService] Failed to sync handshake ${item.id}:`, error);
          useSyncStore.getState().incrementRetry(item.id);
          
          if (error instanceof ConflictError) {
            updateQueueStatus(item.id, 'conflict');
            // Conflicts need manual resolution or special handling
            await this.handleConflict(item, error);
          }
        }
      }

      useSyncStore.getState().setLastSyncTime(Date.now());
      setSyncError(null);
    } catch (error) {
      setSyncError(error instanceof Error ? error.message : 'Unknown sync error');
    } finally {
      setSyncing(false);
      this.isProcessing = false;
    }
  }

  /**
   * Sync a single handshake to the backend
   */
  private async syncHandshake(handshake: PendingHandshake) {
    const response = await apiClient.post('/handshakes/confirm', {
      id: handshake.id,
      material_id: handshake.materialId,
      supplier_sig: handshake.supplierSig,
      buyer_sig: handshake.buyerSig,
      payload_hash: handshake.payloadHash,
      hash_prev: handshake.hashPrev,
      hash_current: handshake.hashCurrent,
      version_vector: handshake.versionVector,
      timestamp_utc: new Date(handshake.timestamp).toISOString(),
    });

    if (!response.ok) {
      if (response.status === 409) {
        const existing = await response.json();
        throw new ConflictError('Concurrent handshake detected', existing);
      }
      throw new Error(`Sync failed with status ${response.status}`);
    }

    return response.json();
  }

  /**
   * Handle conflict resolution
   */
  private async handleConflict(handshake: PendingHandshake, error: ConflictError) {
    console.log(`[SyncService] Handling conflict for handshake ${handshake.id}`);
    
    const resolved = resolveHandshakeConflict(
      {
        id: handshake.id,
        supplierSig: handshake.supplierSig,
        buyerSig: handshake.buyerSig,
        timestamp: handshake.timestamp,
      },
      {
        id: error.existing.id,
        supplierSig: error.existing.supplier_sig,
        buyerSig: error.existing.buyer_sig,
        timestamp: new Date(error.existing.timestamp_utc).getTime(),
      }
    );

    // If our handshake won the resolution, retry sync
    if (resolved.id === handshake.id) {
      console.log(`[SyncService] Local handshake won conflict resolution, retrying...`);
      useSyncStore.getState().updateQueueStatus(handshake.id, 'pending');
    } else {
      console.log(`[SyncService] Remote handshake won conflict resolution, accepting remote version`);
      // Accept remote version - could update local state here
      useSyncStore.getState().removeFromQueue(handshake.id);
    }
  }

  /**
   * Add a handshake to the sync queue
   */
  public queueHandshake(handshake: Omit<PendingHandshake, 'status' | 'retryCount'>) {
    useSyncStore.getState().addToQueue(handshake);
    console.log(`[SyncService] Queued handshake ${handshake.id} for sync`);
    
    // Attempt immediate sync if online
    if (navigator.onLine) {
      this.processQueue();
    }
  }

  /**
   * Get sync status
   */
  public getSyncStatus() {
    const { pendingQueue, isSyncing, lastSyncTime, syncError } = useSyncStore.getState();
    const pendingCount = pendingQueue.filter((item) => item.status === 'pending').length;
    const conflictCount = pendingQueue.filter((item) => item.status === 'conflict').length;
    const failedCount = pendingQueue.filter((item) => item.status === 'failed').length;

    return {
      isSyncing,
      pendingCount,
      conflictCount,
      failedCount,
      lastSyncTime,
      syncError,
      totalPending: pendingQueue.length,
    };
  }
}

/**
 * Custom error class for conflict detection
 */
export class ConflictError extends Error {
  constructor(message: string, public existing: any) {
    super(message);
    this.name = 'ConflictError';
  }
}

// Export singleton instance
export const syncService = SyncService.getInstance();
