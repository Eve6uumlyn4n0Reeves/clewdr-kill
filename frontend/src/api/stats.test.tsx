import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useSystemStats, __resetSystemStatsCache } from './stats';

const mocks = vi.hoisted(() => ({
  get: vi.fn(),
}));

vi.mock('./client', () => ({
  apiClient: {
    get: (...args: unknown[]) => mocks.get(...(args as [])),
  },
}));

const statsSample = {
  total_requests: 1,
  pending_cookies: 0,
  banned_cookies: 0,
  requests_per_minute: 1,
  success_rate: 100,
  average_response_time: 10,
  workers_active: 1,
  uptime_seconds: 5,
  last_update: new Date().toISOString(),
  error_distribution: {},
  performance_metrics: {
    cpu_usage: 1,
    memory_usage: 1,
    network_latency: 1,
    queue_processing_time: 1,
    strategy_effectiveness: 1,
  },
};

describe('useSystemStats cache', () => {
  beforeEach(() => {
    mocks.get.mockResolvedValue(statsSample);
    mocks.get.mockClear();
    __resetSystemStatsCache();
  });

  it('避免短时间重复请求', async () => {
    const { result } = renderHook(() => useSystemStats({ refreshIntervalMs: 10000 }));

    await act(async () => {
      await result.current.refetch(); // 第一次主动拉取
    });
    mocks.get.mockClear(); // 清零以观察后续调用

    await act(async () => {
      await result.current.refetch();
      await result.current.refetch();
    });

    expect(mocks.get).toHaveBeenCalledTimes(0);
  });

  it('缓存过期后会重新拉取', async () => {
    const { result } = renderHook(() => useSystemStats({ refreshIntervalMs: 10000 }));
    await act(async () => {
      await result.current.refetch(); // 初次拉取
    });

    mocks.get.mockClear();
    mocks.get.mockResolvedValueOnce({ ...statsSample, total_requests: 2 });

    // 直接手动将缓存视为过期：调整内部时间戳
    __resetSystemStatsCache();

    await act(async () => {
      await result.current.refetch();
    });

    expect(mocks.get).toHaveBeenCalledTimes(1);
  });

  it('请求失败时返回错误信息并不中毒缓存', async () => {
    mocks.get.mockRejectedValueOnce(new Error('network boom'));
    const { result } = renderHook(() => useSystemStats({ refreshIntervalMs: 10000 }));

    await act(async () => {
      await result.current.refetch();
    });

    await waitFor(() => expect(result.current.error).toBeTruthy());
    expect(result.current.stats).toBeNull();
    expect(mocks.get).toHaveBeenCalledTimes(1);
  });
});
