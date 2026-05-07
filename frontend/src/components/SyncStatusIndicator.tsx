import React from 'react';
import { Card, CardContent } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { useSyncStatus } from '../hooks/useSyncManager';
import { Wifi, WifiOff, AlertTriangle, CheckCircle, Clock } from 'lucide-react';

export function SyncStatusIndicator() {
  const { isSyncing, pendingCount, lastSyncTime, syncError, hasPending, hasConflicts, isOnline } = useSyncStatus();

  const getStatusIcon = () => {
    if (!isOnline) {
      return <WifiOff className="h-4 w-4 text-gray-400" />;
    }
    if (isSyncing) {
      return <Wifi className="h-4 w-4 text-blue-500 animate-pulse" />;
    }
    if (syncError || hasConflicts) {
      return <AlertTriangle className="h-4 w-4 text-red-500" />;
    }
    if (hasPending) {
      return <Clock className="h-4 w-4 text-yellow-500" />;
    }
    return <CheckCircle className="h-4 w-4 text-green-500" />;
  };

  const getStatusText = () => {
    if (!isOnline) return 'Offline';
    if (isSyncing) return 'Syncing...';
    if (syncError) return 'Sync Error';
    if (hasConflicts) return `${pendingCount} Conflict${pendingCount > 1 ? 's' : ''}`;
    if (hasPending) return `${pendingCount} Pending`;
    if (lastSyncTime) {
      const minutes = Math.floor((Date.now() - lastSyncTime) / 60000);
      if (minutes < 1) return 'Just now';
      if (minutes < 60) return `${minutes}m ago`;
      const hours = Math.floor(minutes / 60);
      return `${hours}h ago`;
    }
    return 'Never synced';
  };

  const getBadgeVariant = (): 'default' | 'secondary' | 'destructive' | 'outline' => {
    if (!isOnline) return 'secondary';
    if (syncError || hasConflicts) return 'destructive';
    if (isSyncing || hasPending) return 'default';
    return 'default';
  };

  return (
    <Card className="w-full max-w-sm">
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {getStatusIcon()}
            <span className="text-sm font-medium">{getStatusText()}</span>
          </div>
          <Badge variant={getBadgeVariant()}>
            {!isOnline ? 'OFFLINE' : isSyncing ? 'SYNCING' : hasPending ? 'PENDING' : 'READY'}
          </Badge>
        </div>
        
        {(hasPending || hasConflicts) && (
          <div className="mt-3 space-y-2">
            {hasPending && (
              <div className="flex items-center gap-2 text-xs text-yellow-600">
                <Clock className="h-3 w-3" />
                <span>{pendingCount} item(s) waiting to sync</span>
              </div>
            )}
            {hasConflicts && (
              <div className="flex items-center gap-2 text-xs text-red-600">
                <AlertTriangle className="h-3 w-3" />
                <span>Conflicts require attention</span>
              </div>
            )}
          </div>
        )}
        
        {syncError && (
          <div className="mt-2 text-xs text-red-500">
            Error: {syncError}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

/**
 * Simple inline sync indicator for headers/toolbars
 */
export function SyncStatusBadge() {
  const { isOnline, isSyncing, hasPending, hasConflicts } = useSyncStatus();

  if (!isOnline) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 bg-gray-100 rounded-full">
        <WifiOff className="h-3 w-3 text-gray-500" />
        <span className="text-xs text-gray-600">Offline</span>
      </div>
    );
  }

  if (isSyncing) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 bg-blue-100 rounded-full">
        <Wifi className="h-3 w-3 text-blue-600 animate-pulse" />
        <span className="text-xs text-blue-700">Syncing</span>
      </div>
    );
  }

  if (hasConflicts) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 bg-red-100 rounded-full">
        <AlertTriangle className="h-3 w-3 text-red-600" />
        <span className="text-xs text-red-700">Conflict</span>
      </div>
    );
  }

  if (hasPending) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 bg-yellow-100 rounded-full">
        <Clock className="h-3 w-3 text-yellow-600" />
        <span className="text-xs text-yellow-700">Pending</span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-1.5 px-2 py-1 bg-green-100 rounded-full">
      <CheckCircle className="h-3 w-3 text-green-600" />
      <span className="text-xs text-green-700">Synced</span>
    </div>
  );
}
