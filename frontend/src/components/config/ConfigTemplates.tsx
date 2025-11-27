import React, { useState, useEffect } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button } from '../ui';
import { SparklesIcon, DocumentDuplicateIcon } from '@heroicons/react/24/outline';
import { apiClient } from '../../api';
import type { BanConfig, ConfigTemplates as TemplatesResponse } from '../../types/api.types';

interface ConfigTemplatesProps {
  onTemplateSelect?: (templateConfig: Partial<BanConfig>) => void;
}

const ConfigTemplates: React.FC<ConfigTemplatesProps> = ({ onTemplateSelect }) => {
  const [templates, setTemplates] = useState<TemplatesResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchTemplates = async () => {
      try {
        setLoading(true);
        const data = await apiClient.getConfigTemplates();
        setTemplates(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : '获取配置模板失败');
      } finally {
        setLoading(false);
      }
    };

    fetchTemplates();
  }, []);

  const handleSelectTemplate = (templateKey: keyof TemplatesResponse) => {
    if (!templates) return;
    const template = templates[templateKey];
    if (template && template.config) {
      onTemplateSelect?.(template.config);
    }
  };

  const getTemplateIcon = (key: string) => {
    switch (key) {
      case 'aggressive':
        return '🔥';
      case 'balanced':
        return '⚖️';
      case 'stealth':
        return '🕵️';
      default:
        return '⚙️';
    }
  };

  const getTemplateColor = (key: string) => {
    switch (key) {
      case 'aggressive':
        return 'border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20';
      case 'balanced':
        return 'border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-900/20';
      case 'stealth':
        return 'border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-900/20';
      default:
        return 'border-purple-200 bg-purple-50 dark:border-purple-800 dark:bg-purple-900/20';
    }
  };

  if (loading) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="animate-pulse space-y-4">
            <div className="h-4 bg-gray-200 rounded w-1/4"></div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {[...Array(3)].map((_, i) => (
                <div key={i} className="h-32 bg-gray-200 rounded"></div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardContent className="p-6 text-center text-red-600">
          <p>加载配置模板失败</p>
          <p className="text-sm mt-1">{error}</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <SparklesIcon className="h-5 w-5 text-primary" />
          配置模板
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            选择预设配置模板，快速应用优化过的配置方案。
          </p>

          {templates && (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {Object.entries(templates).map(([key, template]) => (
                <div
                  key={key}
                  className={`border rounded-lg p-4 cursor-pointer transition-all hover:shadow-md ${getTemplateColor(
                    key
                  )}`}
                  onClick={() => handleSelectTemplate(key as keyof TemplatesResponse)}
                >
                  <div className="flex items-center gap-3 mb-2">
                    <span className="text-2xl">{getTemplateIcon(key)}</span>
                    <h3 className="font-semibold">{template.name}</h3>
                  </div>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                    {template.description}
                  </p>
                  <div className="space-y-1">
                    <div className="text-xs">
                      <span className="font-medium">并发数:</span>{' '}
                      <span className="text-gray-900 dark:text-gray-100">
                        {template.config?.concurrency || 'N/A'}
                      </span>
                    </div>
                    <div className="text-xs">
                      <span className="font-medium">延迟:</span>{' '}
                      <span className="text-gray-900 dark:text-gray-100">
                        {template.config?.pause_seconds ? `${template.config.pause_seconds}秒` : 'N/A'}
                      </span>
                    </div>
                    <div className="text-xs">
                      <span className="font-medium">模型:</span>{' '}
                      <span className="text-gray-900 dark:text-gray-100">
                        {template.config?.models?.[0] || 'N/A'}
                      </span>
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="mt-3 w-full"
                    icon={<DocumentDuplicateIcon className="h-4 w-4" />}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleSelectTemplate(key);
                    }}
                  >
                    应用模板
                  </Button>
                </div>
              ))}
            </div>
          )}

          {/* 模板说明 */}
          <div className="mt-6 p-4 bg-gray-50 dark:bg-gray-900/20 rounded-lg">
            <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">模板说明</h4>
            <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
              <li>
                <strong>激进模式 (🔥):</strong> 最高并发，最短延迟，追求最大效率
              </li>
              <li>
                <strong>平衡模式 (⚖️):</strong> 中等并发，适中延迟，平衡效率和稳定性
              </li>
              <li>
                <strong>隐蔽模式 (🕵️):</strong> 低并发，长延迟，避免被检测
              </li>
            </ul>
            <p className="text-xs mt-3 text-gray-500">
              注意：应用模板将覆盖当前的配置设置，请谨慎操作。
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

export default ConfigTemplates;
