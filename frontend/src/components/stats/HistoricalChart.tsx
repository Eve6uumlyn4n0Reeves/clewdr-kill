import React, { useState, useEffect, useCallback, memo } from 'react';
import { statsApi } from '../../api/stats';
import type { HistoricalStats, HistoricalStatsParams } from '../../types/api.types';
import { Card, CardHeader, CardTitle, CardContent } from '../ui';
import {
  ChartBarIcon,
  CalendarIcon,
  ArrowPathIcon
} from '@heroicons/react/24/outline';

const HistoricalChart: React.FC = memo(() => {
  const [data, setData] = useState<HistoricalStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [params, setParams] = useState<HistoricalStatsParams>({
    interval_minutes: 60, // 1小时间隔
    points: 24, // 24个数据点（24小时）
  });

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await statsApi.getHistoricalStats(params);
      setData(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch historical data');
    } finally {
      setLoading(false);
    }
  }, [params]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // 简单的条形图渲染
  const renderBarChart = (values: number[], label: string, color: string) => {
    if (!values.length) return null;

    const max = Math.max(...values);
    const min = Math.min(...values);
    const range = max - min || 1;

    return (
      <div className="space-y-2">
        <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300">{label}</h4>
        <div className="flex items-end space-x-1 h-32">
          {values.map((value, index) => {
            const height = ((value - min) / range) * 100;
            return (
              <div
                key={index}
                className={`flex-1 ${color} rounded-t relative group`}
                style={{ height: `${height}%` }}
              >
                <div className="absolute -top-6 left-1/2 transform -translate-x-1/2
                               bg-gray-900 text-white text-xs px-2 py-1 rounded
                               opacity-0 group-hover:opacity-100 transition-opacity
                               whitespace-nowrap z-10">
                  {value.toFixed(1)}
                </div>
              </div>
            );
          })}
        </div>
        <div className="flex justify-between text-xs text-gray-500">
          <span>{params.points ? params.points : 0} 个数据点</span>
          <span>每 {params.interval_minutes} 分钟</span>
        </div>
      </div>
    );
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <ChartBarIcon className="h-5 w-5 text-primary" />
            历史统计图表
          </CardTitle>
          <div className="flex items-center gap-4">
            <select
              value={params.interval_minutes}
              onChange={(e) => setParams({ ...params, interval_minutes: Number(e.target.value) })}
              className="text-sm border border-gray-300 rounded px-2 py-1 dark:bg-gray-800 dark:border-gray-600"
            >
              <option value={5}>5分钟</option>
              <option value={15}>15分钟</option>
              <option value={30}>30分钟</option>
              <option value={60}>1小时</option>
              <option value={240}>4小时</option>
              <option value={1440}>1天</option>
            </select>
            <select
              value={params.points}
              onChange={(e) => setParams({ ...params, points: Number(e.target.value) })}
              className="text-sm border border-gray-300 rounded px-2 py-1 dark:bg-gray-800 dark:border-gray-600"
            >
              <option value={12}>12个点</option>
              <option value={24}>24个点</option>
              <option value={48}>48个点</option>
              <option value={72}>72个点</option>
              <option value={168}>168个点</option>
            </select>
            <button
              onClick={fetchData}
              disabled={loading}
              className="p-1 hover:bg-gray-100 rounded transition-colors"
            >
              <ArrowPathIcon className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            </button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading && !data && (
          <div className="flex justify-center items-center h-64">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        )}

        {error && (
          <div className="text-center text-red-600 py-8">
            <CalendarIcon className="h-12 w-12 mx-auto mb-2" />
            <p>加载历史数据失败</p>
            <p className="text-sm mt-1">{error}</p>
          </div>
        )}

        {data && (
          <div className="space-y-8">
            {/* 请求数量图表 */}
            {renderBarChart(data.request_counts, '请求数量', 'bg-blue-500')}

            {/* 成功率图表 */}
            {renderBarChart(data.success_rates, '成功率 (%)', 'bg-green-500')}

            {/* 响应时间图表 */}
            {renderBarChart(data.response_times, '响应时间 (ms)', 'bg-orange-500')}

            {/* 错误率图表 */}
            {renderBarChart(data.error_rates, '错误率 (%)', 'bg-red-500')}

            {/* 时间轴显示 */}
            {data.timestamps.length > 0 && (
              <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">时间范围</h4>
                <div className="flex justify-between text-xs text-gray-500">
                  <span>{new Date(data.timestamps[0]).toLocaleString()}</span>
                  <span>{new Date(data.timestamps[data.timestamps.length - 1]).toLocaleString()}</span>
                </div>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
});

HistoricalChart.displayName = 'HistoricalChart';

export default HistoricalChart;
