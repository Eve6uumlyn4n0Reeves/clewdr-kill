import React, { useState, useEffect, useCallback, memo } from 'react';
import {
  TrashIcon,
  ArrowPathIcon,
  XCircleIcon,
  ClockIcon,
  ArrowTrendingUpIcon,
} from '@heroicons/react/24/outline';
import { formatDistanceToNow } from 'date-fns';
import { zhCN } from 'date-fns/locale';
import { Button, Table, Message, Card, CardHeader, CardTitle, Input } from '../ui';
import { apiClient } from '../../api';
import type { QueueCookie, QueueStatusResponse } from '../../types/api.types';
import { useAsyncOperation } from '../../hooks/useAsyncOperation';
import { useDebounce } from '../../hooks/useDebounce';
import CookieTableSkeleton from './Skeleton';

interface CookieListProps {
  refreshKey?: number;
}

const CookieList = memo<CookieListProps>(({ refreshKey }) => {
  const [cookies, setCookies] = useState<QueueStatusResponse>({
    pending: [],
    processing: [],
    banned: [],
    total_requests: 0,
  });
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [checking, setChecking] = useState<string[]>([]);
  const [deleting, setDeleting] = useState<string[]>([]);
  const [activeTab, setActiveTab] = useState<'pending' | 'processing' | 'banned'>('pending');
  const [search, setSearch] = useState('');
  const debouncedSearch = useDebounce(search, 300);

  const { execute: executeAsync } = useAsyncOperation();

  const loadCookies = useCallback(async () => {
    try {
      setLoading(true);
      const data = await executeAsync(() => apiClient.getCookies());
      setCookies(data);
    } catch (err) {
      setMessage({ type: 'error', text: '加载 Cookie 列表失败' });
    } finally {
      setLoading(false);
    }
  }, [executeAsync]);

  useEffect(() => {
    loadCookies();
  }, [refreshKey, loadCookies]);

  const handleCheck = useCallback(async (cookie: string) => {
    setChecking(prev => [...prev, cookie]);
    try {
      const result = await executeAsync(() => apiClient.checkCookieStatus(cookie));
      await loadCookies(); // 重新加载列表
      const statusTexts: Record<string, string> = {
        alive: '正常',
        banned: '已封禁',
        invalid: '格式无效',
        error: '异常',
      };
      const statusText = statusTexts[result.status] ?? result.status;
      const detail = result.error ? `（${result.error}）` : '';
      setMessage({
        type: result.status === 'invalid' || result.status === 'error' ? 'error' : 'success',
        text: `检测状态：${statusText}${detail}`,
      });
    } catch (err) {
      setMessage({ type: 'error', text: '检测失败' });
    } finally {
      setChecking(prev => prev.filter(c => c !== cookie));
    }
  }, [executeAsync, loadCookies]);

  const handleDelete = useCallback(async (cookie: string) => {
    if (!window.confirm('确定要删除这个 Cookie 吗？')) {
      return;
    }

    setDeleting(prev => [...prev, cookie]);
    try {
      await executeAsync(() => apiClient.deleteCookie(cookie));
      await loadCookies();
      setMessage({ type: 'success', text: '删除成功' });
    } catch (err) {
      setMessage({ type: 'error', text: '删除失败' });
    } finally {
      setDeleting(prev => prev.filter(c => c !== cookie));
    }
  }, [executeAsync, loadCookies]);

  const formatCookie = (cookie: string) => {
    if (cookie.startsWith('sk-ant-sid01-')) {
      return `sk-ant-sid01-${cookie.slice(13, 20)}...${cookie.slice(-10)}`;
    }
    return `${cookie.slice(0, 10)}...${cookie.slice(-10)}`;
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return '从未';
    return formatDistanceToNow(new Date(dateString), {
      addSuffix: true,
      locale: zhCN,
    });
  };

  const getStatusLabel = (record: QueueCookie) => {
    if (record.is_banned) {
      return '已封禁';
    }
    if (activeTab === 'processing') {
      return '处理中';
    }
    return '待处理';
  };

  const columns = [
    {
      key: 'cookie',
      title: 'Cookie',
      render: (value: string) => (
        <span className="font-mono text-sm bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
          {formatCookie(value)}
        </span>
      ),
    },
    {
      key: 'status',
      title: '状态',
      render: (_: any, record: QueueCookie) => (
        <span className="text-xs px-2 py-1 rounded bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-200">
          {getStatusLabel(record)}
        </span>
      ),
    },
    {
      key: 'requests_sent',
      title: '请求次数',
      render: (value?: number) => (
        <span className="text-gray-600 dark:text-gray-400">{value ?? 0}</span>
      ),
    },
    {
      key: 'last_used_at',
      title: '最后使用',
      render: (value?: string) => (
        <span className="text-gray-600 dark:text-gray-400">{formatDate(value)}</span>
      ),
    },
    {
      key: 'submitted_at',
      title: '添加时间',
      render: (value?: string) => (
        <span className="text-gray-600 dark:text-gray-400">{formatDate(value)}</span>
      ),
    },
    {
      key: 'actions',
      title: '操作',
      render: (_: any, record: QueueCookie) => (
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleCheck(record.cookie)}
            loading={checking.includes(record.cookie)}
            disabled={checking.includes(record.cookie) || deleting.includes(record.cookie)}
            icon={<ArrowPathIcon className="h-4 w-4" />}
          >
            检测
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => handleDelete(record.cookie)}
            loading={deleting.includes(record.cookie)}
            disabled={checking.includes(record.cookie) || deleting.includes(record.cookie)}
            icon={<TrashIcon className="h-4 w-4" />}
          >
            删除
          </Button>
        </div>
      ),
    },
  ];

  const currentData = (activeTab === 'pending'
    ? cookies.pending
    : activeTab === 'processing'
      ? cookies.processing
      : cookies.banned).filter((c) =>
        debouncedSearch
          ? c.cookie.toLowerCase().includes(debouncedSearch.toLowerCase())
          : true
      );
  const totalRequests =
    cookies.pending.reduce((sum, c) => sum + (c.requests_sent ?? 0), 0) +
    cookies.processing.reduce((sum, c) => sum + (c.requests_sent ?? 0), 0) +
    cookies.banned.reduce((sum, c) => sum + (c.requests_sent ?? 0), 0);

  return (
    <Card>
      <CardHeader>
        <div className="flex flex-col gap-3 md:flex-row md:justify-between md:items-center">
          <CardTitle className="flex items-center gap-2">
            {activeTab === 'pending' && <ClockIcon className="h-5 w-5 text-warning-500" />}
            {activeTab === 'processing' && <ArrowTrendingUpIcon className="h-5 w-5 text-primary-500" />}
            {activeTab === 'banned' && <XCircleIcon className="h-5 w-5 text-error-500" />}
            {activeTab === 'pending' && '待处理'}
            {activeTab === 'processing' && '处理中'}
            {activeTab === 'banned' && '已封禁'} Cookie
            <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
              ({currentData.length} 个)
            </span>
          </CardTitle>
          <div className="flex gap-2">
            <Button
              variant={activeTab === 'pending' ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setActiveTab('pending')}
            >
              待处理 ({cookies.pending.length})
            </Button>
            <Button
              variant={activeTab === 'processing' ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setActiveTab('processing')}
            >
              处理中 ({cookies.processing.length})
            </Button>
            <Button
              variant={activeTab === 'banned' ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setActiveTab('banned')}
            >
              已封禁 ({cookies.banned.length})
            </Button>
          </div>
          <div className="w-full md:w-64">
            <Input
              placeholder="搜索 Cookie 片段"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>

        <div className="text-sm text-gray-500 dark:text-gray-400">
          总请求次数: {totalRequests}
        </div>
      </CardHeader>

      {message && (
        <div className="px-6 pb-4">
          <Message type={message.type}>
            {message.text}
          </Message>
        </div>
      )}

      {loading ? (
        <div className="px-6 pb-6">
          <CookieTableSkeleton />
        </div>
      ) : (
        <Table
          data={currentData}
          columns={columns}
          loading={loading}
        />
      )}
    </Card>
  );
});

CookieList.displayName = 'CookieList';

export default CookieList;
