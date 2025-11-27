import React, { useEffect, useState } from 'react';
import { Button, Card, CardHeader, CardTitle, Input, Message } from '../ui';
import { apiClient } from '../../api';
import { getErrorMessage } from '../../utils/errors';
import type { BanConfig } from '../../types/api.types';

interface AdvancedConfigProps {
  onConfigChange?: (config: BanConfig) => void;
}

const AdvancedConfig: React.FC<AdvancedConfigProps & { refreshKey?: number }> = ({
  onConfigChange,
  refreshKey,
}) => {
  const [config, setConfig] = useState<BanConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [newModel, setNewModel] = useState('');

  useEffect(() => {
    const load = async () => {
      try {
        const data = await apiClient.getConfig();
        if (data?.ban_config) {
          setConfig(data.ban_config);
          onConfigChange?.(data.ban_config);
        }
      } catch (err) {
        setMessage({ type: 'error', text: getErrorMessage(err, '加载配置失败') });
      }
    };
    load();
  }, [onConfigChange, refreshKey]);

  const updateConfig = (updates: Partial<BanConfig>) => {
    if (!config) return;
    const next = { ...config, ...updates };
    setConfig(next);
    onConfigChange?.(next);
  };

  const handleSave = async () => {
    if (!config) return;
    setLoading(true);
    setMessage(null);
    try {
      await apiClient.updateConfig({ ban_config: config });
      setMessage({ type: 'success', text: '配置保存成功' });
    } catch (err) {
      setMessage({ type: 'error', text: getErrorMessage(err, '配置保存失败') });
    } finally {
      setLoading(false);
    }
  };

  const handleReset = async () => {
    setLoading(true);
    setMessage(null);
    try {
      const data = await apiClient.resetConfig();
      if (data?.ban_config) {
        setConfig(data.ban_config);
        onConfigChange?.(data.ban_config);
      }
      setMessage({ type: 'success', text: '已恢复默认配置' });
    } catch (err) {
      setMessage({ type: 'error', text: getErrorMessage(err, '恢复默认配置失败') });
    } finally {
      setLoading(false);
    }
  };

  const handleClearQueues = async () => {
    setLoading(true);
    setMessage(null);
    try {
      await apiClient.adminAction('clear_queue');
      await apiClient.adminAction('clear_banned');
      setMessage({ type: 'success', text: '队列已清空' });
    } catch (err) {
      setMessage({ type: 'error', text: getErrorMessage(err, '清空队列失败') });
    } finally {
      setLoading(false);
    }
  };

  const handleAddModel = () => {
    if (!config || !newModel.trim()) return;
    if (config.models.includes(newModel.trim())) {
      setNewModel('');
      return;
    }
    updateConfig({ models: [...config.models, newModel.trim()] });
    setNewModel('');
  };

  const handleRemoveModel = (index: number) => {
    if (!config) return;
    updateConfig({ models: config.models.filter((_, i) => i !== index) });
  };

  if (!config) {
    return <Message type="info">正在加载配置...</Message>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white">⚙️ 高级配置管理</h3>
        <Button onClick={handleSave} loading={loading}>
          保存配置
        </Button>
      </div>

      {message && (
        <Message type={message.type}>
          {message.text}
        </Message>
      )}

      <Card>
        <CardHeader>
          <CardTitle>基础封号配置</CardTitle>
        </CardHeader>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 p-6">
          <Input
            label="并发工作线程数"
            type="number"
            min={1}
            max={50}
            value={config.concurrency}
            onChange={(e) => updateConfig({ concurrency: parseInt(e.target.value) || 1 })}
          />
          <Input
            label="限流后暂停时间（秒）"
            type="number"
            min={60}
            max={3600}
            value={config.pause_seconds}
            onChange={(e) => updateConfig({ pause_seconds: parseInt(e.target.value) || 60 })}
          />
          <Input
            label="最大 Token 数"
            type="number"
            min={100}
            max={4096}
            value={config.max_tokens}
            onChange={(e) => updateConfig({ max_tokens: parseInt(e.target.value) || 100 })}
          />
          <Input
            label="请求超时（毫秒）"
            type="number"
            min={5000}
            max={120000}
            value={config.request_timeout}
            onChange={(e) => updateConfig({ request_timeout: parseInt(e.target.value) || 5000 })}
          />
          <Input
            label="提示词目录"
            type="text"
            value={config.prompts_dir}
            onChange={(e) => updateConfig({ prompts_dir: e.target.value })}
          />
        </div>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>模型配置</CardTitle>
        </CardHeader>
        <div className="p-6 space-y-4">
          <div className="flex gap-2">
            <Input
              placeholder="输入模型名称后点击添加"
              value={newModel}
              onChange={(e) => setNewModel(e.target.value)}
            />
            <Button variant="secondary" onClick={handleAddModel} disabled={!newModel.trim()}>
              添加
            </Button>
          </div>
          <div className="space-y-2">
            {config.models.map((model, index) => (
              <div
                key={model}
                className="flex items-center justify-between bg-gray-50 dark:bg-gray-800 px-3 py-2 rounded"
              >
                <span className="text-sm font-mono">{model}</span>
                <Button variant="ghost" size="sm" onClick={() => handleRemoveModel(index)}>
                  删除
                </Button>
              </div>
            ))}
            {config.models.length === 0 && (
              <p className="text-sm text-gray-500">暂无配置的模型</p>
            )}
          </div>
        </div>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>危险操作</CardTitle>
        </CardHeader>
        <div className="p-6 space-y-3">
          <Button variant="secondary" loading={loading} onClick={handleReset}>
            重置为默认配置
          </Button>
          <Button variant="danger" loading={loading} onClick={handleClearQueues}>
            清空所有 Cookie 队列
          </Button>
        </div>
      </Card>
    </div>
  );
};

export default AdvancedConfig;
