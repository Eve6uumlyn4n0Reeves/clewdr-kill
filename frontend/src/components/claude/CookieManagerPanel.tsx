import React, { useState, useEffect, useCallback } from "react";
import { getCookieStatus, deleteCookie, checkCookieStatus, postCookie } from "../../api";
import { BanQueueInfo, BanCookie } from "../../types/cookie.types";
import { statsApi, CookieMetrics } from "../../api/stats";
import Button from "../common/Button";
import LoadingSpinner from "../common/LoadingSpinner";
import StatusMessage from "../common/StatusMessage";
import CookieSection from "./CookieSection";
import CookieValue from "./CookieValue";
import DeleteButton from "./DeleteButton";

type CookieCheckResult = {
  alive: boolean;
  banned: boolean;
  lastChecked: string;
  error?: string;
};

type SortField = 'submitted_at' | 'last_used_at' | 'requests_sent' | 'status';
type SortOrder = 'asc' | 'desc';

interface CookieManagerPanelProps {
  refreshKey?: number;
  onExport?: (cookies: BanCookie[]) => void;
  onImport?: (cookies: string[]) => void;
}

const CookieManagerPanel: React.FC<CookieManagerPanelProps> = ({
  refreshKey = 0,
  onExport,
  onImport,
}) => {
  const [queueInfo, setQueueInfo] = useState<BanQueueInfo | null>(null);
  const [cookieMetrics, setCookieMetrics] = useState<Record<string, CookieMetrics>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshCounter, setRefreshCounter] = useState(0);

  // Selection state
  const [selectedCookies, setSelectedCookies] = useState<Set<string>>(new Set());
  const [selectAll, setSelectAll] = useState(false);

  // Filter and sort state
  const [filter, setFilter] = useState({
    status: 'all', // 'all', 'pending', 'banned'
    search: '',
    minRequests: 0,
  });
  const [sortField, setSortField] = useState<SortField>('submitted_at');
  const [sortOrder, setSortOrder] = useState<SortOrder>('desc');

  // UI state
  // const [deletingCookie, setDeletingCookie] = useState<string | null>(null);
  const [checkingCookie, setCheckingCookie] = useState<string | null>(null);
  const [checkResults, setCheckResults] = useState<Record<string, CookieCheckResult>>({});
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);

  // Fetch data
  const fetchCookieStatus = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [queueData, metricsData] = await Promise.all([
        getCookieStatus(),
        statsApi.getCookieMetrics().catch(() => []), // Fallback to empty array
      ]);

      setQueueInfo(queueData);

      // Convert metrics array to record
      const metricsRecord: Record<string, CookieMetrics> = {};
      metricsData.forEach(metric => {
        metricsRecord[metric.cookie_id] = metric;
      });
      setCookieMetrics(metricsRecord);

      setLastUpdated(new Date().toLocaleString());
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      setQueueInfo(null);
      setCookieMetrics({});
    } finally {
      setLoading(false);
    }
  }, []);

  // Auto-refresh
  useEffect(() => {
    fetchCookieStatus();
    const interval = setInterval(fetchCookieStatus, 10000);
    return () => clearInterval(interval);
  }, [fetchCookieStatus, refreshCounter, refreshKey]);

  const handleRefresh = useCallback(() => {
    setRefreshCounter(prev => prev + 1);
  }, []);

  // Filter and sort cookies
  const filteredAndSortedCookies = useCallback((cookies: BanCookie[]) => {
    let filtered = cookies;

    // Apply filters
    if (filter.status !== 'all') {
      filtered = filter.status === 'pending'
        ? queueInfo?.pending || []
        : queueInfo?.banned || [];
    }

    if (filter.search) {
      filtered = filtered.filter(cookie =>
        cookie.cookie.toLowerCase().includes(filter.search.toLowerCase())
      );
    }

    if (filter.minRequests > 0) {
      filtered = filtered.filter(cookie => (cookie.requests_sent ?? 0) >= filter.minRequests);
    }

    // Apply sorting
    filtered.sort((a, b) => {
      let aValue: any, bValue: any;

      switch (sortField) {
        case 'submitted_at':
          aValue = a.submitted_at || '';
          bValue = b.submitted_at || '';
          break;
        case 'last_used_at':
          aValue = a.last_used_at || '';
          bValue = b.last_used_at || '';
          break;
        case 'requests_sent':
          aValue = a.requests_sent ?? 0;
          bValue = b.requests_sent ?? 0;
          break;
        case 'status':
          aValue = a.is_banned ? 'banned' : 'pending';
          bValue = b.is_banned ? 'banned' : 'pending';
          break;
        default:
          aValue = a.submitted_at || '';
          bValue = b.submitted_at || '';
      }

      if (aValue < bValue) return sortOrder === 'asc' ? -1 : 1;
      if (aValue > bValue) return sortOrder === 'asc' ? 1 : -1;
      return 0;
    });

    return filtered;
  }, [queueInfo, filter, sortField, sortOrder]);

  // Selection handlers
  const handleSelectCookie = useCallback((cookie: string) => {
    setSelectedCookies(prev => {
      const newSet = new Set(prev);
      if (newSet.has(cookie)) {
        newSet.delete(cookie);
      } else {
        newSet.add(cookie);
      }
      return newSet;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    if (selectAll) {
      setSelectedCookies(new Set());
    } else {
      const allCookies = [
        ...(queueInfo?.pending || []),
        ...(queueInfo?.banned || [])
      ];
      setSelectedCookies(new Set(allCookies.map(c => c.cookie)));
    }
    setSelectAll(!selectAll);
  }, [selectAll, queueInfo]);

  // Batch operations
  const handleBatchDelete = useCallback(async () => {
    if (selectedCookies.size === 0) return;

    if (!window.confirm(`确定要删除 ${selectedCookies.size} 个Cookie吗？`)) return;

    setLoading(true);
    let successCount = 0;
    let errorCount = 0;

    for (const cookie of selectedCookies) {
      try {
        await deleteCookie(cookie);
        successCount++;
      } catch (err) {
        errorCount++;
        console.error('Failed to delete cookie:', err);
      }
    }

    setSelectedCookies(new Set());
    setSelectAll(false);
    fetchCookieStatus();

    setError(errorCount === 0 ? null : `删除完成：成功 ${successCount}，失败 ${errorCount}`);
    setLoading(false);
  }, [selectedCookies, fetchCookieStatus]);

  const handleBatchCheck = useCallback(async () => {
    if (selectedCookies.size === 0) return;

    setLoading(true);
    for (const cookie of selectedCookies) {
      setCheckingCookie(cookie);
      try {
        const result = await checkCookieStatus(cookie);
        setCheckResults(prev => ({
          ...prev,
          [cookie]: result,
        }));
      } catch (err) {
        console.error('Failed to check cookie:', err);
      } finally {
        setCheckingCookie(null);
      }
    }
    setLoading(false);
  }, [selectedCookies]);

  // Export functionality
  const handleExport = useCallback(() => {
    const allCookies = [
      ...(queueInfo?.pending || []),
      ...(queueInfo?.banned || [])
    ];
    onExport?.(allCookies);
  }, [queueInfo, onExport]);

  // Import functionality
  const handleImport = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.txt,.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const text = await file.text();
      const pushCookies = async (list: string[]) => {
        let success = 0;
        let failed = 0;
        for (const c of list) {
          try {
            await postCookie(c.trim());
            success++;
          } catch (err) {
            failed++;
            console.error("导入失败", err);
          }
        }
        setError(failed > 0 ? `导入完成：成功 ${success}，失败 ${failed}` : null);
        handleRefresh();
      };

      if (file.name.endsWith('.json')) {
        try {
          const data = JSON.parse(text);
          const cookies = Array.isArray(data) ? data : [data];
          const lines = cookies.map(String).filter(Boolean);
          await pushCookies(lines);
        } catch (err) {
          alert('无效的JSON文件格式');
        }
      } else {
        const cookies = text.split('\n').map(l => l.trim()).filter(Boolean);
        await pushCookies(cookies);
      }
    };
    input.click();
  }, [onImport, handleRefresh]);

  const formatDate = (dateStr: string | null | undefined) => {
    if (!dateStr) return "-";
    try {
      return new Date(dateStr).toLocaleString();
    } catch {
      return dateStr;
    }
  };

  const pendingCount = queueInfo?.pending.length || 0;
  const bannedCount = queueInfo?.banned.length || 0;
  const totalCookies = pendingCount + bannedCount;

  return (
    <div className="space-y-6">
      {/* Header with controls */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div>
          <h3 className="text-xl font-semibold text-white">🍪 Cookie管理中心</h3>
          <p className="text-sm text-gray-400 mt-1">
            总计: {totalCookies} | 待处理: {pendingCount} | 已封禁: {bannedCount}
          </p>
          {lastUpdated && (
            <p className="text-xs text-gray-500 mt-1">
              最后更新: {lastUpdated}
            </p>
          )}
        </div>

        <div className="flex flex-wrap gap-2">
          <Button onClick={handleExport} variant="secondary" size="sm">
            📤 导出
          </Button>
          <Button onClick={handleImport} variant="secondary" size="sm">
            📥 导入
          </Button>
          <Button
            onClick={() => setShowAdvanced(!showAdvanced)}
            variant="secondary"
            size="sm"
          >
            {showAdvanced ? '🔼 简化' : '🔽 高级'}
          </Button>
          <Button onClick={fetchCookieStatus} disabled={loading} size="sm">
            🔄 刷新
          </Button>
        </div>
      </div>

      {/* Error Display */}
      {error && <StatusMessage type="error" message={error} />}

      {/* Advanced controls */}
      {showAdvanced && (
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <h4 className="text-white font-medium mb-3">🎛️ 高级控制</h4>

          {/* Filters */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">状态筛选</label>
              <select
                value={filter.status}
                onChange={(e) => setFilter(prev => ({ ...prev, status: e.target.value }))}
                className="w-full bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-white"
              >
                <option value="all">全部</option>
                <option value="pending">待处理</option>
                <option value="banned">已封禁</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">搜索</label>
              <input
                type="text"
                value={filter.search}
                onChange={(e) => setFilter(prev => ({ ...prev, search: e.target.value }))}
                placeholder="搜索Cookie..."
                className="w-full bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-white"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">最小请求数</label>
              <input
                type="number"
                value={filter.minRequests}
                onChange={(e) => setFilter(prev => ({ ...prev, minRequests: parseInt(e.target.value) || 0 }))}
                min="0"
                className="w-full bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-white"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">排序</label>
              <select
                value={`${sortField}-${sortOrder}`}
                onChange={(e) => {
                  const [field, order] = e.target.value.split('-');
                  setSortField(field as SortField);
                  setSortOrder(order as SortOrder);
                }}
                className="w-full bg-gray-700 border border-gray-600 rounded-md px-3 py-2 text-white"
              >
                <option value="submitted_at-desc">提交时间 (新到旧)</option>
                <option value="submitted_at-asc">提交时间 (旧到新)</option>
                <option value="requests_sent-desc">请求数量 (多到少)</option>
                <option value="requests_sent-asc">请求数量 (少到多)</option>
                <option value="last_used_at-desc">最后使用 (新到旧)</option>
              </select>
            </div>
          </div>

          {/* Batch operations */}
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={selectAll}
                onChange={handleSelectAll}
                className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600"
              />
              <span className="text-sm text-gray-300">全选 ({selectedCookies.size})</span>
            </div>

            {selectedCookies.size > 0 && (
              <>
                <Button onClick={handleBatchCheck} disabled={loading} size="sm">
                  🩺 批量测活
                </Button>
                <Button onClick={handleBatchDelete} disabled={loading} variant="secondary" size="sm">
                  🗑️ 批量删除
                </Button>
              </>
            )}
          </div>
        </div>
      )}

      {/* Loading State */}
      {loading && totalCookies === 0 && (
        <div className="flex justify-center py-8">
          <LoadingSpinner size="lg" color="text-cyan-500" />
        </div>
      )}

      {/* Pending Cookies */}
      <CookieSection
        title={`🟡 待处理 Cookie (${pendingCount})`}
        cookies={filteredAndSortedCookies(queueInfo?.pending || [])}
        color="yellow"
        renderStatus={(cookie: BanCookie, index: number) => {
          const cookieStr = cookie.cookie;
          const isSelected = selectedCookies.has(cookieStr);
          const metrics = cookieMetrics[cookieStr];
          const checkResult = checkResults[cookieStr];

          return (
            <div
              key={index}
              className={`py-3 text-sm border-b border-gray-700 last:border-b-0 ${
                isSelected ? 'bg-blue-900/20' : ''
              }`}
            >
              <div className="flex items-start gap-3">
                {/* Checkbox for selection */}
                <input
                  type="checkbox"
                  checked={isSelected}
                  onChange={() => handleSelectCookie(cookieStr)}
                  className="mt-1 w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600"
                />

                {/* Cookie info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <div className="flex-1 min-w-0">
                      <CookieValue cookie={cookie.cookie} />
                    </div>
                    <div className="flex items-center gap-2 whitespace-nowrap">
                      <span className="text-orange-400 font-medium text-xs">
                        ⚡ 处理中
                      </span>
                      <button
                        onClick={() => {
                          setCheckingCookie(cookieStr);
                          checkCookieStatus(cookieStr)
                            .then(result => {
                              setCheckResults(prev => ({
                                ...prev,
                                [cookieStr]: result,
                              }));
                            })
                            .catch(() => {})
                            .finally(() => setCheckingCookie(null));
                        }}
                        disabled={checkingCookie === cookieStr}
                        className="px-2 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-50"
                      >
                        {checkingCookie === cookieStr ? '检测中...' : '测活'}
                      </button>
                      <DeleteButton
                        cookie={cookieStr}
                        onDelete={deleteCookie}
                        isDeleting={false}
                      />
                    </div>
                  </div>

                  {/* Metrics and details */}
                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-2 text-xs text-gray-400">
                    <div>请求数: {cookie.requests_sent ?? 0}</div>
                    <div>提交: {formatDate(cookie.submitted_at)}</div>
                    {cookie.last_used_at && (
                      <div>最后使用: {formatDate(cookie.last_used_at)}</div>
                    )}
                    {metrics && (
                      <div>平均响应: {metrics.average_response_time}ms</div>
                    )}
                  </div>

                  {/* Check result */}
                  {checkResult && (
                    <div className="mt-2 text-xs">
                      <span className={`font-medium ${
                        checkResult.banned ? 'text-red-400' :
                        checkResult.alive ? 'text-green-400' : 'text-yellow-400'
                      }`}>
                        {checkResult.banned ? '✗ 已封禁' :
                         checkResult.alive ? '✓ 存活' : '? 状态未知'}
                      </span>
                      <span className="text-gray-500 ml-2">
                        检测于: {formatDate(checkResult.lastChecked)}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          );
        }}
      />

      {/* Banned Cookies */}
      <CookieSection
        title={`🔴 已封禁 Cookie (${bannedCount})`}
        cookies={filteredAndSortedCookies(queueInfo?.banned || [])}
        color="red"
        renderStatus={(cookie: BanCookie, index: number) => {
          const cookieStr = cookie.cookie;
          const isSelected = selectedCookies.has(cookieStr);
          const metrics = cookieMetrics[cookieStr];
          const checkResult = checkResults[cookieStr];

          return (
            <div
              key={index}
              className={`py-3 text-sm border-b border-gray-700 last:border-b-0 ${
                isSelected ? 'bg-red-900/20' : ''
              }`}
            >
              <div className="flex items-start gap-3">
                {/* Checkbox for selection */}
                <input
                  type="checkbox"
                  checked={isSelected}
                  onChange={() => handleSelectCookie(cookieStr)}
                  className="mt-1 w-4 h-4 rounded border-gray-600 bg-gray-700 text-red-600"
                />

                {/* Cookie info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <div className="flex-1 min-w-0">
                      <CookieValue cookie={cookie.cookie} />
                    </div>
                    <div className="flex items-center gap-2 whitespace-nowrap">
                      <span className="text-red-400 font-medium text-xs">
                        ✅ 已封禁
                      </span>
                      <button
                        onClick={() => {
                          setCheckingCookie(cookieStr);
                          checkCookieStatus(cookieStr)
                            .then(result => {
                              setCheckResults(prev => ({
                                ...prev,
                                [cookieStr]: result,
                              }));
                            })
                            .catch(() => {})
                            .finally(() => setCheckingCookie(null));
                        }}
                        disabled={checkingCookie === cookieStr}
                        className="px-2 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-50"
                      >
                        {checkingCookie === cookieStr ? '检测中...' : '复检'}
                      </button>
                      <DeleteButton
                        cookie={cookieStr}
                        onDelete={deleteCookie}
                        isDeleting={false}
                      />
                    </div>
                  </div>

                  {/* Metrics and details */}
                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-2 text-xs text-gray-400">
                    <div>请求数: {cookie.requests_sent ?? 0}</div>
                    <div>提交: {formatDate(cookie.submitted_at)}</div>
                    {cookie.last_used_at && (
                      <div>最后使用: {formatDate(cookie.last_used_at)}</div>
                    )}
                    {metrics && (
                      <div>平均响应: {metrics.average_response_time}ms</div>
                    )}
                  </div>

                  {/* Check result */}
                  {checkResult && (
                    <div className="mt-2 text-xs">
                      <span className={`font-medium ${
                        checkResult.banned ? 'text-red-400' :
                        checkResult.alive ? 'text-green-400' : 'text-yellow-400'
                      }`}>
                        {checkResult.banned ? '✗ 仍封禁' :
                         checkResult.alive ? '✓ 已恢复' : '? 状态变化'}
                      </span>
                      <span className="text-gray-500 ml-2">
                        检测于: {formatDate(checkResult.lastChecked)}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          );
        }}
      />
    </div>
  );
};

export default CookieManagerPanel;
