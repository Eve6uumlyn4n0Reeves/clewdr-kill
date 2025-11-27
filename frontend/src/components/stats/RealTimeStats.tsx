import React, { useState, useEffect } from "react";
import {
  ChartBarIcon,
  ClockIcon,
  CpuChipIcon,
  SignalIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  MinusIcon,
} from "@heroicons/react/24/outline";
import { useSystemStats } from "../../api/stats";
import { StatCard, Card, CardHeader, CardTitle, CardContent } from "../ui/Card";
import { IconButton } from "../ui/Button";
import { formatTimeElapsed } from "../../utils/formatters";
import toast from "react-hot-toast";

interface AnimatedNumberProps {
  value: number;
  duration?: number;
  formatter?: (value: number) => string;
}

const AnimatedNumber: React.FC<AnimatedNumberProps> = ({
  value,
  duration = 1000,
  formatter = (v) => v.toString(),
}) => {
  const [displayValue, setDisplayValue] = useState(value);
  const [isAnimating, setIsAnimating] = useState(false);

  useEffect(() => {
    if (displayValue !== value) {
      setIsAnimating(true);
      const startValue = displayValue;
      const difference = value - startValue;
      const startTime = Date.now();

      const animate = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);

        // 使用 easeOutCubic 缓动函数
        const easeProgress = 1 - Math.pow(1 - progress, 3);
        const currentValue = Math.round(startValue + difference * easeProgress);

        setDisplayValue(currentValue);

        if (progress < 1) {
          requestAnimationFrame(animate);
        } else {
          setIsAnimating(false);
        }
      };

      requestAnimationFrame(animate);
    }
  }, [value, duration, displayValue]);

  return (
    <span className={isAnimating ? "animate-flash" : ""}>
      {formatter(displayValue)}
    </span>
  );
};

interface TrendIndicatorProps {
  current: number;
  previous: number;
  formatter?: (value: number) => string;
}

const TrendIndicator: React.FC<TrendIndicatorProps> = ({
  current,
  previous,
  formatter = (v) => v.toString(),
}) => {
  const difference = current - previous;
  const percentChange = previous > 0 ? (difference / previous) * 100 : 0;

  if (Math.abs(percentChange) < 0.1) {
    return (
      <div className="flex items-center gap-1 text-xs text-muted">
        <MinusIcon className="h-3 w-3" />
        <span>无变化</span>
      </div>
    );
  }

  const isPositive = difference > 0;
  const Icon = isPositive ? ArrowTrendingUpIcon : ArrowTrendingDownIcon;
  const colorClass = isPositive ? "text-success" : "text-danger";

  return (
    <div className={`flex items-center gap-1 text-xs ${colorClass}`}>
      <Icon className="h-3 w-3" />
      <span>
        {isPositive ? "+" : ""}
        {formatter(difference)} ({percentChange.toFixed(1)}%)
      </span>
    </div>
  );
};

