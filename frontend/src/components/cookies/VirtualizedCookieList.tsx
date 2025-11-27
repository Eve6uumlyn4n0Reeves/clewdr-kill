import React, { useState, useEffect, useCallback, memo, useMemo } from "react";
import { FixedSizeList as List } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";
import {
  TrashIcon,
  ArrowPathIcon,
  XCircleIcon,
  ClockIcon,
  ArrowTrendingUpIcon,
  MagnifyingGlassIcon,
  FunnelIcon,
  ArrowsUpDownIcon,
} from "@heroicons/react/24/outline";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { Button, IconButton } from "../ui/Button";
import { Card, CardHeader, CardTitle, CardContent } from "../ui/Card";
import { apiClient } from "../../api";
import type { QueueCookie, QueueStatusResponse } from "../../types/api.types";
import { useAsyncOperation } from "../../hooks/useAsyncOperation";
import { useDebounce } from "../../hooks";
import { cn } from "../../utils/cn";
import toast from "react-hot-toast";
import CookieTableSkeleton from "./Skeleton";

interface VirtualizedCookieListProps {
  refreshKey?: number;
}

interface CookieRowData {
  items: QueueCookie[];
  checking: string[];
  deleting: string[];
  onCheck: (cookie: string) => void;
  onDelete: (cookie: string) => void;
  activeTab: "pending" | "processing" | "banned";
}

// 单行组件 - 使用 memo 优化重渲染
const CookieRow = memo<{
  index: number;
  style: React.CSSProperties;
  data: CookieRowData;
}>(({ index, style, data }) => {
  const { items, checking, deleting, onCheck, onDelete, activeTab } = data;
  const record = items[index];

  if (!record) return null;

  const formatCookie = (cookie: string) => {
    if (cookie.startsWith("sk-ant-sid01-")) {
      return `sk-ant-sid01-${cookie.slice(13, 20)}...${cookie.slice(-10)}`;
    }
    return `${cookie.slice(0, 10)}...${cookie.slice(-10)}`;
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return "从未";
    return formatDistanceToNow(new Date(dateString), {
      addSuffix: true,
      locale: zhCN,
    });
  };

  const getStatusConfig = () => {
    if (record.is_banned) {
      return {
        label: "已封禁",
        className: "badge-banned",
        icon: <XCircleIcon className="h-3 w-3" />,
      };
    }
    if (activeTab === "processing") {
      return {
        label: "处理中",
        className: "badge-checking",
        icon: <ArrowTrendingUpIcon className="h-3 w-3" />,
      };
    }
    return {
      label: "待处理",
      className: "badge-pending",
      icon: <ClockIcon className="h-3 w-3" />,
    };
  };

  const statusConfig = getStatusConfig();
  const isOperating =
    checking.includes(record.cookie) || deleting.includes(record.cookie);

  return (
    <div
      style={style}
      className={cn(
        "flex items-center px-6 py-3 border-b border-border/50 transition-all duration-200",
        "hover:bg-surfaceHighlight/50 group",
        index % 2 === 0 ? "bg-surface/30" : "bg-transparent",
        isOperating && "opacity-60",
      )}
    >
      {/* Cookie 值 */}
      <div className="flex-1 min-w-0 pr-4">
        <div className="font-mono text-sm bg-surfaceHighlight px-3 py-1.5 rounded-lg border border-border">
          {formatCookie(record.cookie)}
        </div>
        {record.error_message && (
          <div className="text-xs text-danger mt-1 truncate">
            错误: {record.error_message}
          </div>
        )}
      </div>

      {/* 状态 */}
      <div className="w-24 flex-shrink-0 px-2">
        <div className={cn("badge", statusConfig.className)}>
          {statusConfig.icon}
          <span className="ml-1">{statusConfig.label}</span>
        </div>
      </div>

      {/* 请求次数 */}
      <div className="w-20 flex-shrink-0 px-2 text-center">
        <div className="data-value text-sm">{record.requests_sent ?? 0}</div>
        <div className="text-xs text-muted">次</div>
      </div>

      {/* 最后使用时间 */}
      <div className="w-28 flex-shrink-0 px-2 text-center">
        <div className="text-xs text-muted">
          {formatDate(record.last_used_at || undefined)}
        </div>
      </div>

      {/* 添加时间 */}
      <div className="w-28 flex-shrink-0 px-2 text-center">
        <div className="text-xs text-muted">
          {formatDate(record.submitted_at || undefined)}
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="w-32 flex-shrink-0 flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <IconButton
          variant="ghost"
          size="sm"
          icon={<ArrowPathIcon className="h-4 w-4" />}
          onClick={() => onCheck(record.cookie)}
          loading={checking.includes(record.cookie)}
          disabled={isOperating}
          tooltip="检测状态"
        />
        <IconButton
          variant="ghost"
          size="sm"
          icon={<TrashIcon className="h-4 w-4" />}
          onClick={() => onDelete(record.cookie)}
          loading={deleting.includes(record.cookie)}
          disabled={isOperating}
          tooltip="删除"
          className="text-danger hover:text-danger-400"
        />
      </div>
    </div>
  );
});

