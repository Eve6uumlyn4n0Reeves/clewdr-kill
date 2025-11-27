import React, { memo } from 'react';
import { useSystemStats } from '../../api/stats';
import { Card, CardHeader, CardTitle, CardContent, Message, Button } from '../ui';
import { ChartPieIcon, ArrowPathIcon } from '@heroicons/react/24/outline';
import { formatTimeElapsed } from '../../utils/formatters';

export const SystemStats: React.FC<{ refreshKey?: number }> = memo(({ refreshKey }) => {
  const { stats, loading, error, refetch } = useSystemStats({ refreshIntervalMs: 15000 });

  // 强制外部刷新时立即触发
  React.useEffect(() => {
    if (refreshKey !== undefined) {
      refetch();
    }
  }, [refreshKey, refetch]);

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ChartPieIcon className="h-5 w-5 text-primary" />
            系统统计
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Message type="error">{error}</Message>
        </CardContent>
      </Card>
    );
  }

  const values = {
    total: stats?.total_requests ?? 0,
    pending: stats?.pending_cookies ?? 0,
    banned: stats?.banned_cookies ?? 0,
    rpm: stats?.requests_per_minute ?? 0,
    success: stats?.success_rate ?? 0,
    avgResp: stats?.average_response_time ?? 0,
    workers: stats?.workers_active ?? 0,
    uptime: stats ? formatTimeElapsed(stats.uptime_seconds) : '-',
  };

  return (
    <Card>
      <CardHeader className="flex items-center justify-between">
        <CardTitle className="flex items-center gap-2">
          <ChartPieIcon className="h-5 w-5 text-primary" />
          系统统计
        </CardTitle>
        <Button
          size="sm"
          variant="ghost"
          onClick={refetch}
          disabled={loading}
          icon={<ArrowPathIcon className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />}
        >
          刷新
        </Button>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <StatBlock label="总请求" value={values.total.toLocaleString()} />
          <StatBlock label="成功率" value={`${values.success.toFixed(1)}%`} />
          <StatBlock label="平均响应" value={`${values.avgResp} ms`} />
          <StatBlock label="待处理" value={values.pending} />
          <StatBlock label="已封禁" value={values.banned} />
          <StatBlock label="每分钟请求" value={values.rpm.toFixed(1)} />
          <StatBlock label="活跃 Worker" value={values.workers} />
          <StatBlock label="运行时间" value={values.uptime} />
        </div>
      </CardContent>
    </Card>
  );
});

SystemStats.displayName = 'SystemStats';

const StatBlock: React.FC<{ label: string; value: string | number }> = ({ label, value }) => (
  <div className="p-4 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
    <div className="text-sm text-gray-500 dark:text-gray-400">{label}</div>
    <div className="mt-1 text-xl font-semibold text-gray-900 dark:text-white">{value}</div>
  </div>
);

export default SystemStats;
