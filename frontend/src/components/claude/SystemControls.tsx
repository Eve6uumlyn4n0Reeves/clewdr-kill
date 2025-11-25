import React, { useEffect, useState, useCallback } from "react";
import Card from "../common/Card";
import Button from "../common/Button";
import StatusMessage from "../common/StatusMessage";
import {
  pauseAllWorkers,
  resumeAllWorkers,
  emergencyStop,
  clearPendingQueue,
  clearBannedQueue,
  getSystemStatus,
} from "../../api";
import { statsApi } from "../../api/stats";

interface SystemControlsProps {
  onChange?: () => void;
}

type ActionKey =
  | "pause"
  | "resume"
  | "stop"
  | "resetStats"
  | "clearPending"
  | "clearBanned";

const SystemControls: React.FC<SystemControlsProps> = ({ onChange }) => {
  const [loading, setLoading] = useState<Partial<Record<ActionKey, boolean>>>({});
  const [status, setStatus] = useState<string>("-");
  const [workers, setWorkers] = useState<number>(0);
  const [message, setMessage] = useState<{ type: "success" | "error" | "info" | "warning" | null; text: string }>({
    type: null,
    text: "",
  });

  const loadStatus = useCallback(async () => {
    try {
      const data = await getSystemStatus();
      setStatus(data.status || "-");
      setWorkers(data.active_workers ?? 0);
    } catch {
      setStatus("-");
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  const run = async (key: ActionKey, fn: () => Promise<any>, success: string) => {
    setLoading((prev) => ({ ...prev, [key]: true }));
    setMessage({ type: null, text: "" });
    try {
      await fn();
      setMessage({ type: "success", text: success });
      onChange?.();
      await loadStatus();
    } catch (err) {
      const msg = err instanceof Error ? err.message : "操作失败";
      setMessage({ type: "error", text: msg });
    } finally {
      setLoading((prev) => ({ ...prev, [key]: false }));
    }
  };

  return (
    <Card>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h4 className="text-white font-semibold">⚙️ 系统控制</h4>
          <p className="text-gray-400 text-sm">
            状态: <span className="text-blue-300">{status}</span> | 活跃线程:{" "}
            <span className="text-green-300">{workers}</span>
          </p>
        </div>
        <Button size="sm" variant="secondary" onClick={loadStatus}>
          刷新状态
        </Button>
      </div>

      {message.type && <StatusMessage type={message.type} message={message.text} />}

      <div className="grid grid-cols-2 md:grid-cols-3 gap-3 mt-4">
        <Button
          size="sm"
          onClick={() => run("pause", () => pauseAllWorkers(), "已暂停所有 worker")}
          isLoading={loading.pause}
          variant="secondary"
        >
          ⏸️ 暂停
        </Button>
        <Button
          size="sm"
          onClick={() => run("resume", () => resumeAllWorkers(), "已恢复 worker")}
          isLoading={loading.resume}
        >
          ▶️ 恢复
        </Button>
        <Button
          size="sm"
          onClick={() => run("stop", () => emergencyStop(), "已紧急停止")}
          isLoading={loading.stop}
          variant="secondary"
          className="bg-red-600 hover:bg-red-700"
        >
          🛑 停止
        </Button>
        <Button
          size="sm"
          onClick={() => run("clearPending", () => clearPendingQueue(), "已清空待处理队列")}
          isLoading={loading.clearPending}
          variant="secondary"
        >
          🧹 清空待处理
        </Button>
        <Button
          size="sm"
          onClick={() => run("clearBanned", () => clearBannedQueue(), "已清空封禁队列")}
          isLoading={loading.clearBanned}
          variant="secondary"
        >
          ♻️ 清空封禁
        </Button>
        <Button
          size="sm"
          onClick={() => run("resetStats", () => statsApi.resetStats(), "统计已重置")}
          isLoading={loading.resetStats}
        >
          🧮 重置统计
        </Button>
      </div>
    </Card>
  );
};

export default SystemControls;