CookieRow.displayName = "CookieRow";

// 表头组件
const TableHeader: React.FC<{
  sortBy: string;
  sortOrder: "asc" | "desc";
  onSort: (field: string) => void;
}> = ({ sortBy, onSort }) => {
  const SortIcon = ({ field }: { field: string }) => (
    <ArrowsUpDownIcon
      className={cn(
        "h-3 w-3 ml-1 transition-colors",
        sortBy === field ? "text-primary" : "text-muted",
      )}
    />
  );

  return (
    <div className="flex items-center px-6 py-3 bg-surfaceHighlight border-b border-border text-xs font-medium text-muted uppercase tracking-wider">
      <div className="flex-1 pr-4">Cookie</div>
      <div className="w-24 px-2">状态</div>
      <div
        className="w-20 px-2 text-center cursor-pointer hover:text-foreground transition-colors flex items-center justify-center"
        onClick={() => onSort("requests_sent")}
      >
        请求次数
        <SortIcon field="requests_sent" />
      </div>
      <div
        className="w-28 px-2 text-center cursor-pointer hover:text-foreground transition-colors flex items-center justify-center"
        onClick={() => onSort("last_used_at")}
      >
        最后使用
        <SortIcon field="last_used_at" />
      </div>
      <div
        className="w-28 px-2 text-center cursor-pointer hover:text-foreground transition-colors flex items-center justify-center"
        onClick={() => onSort("submitted_at")}
      >
        添加时间
        <SortIcon field="submitted_at" />
      </div>
      <div className="w-32 text-center">操作</div>
    </div>
  );
};

