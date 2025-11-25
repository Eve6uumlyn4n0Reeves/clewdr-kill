import { apiClient } from "./client";

export interface SystemStats {
  total_cookies: number;
  pending_cookies: number;
  banned_cookies: number;
  total_requests: number;
  requests_per_minute: number;
  success_rate: number;
  average_response_time: number;
  workers_active: number;
  uptime_seconds: number;
  last_update: string;
  error_distribution: Record<string, number>;
  performance_metrics: {
    cpu_usage: number;
    memory_usage: number;
    network_latency: number;
    queue_processing_time: number;
    strategy_effectiveness: number;
  };
}

export interface CookieMetrics {
  cookie_id: string;
  requests_sent: number;
  successful_requests: number;
  failed_requests: number;
  average_response_time: number;
  last_request_time?: string;
  consecutive_errors: number;
  adaptive_delay: number;
  status: 'pending' | 'banned' | 'active';
}

export interface HistoricalStats {
  timestamps: string[];
  request_counts: number[];
  success_rates: number[];
  response_times: number[];
  error_rates: number[];
}

export interface HistoricalStatsParams {
  interval_minutes: number;
  points?: number;
  start_time?: string;
  end_time?: string;
}

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

// 便捷的Hook函数
export const useSystemStats = () => {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await statsApi.getSystemStats();
      setStats(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch stats');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 5000); // 每5秒刷新
    return () => clearInterval(interval);
  }, [fetchStats]);

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

// 为了支持这些Hook，需要导入React
import { useState, useCallback, useEffect } from 'react';
