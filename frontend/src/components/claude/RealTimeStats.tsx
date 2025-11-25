import React, { useEffect, useState } from "react";
import { statsApi, SystemStats } from "../../api/stats";
import { Line } from "react-chartjs-2";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Tooltip,
  Legend,
} from "chart.js";

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend);

interface RealTimeStatsProps {
  refreshInterval?: number;
}

const formatUptime = (seconds: number): string => {
  if (!seconds && seconds !== 0) return "00:00:00";
  const hrs = Math.floor(seconds / 3600)
    .toString()
    .padStart(2, "0");
  const mins = Math.floor((seconds % 3600) / 60)
    .toString()
    .padStart(2, "0");
  const secs = Math.floor(seconds % 60)
    .toString()
    .padStart(2, "0");
  return `${hrs}:${mins}:${secs}`;
};

const RealTimeStats: React.FC<RealTimeStatsProps> = ({
  refreshInterval = 5000,
}) => {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<{
    labels: string[];
    requests: number[];
    successRates: number[];
    responseTimes: number[];
    errorRates: number[];
  }>({
    labels: [],
    requests: [],
    successRates: [],
    responseTimes: [],
    errorRates: [],
  });

  useEffect(() => {
    const fetchStats = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const data = await statsApi.getSystemStats();
        setStats(data);
        const hist = await statsApi.getHistoricalStats({ interval_minutes: 1 });
        setHistory({
          labels: hist.timestamps.map((t) => new Date(t).toLocaleTimeString()),
          requests: hist.request_counts,
          successRates: hist.success_rates,
          responseTimes: hist.response_times,
          errorRates: hist.error_rates,
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch stats");
      } finally {
        setIsLoading(false);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, refreshInterval);
    return () => clearInterval(interval);
  }, [refreshInterval]);

  const StatCard: React.FC<{
    title: string;
    value: string | number;
    icon?: string;
    color?: string;
  }> = ({ title, value, icon, color = "blue" }) => {
    const colorClass =
      color === "yellow"
        ? "text-yellow-400"
        : color === "orange"
        ? "text-orange-400"
        : color === "green"
        ? "text-green-400"
        : "text-blue-400";
    return (
      <div
        className={`bg-gray-800 rounded-lg p-4 border border-gray-700 ${isLoading ? "opacity-50" : ""}`}
      >
        <div className="flex items-center justify-between">
          <div>
            <p className="text-gray-400 text-sm">{title}</p>
            <p className={`text-2xl font-bold ${colorClass}`}>
              {typeof value === "number" ? value.toLocaleString() : value}
            </p>
          </div>
          {icon && <div className="text-2xl">{icon}</div>}
        </div>
      </div>
    );
  };

  const ProgressRing: React.FC<{
    percentage: number;
    size?: number;
    strokeWidth?: number;
  }> = ({ percentage, size = 60, strokeWidth = 4 }) => {
    const radius = (size - strokeWidth) / 2;
    const circumference = radius * 2 * Math.PI;
    const offset = circumference - (percentage / 100) * circumference;
    const colorClass =
      percentage >= 80 ? "text-green-500" : percentage >= 50 ? "text-yellow-500" : "text-red-500";

    return (
      <div className="relative inline-flex items-center justify-center">
        <svg className="transform -rotate-90" width={size} height={size}>
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            stroke="currentColor"
            strokeWidth={strokeWidth}
            fill="none"
            className="text-gray-700"
          />
          <circle
            cx={size / 2}
            cy={size / 2}
            r={radius}
            stroke="currentColor"
            strokeWidth={strokeWidth}
            fill="none"
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            className={`${colorClass} transition-all duration-500`}
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-xs font-medium text-white">{Math.round(percentage)}%</span>
        </div>
      </div>
    );
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-semibold text-white">📊 实时统计监控</h3>
        <div className="flex items-center gap-2 text-sm text-gray-400">
          <span>
            最后更新: {stats?.last_update ? new Date(stats.last_update).toLocaleString() : "-"}
          </span>
          {(isLoading || !stats) && (
            <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
          )}
        </div>
      </div>

      {error && (
        <div className="text-sm text-red-400 bg-red-900/30 border border-red-800 rounded p-3">
          {error}
        </div>
      )}

      {!stats && isLoading && <div className="text-gray-400 text-sm">加载中...</div>}

      {stats && (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <StatCard title="总Cookie数" value={stats.total_cookies} icon="🍪" color="yellow" />
            <StatCard title="待封禁" value={stats.pending_cookies} icon="⏳" color="orange" />
            <StatCard title="已封禁" value={stats.banned_cookies} icon="✅" color="green" />
            <StatCard title="总请求数" value={stats.total_requests} icon="🚀" color="blue" />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h4 className="text-white font-medium mb-4">📈 请求性能</h4>
              <div className="space-y-4">
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-400">每分钟请求数</span>
                    <span className="text-white">{stats.requests_per_minute.toFixed(1)}</span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div
                      className="bg-blue-500 h-2 rounded-full transition-all duration-500"
                      style={{ width: `${Math.min(100, (stats.requests_per_minute / 60) * 100)}%` }}
                    />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-400">成功率</span>
                    <span className="text-white">{stats.success_rate.toFixed(1)}%</span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div
                      className="bg-green-500 h-2 rounded-full transition-all duration-500"
                      style={{ width: `${stats.success_rate}%` }}
                    />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-400">平均响应时间</span>
                    <span className="text-white">{stats.average_response_time}ms</span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div
                      className="bg-yellow-500 h-2 rounded-full transition-all duration-500"
                      style={{ width: `${Math.min(100, (stats.average_response_time / 5000) * 100)}%` }}
                    />
                  </div>
                </div>
              </div>
            </div>

            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h4 className="text-white font-medium mb-4">⚙️ 工作状态</h4>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">活跃工作线程</span>
                  <div className="flex items-center gap-2">
                    <span className="text-white">{stats.workers_active}</span>
                    <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                  </div>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">运行时长</span>
                  <span className="text-white font-mono">{formatUptime(stats.uptime_seconds)}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-gray-400">封号进度</span>
                  <ProgressRing
                    percentage={
                      stats.total_cookies > 0
                        ? (stats.banned_cookies / stats.total_cookies) * 100
                        : 0
                    }
                  />
                </div>
              </div>
            </div>

            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-3">
              <h4 className="text-white font-medium">📊 概览</h4>
              <p className="text-gray-400 text-sm">
                成功率：{stats.success_rate.toFixed(1)}% | 平均响应：{stats.average_response_time}ms | 请求速率：
                {stats.requests_per_minute.toFixed(1)}/min
              </p>
              <div className="grid grid-cols-2 gap-2 text-sm text-gray-300">
                <div>CPU: {(stats.performance_metrics?.cpu_usage ?? 0).toFixed(1)}%</div>
                <div>内存: {(((stats.performance_metrics?.memory_usage ?? 0) / 1024 / 1024)).toFixed(1)} MB</div>
                <div>队列处理: {stats.performance_metrics?.queue_processing_time ?? 0} ms</div>
                <div>网络: {stats.performance_metrics?.network_latency ?? 0} ms</div>
                <div className="col-span-2">
                  策略有效性: {(stats.performance_metrics?.strategy_effectiveness ?? 0).toFixed(1)}%
                </div>
              </div>
            </div>
          </div>

          <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <h4 className="text-white font-medium mb-4">📉 历史趋势</h4>
            {history.labels.length > 1 ? (
              <Line
                data={{
                  labels: history.labels,
                  datasets: [
                    {
                      label: "请求数",
                      data: history.requests,
                      borderColor: "rgba(59,130,246,0.8)",
                      backgroundColor: "rgba(59,130,246,0.2)",
                      yAxisID: "y",
                    },
                    {
                      label: "成功率 %",
                      data: history.successRates,
                      borderColor: "rgba(34,197,94,0.8)",
                      backgroundColor: "rgba(34,197,94,0.2)",
                      yAxisID: "y1",
                    },
                    {
                      label: "响应时间 ms",
                      data: history.responseTimes,
                      borderColor: "rgba(234,179,8,0.8)",
                      backgroundColor: "rgba(234,179,8,0.2)",
                      yAxisID: "y2",
                    },
                    {
                      label: "错误率 %",
                      data: history.errorRates,
                      borderColor: "rgba(248,113,113,0.8)",
                      backgroundColor: "rgba(248,113,113,0.2)",
                      yAxisID: "y1",
                      borderDash: [6, 6],
                    },
                  ],
                }}
                options={{
                  responsive: true,
                  scales: {
                    y: { beginAtZero: true, position: "left" },
                    y1: { beginAtZero: true, position: "right", grid: { drawOnChartArea: false } },
                    y2: { beginAtZero: true, position: "right", grid: { drawOnChartArea: false } },
                  },
                  plugins: {
                    legend: { labels: { color: "#fff" } },
                  },
                }}
              />
            ) : (
              <p className="text-gray-400 text-sm">暂无历史数据</p>
            )}
          </div>

          <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <h4 className="text-white font-medium mb-3">⚠️ 错误分布</h4>
            {Object.keys(stats.error_distribution || {}).length === 0 ? (
              <p className="text-gray-400 text-sm">暂无错误</p>
            ) : (
              <ul className="space-y-2 text-sm text-gray-300">
                {Object.entries(stats.error_distribution)
                  .sort((a, b) => b[1] - a[1])
                  .slice(0, 5)
                  .map(([k, v]) => (
                    <li key={k} className="flex justify-between">
                      <span>{k}</span>
                      <span className="text-red-300">{v}</span>
                    </li>
                  ))}
              </ul>
            )}
          </div>
        </>
      )}
    </div>
  );
};

export default RealTimeStats;
