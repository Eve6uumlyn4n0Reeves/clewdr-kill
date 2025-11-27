import { useState, useCallback, useEffect, useRef } from 'react';
import { apiClient } from "./client";
import type {
  SystemStats,
  CookieMetrics,
  HistoricalStats,
  HistoricalStatsParams,
} from "../types/api.types";

export const statsApi = {
  /**
   * 获取系统统计数据
   */
  async getSystemStats(): Promise<SystemStats> {
    return apiClient.get<SystemStats>("/stats/system");
  },

  /**
   * 获取Cookie详细指标
   */
  async getCookieMetrics(): Promise<CookieMetrics[]> {
    return apiClient.get<CookieMetrics[]>("/stats/cookies");
  },

  /**
   * 获取历史统计数据
   */
  async getHistoricalStats(params: HistoricalStatsParams): Promise<HistoricalStats> {
    return apiClient.post<HistoricalStats>("/stats/historical", params);
  },

  /**
   * 重置统计数据
   */
  async resetStats(): Promise<void> {
    await apiClient.post("/stats/reset");
  },
};

// 便捷的 Hook 函数
type UseSystemStatsOptions = {
  refreshIntervalMs?: number;
};

type CachedStats = {
  data: SystemStats | null;
  timestamp: number;
};

const statsCache: CachedStats = {
  data: null,
  timestamp: 0,
};

// 仅测试使用：重置缓存
export const __resetSystemStatsCache = () => {
  statsCache.data = null;
  statsCache.timestamp = 0;
};

export const useSystemStats = (options?: UseSystemStatsOptions) => {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const refreshInterval = Math.max(options?.refreshIntervalMs ?? 15000, 5000);
  const inFlight = useRef<Promise<void> | null>(null);

  const fetchStats = useCallback(async () => {
    const now = Date.now();
    if (statsCache.data && now - statsCache.timestamp < refreshInterval) {
      setStats(statsCache.data);
      setLoading(false);
      return;
    }

    if (inFlight.current) {
      await inFlight.current;
      return;
    }

    const promise = (async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await statsApi.getSystemStats();
        statsCache.data = data;
        statsCache.timestamp = Date.now();
        setStats(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch stats');
      } finally {
        setLoading(false);
        inFlight.current = null;
      }
    })();

    inFlight.current = promise;
    await promise;
  }, [refreshInterval]);

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, refreshInterval);
    return () => clearInterval(interval);
  }, [fetchStats, refreshInterval]);

  return { stats, loading, error, refetch: fetchStats };
};

export const useHistoricalStats = (params: HistoricalStatsParams) => {
  const [data, setData] = useState<HistoricalStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await statsApi.getHistoricalStats(params);
      setData(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch historical stats');
    } finally {
      setLoading(false);
    }
  }, [params]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
};
