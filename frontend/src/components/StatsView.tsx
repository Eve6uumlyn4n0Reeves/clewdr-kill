import React, { useState, useCallback } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button, Message } from './ui';
import { TrashIcon, ArrowPathIcon, ChartBarIcon } from '@heroicons/react/24/outline';
import { apiClient } from '../api';
import SystemStats from './stats/SystemStats';
import HistoricalChart from './stats/HistoricalChart';
import CookieMetrics from './stats/CookieMetrics';
import { usePerfMonitor } from '../hooks';

const StatsView: React.FC = () => {
  const [refreshKey, setRefreshKey] = useState(0);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  usePerfMonitor('stats-view');

  const handleRefresh = useCallback(() => {
    setRefreshKey(prev => prev + 1);
    setMessage({ type: 'info', text: '正在刷新数据...' });
    setTimeout(() => setMessage(null), 2000);
  }, []);

  const handleResetStats = async () => {
    if (!window.confirm('确定要重置所有统计数据吗？此操作不可恢复。')) {
      return;
    }

    try {
      await apiClient.resetStats();
      setMessage({ type: 'success', text: '统计数据已重置' });
      handleRefresh(); // 刷新显示
    } catch (err) {
      setMessage({
        type: 'error',
        text: err instanceof Error ? err.message : '重置统计数据失败'
      });
    }
  };

  return (
    <div className="space-y-6">
      {/* 页面标题和操作 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <ChartBarIcon className="h-5 w-5 text-primary" />
              统计与分析
            </CardTitle>
            <div className="flex gap-2">
              <Button
                variant="ghost"
                onClick={handleRefresh}
                icon={<ArrowPathIcon className="h-4 w-4" />}
              >
                刷新所有
              </Button>
              <Button
                variant="danger"
                onClick={handleResetStats}
                icon={<TrashIcon className="h-4 w-4" />}
              >
                重置统计
              </Button>
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* 消息提示 */}
      {message && (
        <Message type={message.type}>
          {message.text}
        </Message>
      )}

      {/* 系统统计概览 */}
      <SystemStats refreshKey={refreshKey} />

      {/* 历史统计图表 */}
      <HistoricalChart />

      {/* Cookie详细指标 */}
      <CookieMetrics />

      {/* 使用说明 */}
      <Card>
        <CardHeader>
          <CardTitle>统计说明</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3 text-sm text-gray-600 dark:text-gray-400">
            <div>
              <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-1">数据更新频率</h4>
              <ul className="list-disc list-inside space-y-1 ml-4">
                <li>系统统计：每15秒自动刷新（手动刷新优先）</li>
                <li>Cookie指标：每30秒自动刷新</li>
                <li>历史数据：手动刷新</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-1">图表说明</h4>
              <ul className="list-disc list-inside space-y-1 ml-4">
                <li>条形图显示选定时间范围内的数据趋势</li>
                <li>鼠标悬停在条形图上可查看具体数值</li>
                <li>可调整时间间隔和数据点数量</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-1">重置统计</h4>
              <p>重置操作将清零所有累计的统计数据，包括请求数、成功率等。建议在新的测试阶段开始前执行此操作。</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default StatsView;
