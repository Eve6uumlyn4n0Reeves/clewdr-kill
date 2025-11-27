import React, { useState, useEffect } from "react";
import {
  ChartPieIcon,
  ArrowRightIcon,
  CommandLineIcon,
  RocketLaunchIcon,
  ShieldCheckIcon,
  BoltIcon,
  EyeIcon,
  PlusIcon,
  Cog6ToothIcon,
  ChartBarIcon,
  CpuChipIcon,
} from "@heroicons/react/24/outline";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  StatusCard,
} from "./ui/Card";
import { Button, IconButton, ButtonGroup } from "./ui/Button";
import { RealTimeStats } from "./stats";
import { SystemControls } from "./system";
import { useNavigate } from "react-router-dom";
import { useSystemStats } from "../api/stats";
import { usePerfMonitor } from "../hooks";

const Dashboard: React.FC = () => {
  const navigate = useNavigate();
  const { stats } = useSystemStats({ refreshIntervalMs: 10000 });
  const [currentTime, setCurrentTime] = useState(new Date());
  usePerfMonitor("dashboard");

  // 更新当前时间
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const quickActions = [
    {
      title: "添加Cookie",
      description: "批量导入Cookie进行封号处理",
      icon: <PlusIcon className="h-5 w-5" />,
      path: "/cookies",
      variant: "primary" as const,
      glow: "primary" as const,
    },
    {
      title: "系统配置",
      description: "调整封号策略和系统参数",
      icon: <Cog6ToothIcon className="h-5 w-5" />,
      path: "/config",
      variant: "secondary" as const,
      glow: "info" as const,
    },
    {
      title: "数据统计",
      description: "查看详细的封号统计报告",
      icon: <ChartBarIcon className="h-5 w-5" />,
      path: "/stats",
      variant: "secondary" as const,
      glow: "success" as const,
    },
  ];

  const systemOverview = {
    status: (stats?.workers_active ?? 0) > 0 ? "运行中" : "待机",
    efficiency: stats?.success_rate ?? 0,
    totalProcessed: stats?.total_requests ?? 0,
    queueSize: stats?.pending_cookies ?? 0,
  };

  return (
    <div className="min-h-screen space-y-8 animate-fade-in">
      {/* 指挥中心标题 */}
      <div className="relative">
        <div className="absolute inset-0 bg-gradient-to-r from-primary/10 via-transparent to-success/10 rounded-2xl blur-xl" />
        <Card variant="glass-strong" className="relative">
          <CardContent className="py-8">
            <div className="flex flex-col lg:flex-row items-center justify-between gap-6">
              <div className="text-center lg:text-left">
                <h1 className="text-4xl lg:text-5xl font-bold text-gradient mb-2">
                  ClewdR 指挥中心
                </h1>
                <p className="text-lg text-muted">
                  高效的Claude Cookie封号控制台 - Kill Edition
                </p>
                <div className="flex items-center gap-4 mt-4 justify-center lg:justify-start">
                  <div className="flex items-center gap-2 text-sm text-muted">
                    <div className="pulse-dot-success" />
                    <span>系统在线</span>
                  </div>
                  <div className="text-sm text-muted font-mono">
                    {currentTime.toLocaleString()}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <div className="text-center">
                  <div className="data-value text-primary text-2xl">
                    {systemOverview.efficiency.toFixed(1)}%
                  </div>
                  <div className="data-label">效率</div>
                </div>
                <div className="w-px h-12 bg-border" />
                <div className="text-center">
                  <div className="data-value text-success text-2xl">
                    {systemOverview.totalProcessed.toLocaleString()}
                  </div>
                  <div className="data-label">已处理</div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 主要内容区域 */}
      <div className="grid grid-cols-1 xl:grid-cols-4 gap-8">
        {/* 左侧主要区域 */}
        <div className="xl:col-span-3 space-y-8">
          {/* 实时统计 */}
          <Card variant="glass" glow="primary">
            <CardHeader>
              <CardTitle className="text-gradient">
                <ChartPieIcon className="h-6 w-6" />
                实时数据监控
              </CardTitle>
            </CardHeader>
            <CardContent>
              <RealTimeStats />
            </CardContent>
          </Card>

          {/* 系统控制 */}
          <SystemControls />
        </div>

        {/* 右侧快速操作区域 */}
        <div className="space-y-6">
          {/* 系统状态卡片 */}
          <Card variant="glass-strong" glow="success">
            <CardHeader>
              <CardTitle>
                <CpuChipIcon className="h-5 w-5" />
                系统状态
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <StatusCard
                status={(stats?.workers_active ?? 0) > 0 ? "online" : "offline"}
                title={systemOverview.status}
                description={
                  (stats?.workers_active ?? 0) > 0
                    ? `${stats?.workers_active ?? 0} 个工作线程活跃`
                    : "系统处于待机状态"
                }
              />

              <div className="grid grid-cols-2 gap-3">
                <div className="text-center p-3 bg-surfaceHighlight/50 rounded-lg">
                  <div className="data-value text-warning text-lg">
                    {systemOverview.queueSize}
                  </div>
                  <div className="data-label text-xs">队列</div>
                </div>
                <div className="text-center p-3 bg-surfaceHighlight/50 rounded-lg">
                  <div className="data-value text-info text-lg">
                    {stats?.workers_active ?? 0}
                  </div>
                  <div className="data-label text-xs">工作线程</div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* 快速操作 */}
          <Card variant="glass" glow="info">
            <CardHeader>
              <CardTitle>
                <RocketLaunchIcon className="h-5 w-5" />
                快速操作
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {quickActions.map((action, index) => (
                <Button
                  key={index}
                  variant="secondary"
                  fullWidth
                  className="justify-start h-auto p-4 text-left"
                  onClick={() => navigate(action.path)}
                  glow={index === 0} // 只有第一个按钮有辉光效果
                >
                  <div className="flex items-center gap-3 w-full">
                    <div className="p-2 rounded-lg bg-primary/10 text-primary flex-shrink-0">
                      {action.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-foreground">
                        {action.title}
                      </div>
                      <div className="text-xs text-muted mt-1 truncate">
                        {action.description}
                      </div>
                    </div>
                    <ArrowRightIcon className="h-4 w-4 text-muted flex-shrink-0" />
                  </div>
                </Button>
              ))}
            </CardContent>
          </Card>

          {/* 性能指标 */}
          <Card variant="compact">
            <CardContent>
              <div className="space-y-3">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted">CPU 使用率</span>
                  <span className="font-mono text-success">12%</span>
                </div>
                <div className="progress">
                  <div
                    className="progress-bar bg-gradient-to-r from-success to-success-600"
                    style={{ width: "12%" }}
                  />
                </div>

                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted">内存使用</span>
                  <span className="font-mono text-info">256MB</span>
                </div>
                <div className="progress">
                  <div
                    className="progress-bar bg-gradient-to-r from-info to-info-600"
                    style={{ width: "35%" }}
                  />
                </div>

                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted">网络延迟</span>
                  <span className="font-mono text-warning">45ms</span>
                </div>
                <div className="progress">
                  <div
                    className="progress-bar bg-gradient-to-r from-warning to-warning-600"
                    style={{ width: "25%" }}
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          {/* 快捷工具 */}
          <Card variant="compact">
            <CardContent>
              <div className="flex items-center justify-between mb-3">
                <span className="text-sm font-medium text-muted">快捷工具</span>
              </div>
              <ButtonGroup className="w-full">
                <IconButton
                  variant="ghost"
                  size="sm"
                  icon={<EyeIcon className="h-4 w-4" />}
                  tooltip="监控面板"
                  onClick={() => navigate("/stats")}
                />
                <IconButton
                  variant="ghost"
                  size="sm"
                  icon={<CommandLineIcon className="h-4 w-4" />}
                  tooltip="系统日志"
                  onClick={() => console.log("Open logs")}
                />
                <IconButton
                  variant="ghost"
                  size="sm"
                  icon={<ShieldCheckIcon className="h-4 w-4" />}
                  tooltip="安全设置"
                  onClick={() => navigate("/config")}
                />
                <IconButton
                  variant="ghost"
                  size="sm"
                  icon={<BoltIcon className="h-4 w-4" />}
                  tooltip="性能优化"
                  onClick={() => console.log("Performance tuning")}
                />
              </ButtonGroup>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* 底部状态栏 */}
      <Card variant="compact">
        <CardContent>
          <div className="flex items-center justify-between text-xs text-muted">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <div className="pulse-dot-success" />
                <span>系统正常运行</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="pulse-dot-info" />
                <span>数据同步中</span>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <span>版本: v0.11.27</span>
              <span>构建: Kill Edition</span>
              <span>
                运行时间:{" "}
                {stats?.uptime_seconds
                  ? Math.floor(stats.uptime_seconds / 3600)
                  : 0}
                h
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Dashboard;