const RealTimeStats: React.FC = () => {
  const { stats, loading, error, refetch } = useSystemStats({
    refreshIntervalMs: 5000,
  });
  const [previousStats, setPreviousStats] = useState(stats);
  const [lastUpdateTime, setLastUpdateTime] = useState<Date>(new Date());

  // 保存上一次的统计数据用于趋势计算
  useEffect(() => {
    if (stats && stats !== previousStats) {
      setPreviousStats(previousStats || stats);
      setLastUpdateTime(new Date());
    }
  }, [stats, previousStats]);

  const handleRefresh = async () => {
    try {
      await refetch();
      toast.success("数据已刷新");
    } catch (err) {
      toast.error("刷新失败");
    }
  };

  if (error) {
    return (
      <Card variant="glass" glow="danger">
        <CardContent>
          <div className="text-center py-8">
            <div className="text-danger text-lg font-medium mb-2">
              数据加载失败
            </div>
            <div className="text-muted text-sm mb-4">{error}</div>
            <IconButton
              variant="danger"
              icon={<ArrowTrendingUpIcon className="h-4 w-4" />}
              onClick={handleRefresh}
            >
              重试
            </IconButton>
          </div>
        </CardContent>
      </Card>
    );
  }

  const placeholders = {
    total_requests: stats?.total_requests ?? 0,
    success_rate: stats?.success_rate ?? 0,
    pending_cookies: stats?.pending_cookies ?? 0,
    banned_cookies: stats?.banned_cookies ?? 0,
    workers_active: stats?.workers_active ?? 0,
    uptime: stats ? formatTimeElapsed(stats.uptime_seconds) : "--",
    average_response_time: stats?.average_response_time ?? 0,
    requests_per_minute: stats?.requests_per_minute ?? 0,
  };

  const previousPlaceholders = {
    total_requests: previousStats?.total_requests ?? 0,
    success_rate: previousStats?.success_rate ?? 0,
    pending_cookies: previousStats?.pending_cookies ?? 0,
    banned_cookies: previousStats?.banned_cookies ?? 0,
    requests_per_minute: previousStats?.requests_per_minute ?? 0,
  };

  return (
    <div className="space-y-6">
      {/* 标题栏 */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-gradient flex items-center gap-2">
            <SignalIcon className="h-6 w-6" />
            实时数据监控
          </h2>
          <p className="text-sm text-muted mt-1">
            最后更新: {lastUpdateTime.toLocaleTimeString()}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-2 text-xs text-muted">
            <div className="pulse-dot-success" />
            <span>实时更新</span>
          </div>
          <IconButton
            variant="ghost"
            size="sm"
            icon={<ArrowTrendingUpIcon className="h-4 w-4" />}
            onClick={handleRefresh}
            loading={loading}
            tooltip="手动刷新"
          />
        </div>
      </div>

      {/* 核心指标 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="总请求数"
          value={
            <AnimatedNumber
              value={placeholders.total_requests}
              formatter={(v) => v.toLocaleString()}
            />
          }
          change={
            <TrendIndicator
              current={placeholders.total_requests}
              previous={previousPlaceholders.total_requests}
              formatter={(v) => v.toLocaleString()}
            />
          }
          trend={
            placeholders.total_requests > previousPlaceholders.total_requests
              ? "up"
              : "neutral"
          }
          icon={<ChartBarIcon className="h-5 w-5" />}
          glow="primary"
        />

        <StatCard
          title="成功率"
          value={
            <AnimatedNumber
              value={placeholders.success_rate}
              formatter={(v) => `${v.toFixed(1)}%`}
            />
          }
          change={
            <TrendIndicator
              current={placeholders.success_rate}
              previous={previousPlaceholders.success_rate}
              formatter={(v) => `${v.toFixed(1)}%`}
            />
          }
          trend={
            placeholders.success_rate > 80
              ? "up"
              : placeholders.success_rate < 60
                ? "down"
                : "neutral"
          }
          icon={<ArrowTrendingUpIcon className="h-5 w-5" />}
          glow={placeholders.success_rate > 80 ? "success" : "warning"}
        />

        <StatCard
          title="待处理"
          value={
            <AnimatedNumber
              value={placeholders.pending_cookies}
              formatter={(v) => v.toLocaleString()}
            />
          }
          change={
            <TrendIndicator
              current={placeholders.pending_cookies}
              previous={previousPlaceholders.pending_cookies}
              formatter={(v) => v.toLocaleString()}
            />
          }
          trend={
            placeholders.pending_cookies > previousPlaceholders.pending_cookies
              ? "up"
              : placeholders.pending_cookies <
                  previousPlaceholders.pending_cookies
                ? "down"
                : "neutral"
          }
          icon={<ClockIcon className="h-5 w-5" />}
          glow="warning"
        />

        <StatCard
          title="已封禁"
          value={
            <AnimatedNumber
              value={placeholders.banned_cookies}
              formatter={(v) => v.toLocaleString()}
            />
          }
          change={
            <TrendIndicator
              current={placeholders.banned_cookies}
              previous={previousPlaceholders.banned_cookies}
              formatter={(v) => v.toLocaleString()}
            />
          }
          trend={
            placeholders.banned_cookies > previousPlaceholders.banned_cookies
              ? "up"
              : "neutral"
          }
          icon={<MinusIcon className="h-5 w-5" />}
          glow="danger"
        />
      </div>

      {/* 详细指标 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <Card variant="glass" className="text-center">
          <CardContent>
            <div className="flex items-center justify-center mb-2">
              <CpuChipIcon className="h-5 w-5 text-info mr-2" />
              <span className="text-sm text-muted">活跃工作线程</span>
            </div>
            <div className="data-value text-info">
              {loading ? (
                <div className="loading-spinner w-6 h-6 mx-auto" />
              ) : (
                <AnimatedNumber value={placeholders.workers_active} />
              )}
            </div>
          </CardContent>
        </Card>

        <Card variant="glass" className="text-center">
          <CardContent>
            <div className="flex items-center justify-center mb-2">
              <ClockIcon className="h-5 w-5 text-primary mr-2" />
              <span className="text-sm text-muted">系统运行时间</span>
            </div>
            <div className="data-value text-primary font-mono">
              {loading ? "--:--:--" : placeholders.uptime}
            </div>
          </CardContent>
        </Card>

        <Card variant="glass" className="text-center">
          <CardContent>
            <div className="flex items-center justify-center mb-2">
              <SignalIcon className="h-5 w-5 text-success mr-2" />
              <span className="text-sm text-muted">平均响应时间</span>
            </div>
            <div className="data-value text-success">
              {loading ? (
                <div className="loading-spinner w-6 h-6 mx-auto" />
              ) : (
                <AnimatedNumber
                  value={placeholders.average_response_time}
                  formatter={(v) => `${v}ms`}
                />
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 性能指标 */}
      <Card variant="glass-strong">
        <CardHeader>
          <CardTitle>
            <ChartBarIcon className="h-5 w-5" />
            性能指标
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-muted">每分钟请求数</span>
                <span className="text-sm font-mono text-primary">
                  {loading ? (
                    <div className="loading-spinner w-4 h-4" />
                  ) : (
                    <AnimatedNumber
                      value={placeholders.requests_per_minute}
                      formatter={(v) => `${v}/min`}
                    />
                  )}
                </span>
              </div>
              <div className="progress">
                <div
                  className="progress-bar bg-gradient-to-r from-primary to-primary-600"
                  style={{
                    width: `${Math.min((placeholders.requests_per_minute / 100) * 100, 100)}%`,
                  }}
                />
              </div>
            </div>

            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-muted">成功率</span>
                <span className="text-sm font-mono text-success">
                  {loading ? (
                    <div className="loading-spinner w-4 h-4" />
                  ) : (
                    <AnimatedNumber
                      value={placeholders.success_rate}
                      formatter={(v) => `${v.toFixed(1)}%`}
                    />
                  )}
                </span>
              </div>
              <div className="progress">
                <div
                  className="progress-bar bg-gradient-to-r from-success to-success-600"
                  style={{ width: `${placeholders.success_rate}%` }}
                />
              </div>
            </div>
          </div>

          {/* 状态指示器 */}
          <div className="mt-6 pt-4 border-t border-border/50">
            <div className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <div className="pulse-dot-success" />
                  <span className="text-muted">系统正常</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="pulse-dot-info" />
                  <span className="text-muted">数据同步</span>
                </div>
              </div>
              <div className="text-muted">刷新间隔: 5秒</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default RealTimeStats;
