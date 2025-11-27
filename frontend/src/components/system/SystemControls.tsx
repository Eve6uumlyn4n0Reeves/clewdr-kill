import React, { useState, useEffect } from "react";
import {
  PlayIcon,
  PauseIcon,
  StopIcon,
  ArrowPathIcon,
  ExclamationTriangleIcon,
  ChartBarIcon,
  CpuChipIcon,
  ServerIcon,
} from "@heroicons/react/24/outline";
import { Button, IconButton, ButtonGroup } from "../ui/Button";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  StatusCard,
} from "../ui/Card";
import { apiClient } from "../../api";
import toast from "react-hot-toast";

interface SystemStatus {
  status: string;
  uptime_seconds: number;
  active_workers: number;
  queue_size: number;
  banned_count: number;
  total_requests: number;
  maintenance_mode: boolean;
}

const SystemControls: React.FC = () => {
  const [loading, setLoading] = useState<string | null>(null);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // 获取系统状态
  const fetchSystemStatus = async () => {
    try {
      const status = await apiClient.getSystemStatus();
      setSystemStatus(status);
      setLastUpdate(new Date());
    } catch (error) {
      console.error("Failed to fetch system status:", error);
    }
  };

  // 定期更新系统状态
  useEffect(() => {
    fetchSystemStatus();
    const interval = setInterval(fetchSystemStatus, 5000); // 每5秒更新
    return () => clearInterval(interval);
  }, []);

  const handleAction = async (
    action: Parameters<typeof apiClient.adminAction>[0],
    label: string,
    confirmMessage?: string,
  ) => {
    // 危险操作需要确认
    if (confirmMessage && !window.confirm(confirmMessage)) {
      return;
    }

    setLoading(action);

    try {
      const result = await apiClient.adminAction(action);
      toast.success(`${label}成功: ${result.message || "操作完成"}`);

      // 操作成功后立即更新状态
      setTimeout(fetchSystemStatus, 1000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "未知错误";
      toast.error(`${label}失败: ${errorMessage}`);
    } finally {
      setLoading(null);
    }
  };

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  };

  const getSystemStatusConfig = () => {
    if (!systemStatus)
      return { status: "offline", title: "离线", description: "无法获取状态" };

    if (systemStatus.maintenance_mode) {
      return {
        status: "warning" as const,
        title: "维护模式",
        description: "系统处于维护状态",
      };
    }

    if (systemStatus.status === "running" && systemStatus.active_workers > 0) {
      return {
        status: "online" as const,
        title: "运行中",
        description: `${systemStatus.active_workers} 个工作线程活跃`,
      };
    }

    if (systemStatus.status === "paused") {
      return {
        status: "warning" as const,
        title: "已暂停",
        description: "所有工作线程已暂停",
      };
    }

    return {
      status: "error" as const,
      title: "异常",
      description: "系统状态异常",
    };
  };

  const statusConfig = getSystemStatusConfig();

  return (
    <div className="space-y-6">
      {/* 系统状态概览 */}
      <Card variant="glass" glow="primary">
        <CardHeader>
          <CardTitle className="text-gradient">
            <CpuChipIcon className="h-6 w-6" />
            系统控制中心
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            {/* 系统状态 */}
            <StatusCard
              status={statusConfig.status}
              title={statusConfig.title}
              description={statusConfig.description}
            />

            {/* 运行时间 */}
            <Card variant="compact" className="text-center">
              <CardContent>
                <div className="data-value text-info">
                  {systemStatus
                    ? formatUptime(systemStatus.uptime_seconds)
                    : "--"}
                </div>
                <div className="data-label">运行时间</div>
              </CardContent>
            </Card>

            {/* 队列大小 */}
            <Card variant="compact" className="text-center">
              <CardContent>
                <div className="data-value text-warning">
                  {systemStatus?.queue_size ?? "--"}
                </div>
                <div className="data-label">队列大小</div>
              </CardContent>
            </Card>

            {/* 总请求数 */}
            <Card variant="compact" className="text-center">
              <CardContent>
                <div className="data-value text-primary">
                  {systemStatus?.total_requests?.toLocaleString() ?? "--"}
                </div>
                <div className="data-label">总请求数</div>
              </CardContent>
            </Card>
          </div>

          {/* 最后更新时间 */}
          <div className="text-xs text-muted text-center">
            最后更新: {lastUpdate.toLocaleTimeString()}
          </div>
        </CardContent>
      </Card>

      {/* 控制面板 */}
      <Card variant="glass-strong">
        <CardHeader>
          <CardTitle>
            <ServerIcon className="h-5 w-5" />
            操作面板
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* 基础控制 */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-muted uppercase tracking-wide">
                基础控制
              </h4>
              <ButtonGroup orientation="vertical" className="w-full">
                <Button
                  variant="success"
                  fullWidth
                  loading={loading === "resume_all"}
                  onClick={() => handleAction("resume_all", "恢复所有工作")}
                  icon={<PlayIcon className="h-4 w-4" />}
                  disabled={systemStatus?.status === "running"}
                >
                  恢复所有
                </Button>
                <Button
                  variant="warning"
                  fullWidth
                  loading={loading === "pause_all"}
                  onClick={() => handleAction("pause_all", "暂停所有工作")}
                  icon={<PauseIcon className="h-4 w-4" />}
                  disabled={systemStatus?.status === "paused"}
                >
                  暂停所有
                </Button>
              </ButtonGroup>
            </div>

            {/* 高级控制 */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-muted uppercase tracking-wide">
                高级控制
              </h4>
              <ButtonGroup orientation="vertical" className="w-full">
                <Button
                  variant="info"
                  fullWidth
                  loading={loading === "reset_stats"}
                  onClick={() =>
                    handleAction(
                      "reset_stats",
                      "重置统计数据",
                      "确定要重置所有统计数据吗？此操作不可撤销。",
                    )
                  }
                  icon={<ChartBarIcon className="h-4 w-4" />}
                >
                  重置统计
                </Button>
                <Button
                  variant="secondary"
                  fullWidth
                  loading={loading === "clear_all"}
                  onClick={() =>
                    handleAction(
                      "clear_all",
                      "清空队列",
                      "确定要清空所有队列吗？这将删除所有待处理和已封禁的Cookie。",
                    )
                  }
                  icon={<ArrowPathIcon className="h-4 w-4" />}
                >
                  清空队列
                </Button>
              </ButtonGroup>
            </div>
          </div>

          {/* 紧急控制 */}
          <div className="mt-6 pt-4 border-t border-border/50">
            <div className="flex items-center justify-between">
              <div>
                <h4 className="text-sm font-medium text-danger flex items-center gap-2">
                  <ExclamationTriangleIcon className="h-4 w-4" />
                  紧急控制
                </h4>
                <p className="text-xs text-muted mt-1">
                  立即停止所有操作，进入维护模式
                </p>
              </div>
              <Button
                variant="danger"
                loading={loading === "emergency_stop"}
                onClick={() =>
                  handleAction(
                    "emergency_stop",
                    "紧急停止",
                    "确定要执行紧急停止吗？这将立即停止所有工作线程并进入维护模式。",
                  )
                }
                icon={<StopIcon className="h-4 w-4" />}
                glow
              >
                紧急停止
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 快速操作栏 */}
      <Card variant="compact">
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="text-sm text-muted">快速操作</div>
            <div className="flex items-center gap-2">
              <IconButton
                variant="ghost"
                size="sm"
                icon={<ArrowPathIcon className="h-4 w-4" />}
                onClick={fetchSystemStatus}
                tooltip="刷新状态"
              />
              <IconButton
                variant="ghost"
                size="sm"
                icon={<ChartBarIcon className="h-4 w-4" />}
                onClick={() => window.open("/stats", "_blank")}
                tooltip="查看统计"
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default SystemControls;