// 主组件
const VirtualizedCookieList = memo<VirtualizedCookieListProps>(
  ({ refreshKey }) => {
    const [cookies, setCookies] = useState<QueueStatusResponse>({
      pending: [],
      processing: [],
      banned: [],
      total_requests: 0,
    });
    const [loading, setLoading] = useState(true);
    const [checking, setChecking] = useState<string[]>([]);
    const [deleting, setDeleting] = useState<string[]>([]);
    const [activeTab, setActiveTab] = useState<
      "pending" | "processing" | "banned"
    >("pending");
    const [searchTerm, setSearchTerm] = useState("");
    const debouncedSearch = useDebounce(searchTerm, 300);
    const [sortBy, setSortBy] = useState<string>("submitted_at");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

    const { execute: executeAsync } = useAsyncOperation();

    const loadCookies = useCallback(async () => {
      try {
        setLoading(true);
        const data = await executeAsync(() => apiClient.getCookies());
        setCookies(data as QueueStatusResponse);
      } catch (err) {
        toast.error("加载 Cookie 列表失败");
      } finally {
        setLoading(false);
      }
    }, [executeAsync]);

    useEffect(() => {
      loadCookies();
    }, [refreshKey, loadCookies]);

    const handleCheck = useCallback(
      async (cookie: string) => {
        setChecking((prev) => [...prev, cookie]);
        try {
          const result = await executeAsync(() =>
            apiClient.checkCookieStatus(cookie),
          );
          await loadCookies();

          const statusTexts: Record<string, string> = {
            alive: "正常",
            banned: "已封禁",
            invalid: "格式无效",
            error: "异常",
          };
          const result_typed = result as any;
          const statusText =
            statusTexts[result_typed.status] ?? result_typed.status;
          const detail = result_typed.error ? `（${result_typed.error}）` : "";

          if (
            result_typed.status === "invalid" ||
            result_typed.status === "error"
          ) {
            toast.error(`检测状态：${statusText}${detail}`);
          } else {
            toast.success(`检测状态：${statusText}${detail}`);
          }
        } catch (err) {
          toast.error("检测失败");
        } finally {
          setChecking((prev) => prev.filter((c) => c !== cookie));
        }
      },
      [executeAsync, loadCookies],
    );

    const handleDelete = useCallback(
      async (cookie: string) => {
        if (!window.confirm("确定要删除这个 Cookie 吗？")) {
          return;
        }

        setDeleting((prev) => [...prev, cookie]);
        try {
          await executeAsync(() => apiClient.deleteCookie(cookie));
          await loadCookies();
          toast.success("删除成功");
        } catch (err) {
          toast.error("删除失败");
        } finally {
          setDeleting((prev) => prev.filter((c) => c !== cookie));
        }
      },
      [executeAsync, loadCookies],
    );

    const handleSort = useCallback(
      (field: string) => {
        if (sortBy === field) {
          setSortOrder((prev) => (prev === "asc" ? "desc" : "asc"));
        } else {
          setSortBy(field);
          setSortOrder("desc");
        }
      },
      [sortBy],
    );

    // 过滤和排序数据
    const filteredAndSortedData = useMemo(() => {
      let data =
        activeTab === "pending"
          ? cookies.pending
          : activeTab === "processing"
            ? cookies.processing
            : cookies.banned;

      // 搜索过滤
      if (debouncedSearch) {
        data = data.filter(
          (item) =>
            item.cookie.toLowerCase().includes(debouncedSearch.toLowerCase()) ||
            (item.error_message &&
              item.error_message
                .toLowerCase()
                .includes(debouncedSearch.toLowerCase())),
        );
      }

      // 排序
      data = [...data].sort((a, b) => {
        let aValue: any = a[sortBy as keyof QueueCookie];
        let bValue: any = b[sortBy as keyof QueueCookie];

        // 处理日期字段
        if (sortBy === "submitted_at" || sortBy === "last_used_at") {
          aValue = aValue ? new Date(aValue).getTime() : 0;
          bValue = bValue ? new Date(bValue).getTime() : 0;
        }

        // 处理数字字段
        if (sortBy === "requests_sent") {
          aValue = aValue ?? 0;
          bValue = bValue ?? 0;
        }

        if (aValue < bValue) return sortOrder === "asc" ? -1 : 1;
        if (aValue > bValue) return sortOrder === "asc" ? 1 : -1;
        return 0;
      });

      return data;
    }, [cookies, activeTab, debouncedSearch, sortBy, sortOrder]);

    // 虚拟列表数据
    const rowData: CookieRowData = useMemo(
      () => ({
        items: filteredAndSortedData,
        checking,
        deleting,
        onCheck: handleCheck,
        onDelete: handleDelete,
        activeTab,
      }),
      [
        filteredAndSortedData,
        checking,
        deleting,
        handleCheck,
        handleDelete,
        activeTab,
      ],
    );

    const totalRequests = useMemo(
      () =>
        cookies.pending.reduce((sum, c) => sum + (c.requests_sent ?? 0), 0) +
        cookies.processing.reduce((sum, c) => sum + (c.requests_sent ?? 0), 0) +
        cookies.banned.reduce((sum, c) => sum + (c.requests_sent ?? 0), 0),
      [cookies],
    );

    const tabConfig = [
      {
        key: "pending" as const,
        label: "待处理",
        count: cookies.pending.length,
        icon: <ClockIcon className="h-4 w-4" />,
        color: "warning",
      },
      {
        key: "processing" as const,
        label: "处理中",
        count: cookies.processing.length,
        icon: <ArrowTrendingUpIcon className="h-4 w-4" />,
        color: "info",
      },
      {
        key: "banned" as const,
        label: "已封禁",
        count: cookies.banned.length,
        icon: <XCircleIcon className="h-4 w-4" />,
        color: "danger",
      },
    ];

    return (
      <Card variant="glass" className="h-full flex flex-col">
        <CardHeader>
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
            <div className="flex items-center gap-4">
              <CardTitle className="text-gradient">Cookie 管理中心</CardTitle>
              <div className="text-sm text-muted">
                总请求: {totalRequests.toLocaleString()}
              </div>
            </div>

            {/* 搜索和过滤 */}
            <div className="flex items-center gap-3">
              <div className="relative">
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted" />
                <input
                  type="text"
                  placeholder="搜索 Cookie..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="input pl-10 w-64"
                />
              </div>
              <IconButton
                variant="ghost"
                size="sm"
                icon={<FunnelIcon className="h-4 w-4" />}
                tooltip="高级过滤"
              />
            </div>
          </div>

          {/* 标签页 */}
          <div className="flex items-center gap-2 mt-4">
            {tabConfig.map((tab) => (
              <Button
                key={tab.key}
                variant={activeTab === tab.key ? "primary" : "ghost"}
                size="sm"
                onClick={() => setActiveTab(tab.key)}
                icon={tab.icon}
                className="flex-shrink-0"
              >
                {tab.label} ({tab.count})
              </Button>
            ))}
          </div>

          {/* 搜索结果提示 */}
          {searchTerm && (
            <div className="text-sm text-muted">
              找到 {filteredAndSortedData.length} 个匹配的结果
            </div>
          )}
        </CardHeader>

        <CardContent className="flex-1 p-0 min-h-0">
          {loading ? (
            <div className="p-6">
              <CookieTableSkeleton />
            </div>
          ) : filteredAndSortedData.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-64 text-muted">
              <XCircleIcon className="h-12 w-12 mb-3 opacity-50" />
              <div className="text-lg font-medium">暂无数据</div>
              <div className="text-sm">
                {searchTerm ? "没有找到匹配的 Cookie" : "当前分类下没有 Cookie"}
              </div>
            </div>
          ) : (
            <div className="h-full flex flex-col">
              {/* 表头 */}
              <TableHeader
                sortBy={sortBy}
                sortOrder={sortOrder}
                onSort={handleSort}
              />

              {/* 虚拟滚动列表 */}
              <div className="flex-1 min-h-0">
                <AutoSizer>
                  {({ height, width }) => (
                    <List
                      height={height}
                      width={width}
                      itemCount={filteredAndSortedData.length}
                      itemSize={60} // 每行高度
                      itemData={rowData}
                      overscanCount={5} // 预渲染行数
                    >
                      {CookieRow}
                    </List>
                  )}
                </AutoSizer>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    );
  },
);

VirtualizedCookieList.displayName = "VirtualizedCookieList";

export default VirtualizedCookieList;
