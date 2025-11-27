import React, { useState } from 'react';
import { Cog6ToothIcon, DocumentDuplicateIcon, FolderOpenIcon, DocumentArrowUpIcon, DocumentTextIcon } from '@heroicons/react/24/outline';
import { Card, CardHeader, CardTitle, CardContent, Message } from './ui';
import { AdvancedConfig } from './config';
import ConfigValidator from './config/ConfigValidator';
import ConfigTemplates from './config/ConfigTemplates';
import ConfigImportExport from './config/ConfigImportExport';
import { PromptManager } from './config/PromptManager';
import { apiClient } from '../api';
import type { BanConfig, ConfigResponse } from '../types/api.types';

type TabKey = 'basic' | 'validator' | 'templates' | 'import-export' | 'prompts';

const ConfigView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabKey>('basic');
  const [config, setConfig] = useState<ConfigResponse | null>(null);
  const [configRefreshKey, setConfigRefreshKey] = useState(0);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);

  const loadConfig = async () => {
    try {
      const data = await apiClient.getConfig();
      setConfig(data);
    } catch (err) {
      console.error('Failed to load config:', err);
    }
  };

  const handleTemplateSelect = async (template: Partial<BanConfig>) => {
    if (!config) return;

    try {
      // 使用模板的配置更新当前 ban_config，仅覆盖提供的字段
      const updatedConfig = {
        ...config,
        ban_config: {
          ...config.ban_config,
          ...template,
        },
      };

      await apiClient.updateConfig({ ban_config: updatedConfig.ban_config });
      setConfig(updatedConfig);
      setConfigRefreshKey((key) => key + 1);
      setMessage({ type: 'success', text: '模板应用成功' });
    } catch (err) {
      setMessage({
        type: 'error',
        text: err instanceof Error ? err.message : '应用模板失败'
      });
    }
  };

  React.useEffect(() => {
    loadConfig();
  }, []);

  const tabs: Array<{ key: TabKey; label: string; icon: typeof Cog6ToothIcon }> = [
    { key: 'basic', label: '基础配置', icon: Cog6ToothIcon },
    { key: 'validator', label: '配置验证', icon: DocumentDuplicateIcon },
    { key: 'templates', label: '配置模板', icon: FolderOpenIcon },
    { key: 'import-export', label: '导入导出', icon: DocumentArrowUpIcon },
    { key: 'prompts', label: 'Prompt管理', icon: DocumentTextIcon },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          系统配置
        </h1>
        <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
          管理封号策略和系统设置
        </p>
      </div>

      {/* Tab Navigation */}
      <Card>
        <CardContent className="p-0">
          <div className="flex border-b border-gray-200 dark:border-gray-700">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key)}
                  className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${
                    activeTab === tab.key
                      ? 'border-primary text-primary'
                      : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                  }`}
                >
                  <Icon className="h-4 w-4" />
                  <span className="font-medium">{tab.label}</span>
                </button>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* 消息提示 */}
      {message && (
        <Message type={message.type}>
          {message.text}
        </Message>
      )}

      {/* Tab Content */}
      <Card>
        {activeTab === 'basic' && (
          <>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Cog6ToothIcon className="h-5 w-5 text-primary" />
                基础配置
              </CardTitle>
            </CardHeader>
            <CardContent>
              <AdvancedConfig
                refreshKey={configRefreshKey}
                onConfigChange={(ban) =>
                  setConfig((prev) =>
                    prev ? { ...prev, ban_config: ban } : prev,
                  )
                }
              />
            </CardContent>
          </>
        )}

        {activeTab === 'validator' && (
          <ConfigValidator
            config={config}
            onValidationComplete={(result) => {
              if (result.valid) {
                setMessage({ type: 'success', text: '配置验证通过' });
              }
            }}
          />
        )}

        {activeTab === 'templates' && (
          <ConfigTemplates onTemplateSelect={handleTemplateSelect} />
        )}

        {activeTab === 'import-export' && (
          <ConfigImportExport
            onConfigImported={(newConfig) => {
              setConfig(newConfig);
              setConfigRefreshKey((key) => key + 1);
            }}
          />
        )}

        {activeTab === 'prompts' && <PromptManager />}
      </Card>

      {/* 功能说明 */}
      <Card>
        <CardHeader>
          <CardTitle>配置功能说明</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div>
              <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                基础配置
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                调整并发数、延迟时间、模型选择等核心参数，优化封号效率。
              </p>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                配置验证
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                检查配置的有效性，发现潜在问题和错误，确保系统稳定运行。
              </p>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                配置模板
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                快速应用预设配置方案，包括激进、平衡、隐蔽等模式。
              </p>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                导入导出
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                备份配置文件，批量部署配置，支持合并和替换两种模式。
              </p>
            </div>
            <div>
              <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                Prompt管理
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                管理封号提示词，支持创建、编辑、导入导出，方便GitHub同步。
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default ConfigView;
