import React, { useState, useEffect } from "react";
import Button from "../common/Button";
import FormInput from "../common/FormInput";
import StatusMessage from "../common/StatusMessage";
import {
  fetchConfig,
  saveConfig as saveConfigApi,
  resetConfig as resetConfigApi,
  clearPendingQueue,
  clearBannedQueue,
} from "../../api";

interface BanConfig {
  concurrency: number;
  pause_seconds: number;
  prompts_dir: string;
  models: string[];
  max_tokens: number;
  request_timeout: number;
  retry_attempts: number;
  adaptive_throttling: boolean;
  smart_error_handling: boolean;
  proxy_rotation: boolean;
  user_agent_rotation: boolean;
  request_jitter_min: number;
  request_jitter_max: number;
  working_hours: {
    enabled: boolean;
    start: string;
    end: string;
    timezone: string;
  };
}

interface AdvancedConfigProps {
  onConfigChange?: (config: BanConfig) => void;
}

const AdvancedConfig: React.FC<AdvancedConfigProps> = ({ onConfigChange }) => {
  const [config, setConfig] = useState<BanConfig | null>(null);

  const [isLoading, setIsLoading] = useState(false);
  const [saveStatus, setSaveStatus] = useState<{
    type: "success" | "error" | null;
    message: string;
  }>({ type: null, message: "" });

  // 加载配置
  useEffect(() => {
    const loadConfig = async () => {
      try {
        const data = await fetchConfig();
        if (data?.ban_config) {
          setConfig(data.ban_config);
          onConfigChange?.(data.ban_config);
        }
      } catch (error) {
        console.error("Failed to load config:", error);
        setSaveStatus({ type: "error", message: "加载配置失败" });
      }
    };
    loadConfig();
  }, [onConfigChange]);

  // 更新配置
  const updateConfig = (updates: Partial<BanConfig>) => {
    if (!config) return;
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    onConfigChange?.(newConfig);
  };

  // 保存配置
  const saveConfig = async () => {
    setIsLoading(true);
    setSaveStatus({ type: null, message: "" });

    try {
      if (!config) throw new Error("配置未加载");
      await saveConfigApi({ ban_config: config });

      setSaveStatus({
        type: "success",
        message: "配置保存成功！",
      });

      setTimeout(() => setSaveStatus({ type: null, message: "" }), 3000);
    } catch (error) {
      setSaveStatus({
        type: "error",
        message: error instanceof Error ? error.message : "配置保存失败，请重试",
      });
    } finally {
      setIsLoading(false);
    }
  };

  // 添加模型
  const addModel = () => {
    if (!config) return;
    const newModel = prompt("请输入新模型名称：");
    if (newModel && !config.models.includes(newModel)) {
      updateConfig({
        models: [...config.models, newModel],
      });
    }
  };

  // 删除模型
  const removeModel = (index: number) => {
    if (!config) return;
    updateConfig({
      models: config.models.filter((_, i) => i !== index),
    });
  };

  const handleResetConfig = async () => {
    setIsLoading(true);
    try {
      const data = await resetConfigApi();
      if (data && typeof data === 'object' && 'ban_config' in data) {
        setConfig((data as any).ban_config);
        setSaveStatus({ type: "success", message: "已恢复默认配置" });
      }
    } catch (err) {
      setSaveStatus({ type: "error", message: "恢复默认配置失败" });
    } finally {
      setIsLoading(false);
    }
  };

  const handleClearQueues = async () => {
    setIsLoading(true);
    try {
      await clearPendingQueue();
      await clearBannedQueue();
      setSaveStatus({ type: "success", message: "队列已清空" });
    } catch (err) {
      setSaveStatus({ type: "error", message: "清空队列失败" });
    } finally {
      setIsLoading(false);
    }
  };

  if (!config) {
    return <StatusMessage type="info" message="正在加载配置..." />;
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-semibold text-white">⚙️ 高级配置管理</h3>
        <Button
          onClick={saveConfig}
          isLoading={isLoading}
          className="px-6"
        >
          保存配置
        </Button>
      </div>

      {saveStatus.type && (
        <StatusMessage type={saveStatus.type} message={saveStatus.message} />
      )}

      {/* 基础封号配置 */}
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h4 className="text-white font-medium mb-4">🎯 基础封号配置</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <FormInput
            id="concurrency"
            name="concurrency"
            type="number"
            value={config.concurrency.toString()}
            onChange={(e) => updateConfig({ concurrency: parseInt(e.target.value) || 1 })}
            label="并发工作线程数"
            min="1"
            max="20"
          />
          <FormInput
            id="pause_seconds"
            name="pause_seconds"
            type="number"
            value={config.pause_seconds.toString()}
            onChange={(e) => updateConfig({ pause_seconds: parseInt(e.target.value) || 60 })}
            label="全局暂停时间（秒）"
            min="60"
            max="3600"
          />
          <FormInput
            id="max_tokens"
            name="max_tokens"
            type="number"
            value={config.max_tokens.toString()}
            onChange={(e) => updateConfig({ max_tokens: parseInt(e.target.value) || 100 })}
            label="最大Token数"
            min="100"
            max="4096"
          />
          <FormInput
            id="request_timeout"
            name="request_timeout"
            type="number"
            value={config.request_timeout.toString()}
            onChange={(e) => updateConfig({ request_timeout: parseInt(e.target.value) || 5000 })}
            label="请求超时时间（毫秒）"
            min="5000"
            max="120000"
          />
        </div>
      </div>

      {/* 模型配置 */}
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h4 className="text-white font-medium mb-4">🤖 模型配置</h4>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-gray-300">已配置模型</span>
            <Button onClick={addModel} variant="secondary" size="sm">
              添加模型
            </Button>
          </div>
          <div className="space-y-2">
            {config.models.map((model, index) => (
              <div key={index} className="flex items-center justify-between bg-gray-900 rounded p-3">
                <span className="text-white font-mono text-sm">{model}</span>
                <Button
                  onClick={() => removeModel(index)}
                  variant="secondary"
                  size="sm"
                  className="text-red-400 hover:text-red-300"
                >
                  删除
                </Button>
              </div>
            ))}
            {config.models.length === 0 && (
              <p className="text-gray-500 text-center py-4">暂无配置的模型</p>
            )}
          </div>
        </div>
      </div>

      {/* 智能策略配置 */}
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h4 className="text-white font-medium mb-4">🧠 智能策略配置</h4>
        <div className="space-y-4">
          <label className="flex items-center justify-between">
            <span className="text-gray-300">自适应节流</span>
            <input
              type="checkbox"
              checked={config.adaptive_throttling}
              onChange={(e) => updateConfig({ adaptive_throttling: e.target.checked })}
              className="w-5 h-5 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
            />
          </label>
          <p className="text-xs text-gray-500">根据响应时间自动调整请求频率</p>

          <label className="flex items-center justify-between">
            <span className="text-gray-300">智能错误处理</span>
            <input
              type="checkbox"
              checked={config.smart_error_handling}
              onChange={(e) => updateConfig({ smart_error_handling: e.target.checked })}
              className="w-5 h-5 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
            />
          </label>
          <p className="text-xs text-gray-500">基于错误类型智能选择重试策略</p>

          <label className="flex items-center justify-between">
            <span className="text-gray-300">代理轮换</span>
            <input
              type="checkbox"
              checked={config.proxy_rotation}
              onChange={(e) => updateConfig({ proxy_rotation: e.target.checked })}
              className="w-5 h-5 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
            />
          </label>
          <p className="text-xs text-gray-500">在多个代理之间轮换请求</p>

          <label className="flex items-center justify-between">
            <span className="text-gray-300">User-Agent轮换</span>
            <input
              type="checkbox"
              checked={config.user_agent_rotation}
              onChange={(e) => updateConfig({ user_agent_rotation: e.target.checked })}
              className="w-5 h-5 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
            />
          </label>
          <p className="text-xs text-gray-500">随机化User-Agent头部提高隐蔽性</p>
        </div>
      </div>

      {/* 请求间隔配置 */}
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h4 className="text-white font-medium mb-4">⏱️ 请求间隔配置</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <FormInput
            id="jitter_min"
            name="jitter_min"
            type="number"
            value={config.request_jitter_min.toString()}
            onChange={(e) => updateConfig({ request_jitter_min: parseInt(e.target.value) || 0 })}
            label="最小间隔（毫秒）"
            min="0"
            max="10000"
          />
          <FormInput
            id="jitter_max"
            name="jitter_max"
            type="number"
            value={config.request_jitter_max.toString()}
            onChange={(e) => updateConfig({ request_jitter_max: parseInt(e.target.value) || 100 })}
            label="最大间隔（毫秒）"
            min="100"
            max="30000"
          />
        </div>
        <p className="text-xs text-gray-500 mt-2">
          请求间隔将在最小值和最大值之间随机选择，提高请求的自然性
        </p>
      </div>

      {/* 工作时间配置 */}
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h4 className="text-white font-medium mb-4">🕐 工作时间配置</h4>
        <div className="space-y-4">
          <label className="flex items-center justify-between">
            <span className="text-gray-300">启用工作时间限制</span>
            <input
              type="checkbox"
              checked={config.working_hours.enabled}
              onChange={(e) => updateConfig({
                working_hours: { ...config.working_hours, enabled: e.target.checked }
              })}
              className="w-5 h-5 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
            />
          </label>

          {config.working_hours.enabled && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <FormInput
                id="start_time"
                name="start_time"
                type="time"
                value={config.working_hours.start}
                onChange={(e) => updateConfig({
                  working_hours: { ...config.working_hours, start: e.target.value }
                })}
                label="开始时间"
              />
              <FormInput
                id="end_time"
                name="end_time"
                type="time"
                value={config.working_hours.end}
                onChange={(e) => updateConfig({
                  working_hours: { ...config.working_hours, end: e.target.value }
                })}
                label="结束时间"
              />
              <FormInput
                id="timezone"
                name="timezone"
                value={config.working_hours.timezone}
                onChange={(e) => updateConfig({
                  working_hours: { ...config.working_hours, timezone: e.target.value }
                })}
                label="时区"
                placeholder="UTC"
              />
            </div>
          )}
        </div>
      </div>

      {/* 危险操作 */}
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6">
        <h4 className="text-red-400 font-medium mb-4">⚠️ 危险操作</h4>
        <div className="space-y-3">
          <Button
            variant="secondary"
            className="bg-red-600 hover:bg-red-700 text-white"
            onClick={handleResetConfig}
            isLoading={isLoading}
          >
            重置为默认配置
          </Button>
          <Button
            variant="secondary"
            className="bg-red-600 hover:bg-red-700 text-white"
            onClick={handleClearQueues}
            isLoading={isLoading}
          >
            清空所有Cookie队列
          </Button>
        </div>
      </div>
    </div>
  );
};

export default AdvancedConfig;
