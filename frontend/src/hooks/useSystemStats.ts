import { useState, useEffect, useCallback } from 'react';
import { apiClient } from '../api';
import type { SystemStats } from '../types/api.types';

interface UseSystemStatsOptions {
  refreshIntervalMs?: number;
  enabled?: boolean;
}

interface UseSystemStatsReturn {
  stats: SystemStats | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useSystemStats(options: UseSystemStatsOptions = {}): UseSystemStatsReturn {
  const { refreshIntervalMs = 30000, enabled = true } = options;

  const [stats, setStats] = useState<SystemStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = useCallback(async () => {
    if (!enabled) return;

    try {
      setError(null);
      const data = await apiClient.getSystemStats();
      setStats(data);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch system stats';
      setError(message);
      console.error('Failed to fetch system stats:', err);
    } finally {
      setLoading(false);
    }
  }, [enabled]);

  const refresh = useCallback(async () => {
    setLoading(true);
    await fetchStats();
  }, [fetchStats]);

  useEffect(() => {
    if (!enabled) return;

    // Initial fetch
    fetchStats();

    // Set up interval for periodic updates
    if (refreshIntervalMs > 0) {
      const interval = setInterval(fetchStats, refreshIntervalMs);
      return () => clearInterval(interval);
    }
  }, [fetchStats, refreshIntervalMs, enabled]);

  return {
    stats,
    loading,
    error,
    refresh,
  };
}
