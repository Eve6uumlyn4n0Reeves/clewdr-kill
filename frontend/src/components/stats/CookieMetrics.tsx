import React, { useState, useEffect, useCallback, memo } from 'react';
import { apiClient } from '../../api';
import type { CookieMetrics } from '../../types/api.types';
import { Card, CardHeader, CardTitle, CardContent, Table, Button, Message } from '../ui';
import {
  TableCellsIcon,
  ArrowPathIcon,
  ChartBarIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';

const CookieMetrics: React.FC = memo(() => {
  const [metrics, setMetrics] = useState<CookieMetrics[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<keyof CookieMetrics>('requests_sent');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const fetchMetrics = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.getCookieMetrics();
      setMetrics(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch cookie metrics');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchMetrics();
    // 每30秒自动刷新
    const interval = setInterval(fetchMetrics, 30000);
    return () => clearInterval(interval);
  }, [fetchMetrics]);

  const handleSort = (column: keyof CookieMetrics) => {
    if (sortBy === column) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(column);
      setSortOrder('desc');
    }
  };

  const sortedMetrics = [...metrics].sort((a, b) => {
    const aValue = a[sortBy];
    const bValue = b[sortBy];

    if (typeof aValue === 'string' && typeof bValue === 'string') {
      return sortOrder === 'asc'
        ? aValue.localeCompare(bValue)
        : bValue.localeCompare(aValue);
    }

    if (typeof aValue === 'number' && typeof bValue === 'number') {
      return sortOrder === 'asc' ? aValue - bValue : bValue - aValue;
    }

    return 0;
  });

  const formatResponseTime = (ms: number): string => {
    if (ms < 1000) {
      return `${ms.toFixed(0)}ms`;
    } else {
      return `${(ms / 1000).toFixed(2)}s`;
    }
  };

  const getSuccessRate = (metric: CookieMetrics): number => {
    const total = metric.requests_sent;
    if (total === 0) return 0;
    return (metric.successful_requests / total) * 100;
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'active':
        return 'text-green-600 dark:text-green-400';
      case 'processing':
        return 'text-primary-600 dark:text-primary-400';
      case 'pending':
        return 'text-yellow-600 dark:text-yellow-400';
      case 'banned':
        return 'text-red-600 dark:text-red-400';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  };

  const getStatusText = (status: string): string => {
    switch (status) {
      case 'active':
        return '活跃';
      case 'processing':
        return '处理中';
      case 'pending':
        return '待处理';
      case 'banned':
        return '已封禁';
      default:
        return status;
    }
  };

  const columns = [
    {
      key: 'cookie_id',
      title: 'Cookie ID',
      sortable: true,
      render: (value: string) => (
        <span className="font-mono text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
          {value.slice(0, 12)}...{value.slice(-12)}
        </span>
      ),
    },
    {
      key: 'requests_sent',
      title: '总请求',
      sortable: true,
      render: (value: number) => (
        <span className="font-medium">{value.toLocaleString()}</span>
      ),
    },
    {
      key: 'successful_requests',
      title: '成功',
      sortable: true,
      render: (value: number) => (
        <span className="text-green-600 dark:text-green-400">{value.toLocaleString()}</span>
      ),
    },
    {
      key: 'failed_requests',
      title: '失败',
      sortable: true,
      render: (value: number, record: CookieMetrics) => (
        <span className={value > 0 ? 'text-red-600 dark:text-red-400' : 'text-gray-500'}>
          {value.toLocaleString()}
          {value > 0 && (
            <span className="text-xs ml-1">
              ({((value / record.requests_sent) * 100).toFixed(1)}%)
            </span>
          )}
        </span>
      ),
    },
    {
      key: 'success_rate',
      title: '成功率',
      sortable: false,
      render: (_: any, record: CookieMetrics) => {
        const rate = getSuccessRate(record);
        return (
          <div className="flex items-center gap-2">
            <div className="w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${
                  rate >= 90 ? 'bg-green-500' : rate >= 70 ? 'bg-yellow-500' : 'bg-red-500'
                }`}
                style={{ width: `${rate}%` }}
              />
            </div>
            <span className="text-sm font-medium">{rate.toFixed(1)}%</span>
          </div>
        );
      },
    },
    {
      key: 'average_response_time',
      title: '平均响应',
      sortable: true,
      render: (value: number) => (
        <span className="text-sm">{formatResponseTime(value)}</span>
      ),
    },
    {
      key: 'status',
      title: '状态',
      sortable: true,
      render: (value: string) => (
        <span className={`font-medium ${getStatusColor(value)}`}>
          {getStatusText(value)}
        </span>
      ),
    },
    {
      key: 'last_request_time',
      title: '最后请求',
      sortable: true,
      render: (value?: string) => (
        <span className="text-sm text-gray-600 dark:text-gray-400">
          {value ? new Date(value).toLocaleString() : '从未'}
        </span>
      ),
    },
  ];

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <TableCellsIcon className="h-5 w-5 text-primary" />
            Cookie 详细指标
          </CardTitle>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-500">
              共 {metrics.length} 个Cookie
            </span>
            <Button
              variant="ghost"
              size="sm"
              onClick={fetchMetrics}
              disabled={loading}
              icon={<ArrowPathIcon className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />}
            >
              刷新
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {error && (
          <Message type="error" className="mb-4">
            <div className="flex items-center gap-2">
              <ExclamationTriangleIcon className="h-4 w-4" />
              {error}
            </div>
          </Message>
        )}

        <Table
          data={sortedMetrics}
          columns={columns}
          loading={loading}
          emptyMessage="暂无Cookie指标数据"
        />

        {/* 统计摘要 */}
        {!loading && metrics.length > 0 && (
          <div className="mt-6 grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <p className="text-sm text-gray-500">平均成功率</p>
              <p className="text-lg font-semibold">
                {(metrics.reduce((sum, m) => sum + getSuccessRate(m), 0) / metrics.length).toFixed(1)}%
              </p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">平均响应时间</p>
              <p className="text-lg font-semibold">
                {formatResponseTime(metrics.reduce((sum, m) => sum + m.average_response_time, 0) / metrics.length)}
              </p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">总请求数</p>
              <p className="text-lg font-semibold">
                {metrics.reduce((sum, m) => sum + m.requests_sent, 0).toLocaleString()}
              </p>
            </div>
            <div className="text-center">
              <p className="text-sm text-gray-500">活跃Cookie</p>
              <p className="text-lg font-semibold text-green-600">
                {metrics.filter(m => m.status === 'active').length}
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
});

CookieMetrics.displayName = 'CookieMetrics';

export default CookieMetrics;
