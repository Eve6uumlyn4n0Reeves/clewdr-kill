import React, { useState, useRef } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button, Message } from '../ui';
import {
  ArrowDownTrayIcon,
  ArrowUpTrayIcon,
  DocumentArrowDownIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
} from '@heroicons/react/24/outline';
import { apiClient } from '../../api';
import type { ConfigResponse } from '../../types/api.types';

interface ConfigImportExportProps {
  onConfigImported?: (config: ConfigResponse) => void;
}

const ConfigImportExport: React.FC<ConfigImportExportProps> = ({ onConfigImported }) => {
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [importResult, setImportResult] = useState<ConfigResponse | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleExport = async () => {
    setExporting(true);
    setMessage(null);

    try {
      const exportData = await apiClient.exportConfig();

      // 创建下载
      const blob = new Blob([JSON.stringify(exportData, null, 2)], {
        type: 'application/json'
      });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `clewdr-config-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      setMessage({
        type: 'success',
        text: '配置导出成功'
      });
    } catch (err) {
      setMessage({
        type: 'error',
        text: err instanceof Error ? err.message : '导出配置失败'
      });
    } finally {
      setExporting(false);
    }
  };

  const handleImport = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setImporting(true);
    setMessage(null);
    setImportResult(null);

    try {
      const text = await file.text();
      const config = JSON.parse(text);

      // 让用户选择合并模式
      const mergeMode = window.confirm(
        '是否使用合并模式？\n\n' +
        '确定(OK): 合并模式 - 保留现有配置，只更新导入的部分\n' +
        '取消(Cancel): 替换模式 - 完全替换现有配置'
      )
        ? 'merge'
        : 'replace';

      const result = await apiClient.importConfig(config, mergeMode as 'replace' | 'merge');
      setImportResult(result);

      setMessage({
        type: 'success',
        text: '配置导入成功',
      });
      onConfigImported?.(result);
    } catch (err) {
      setMessage({
        type: 'error',
        text: err instanceof Error ? err.message : '导入配置失败'
      });
    } finally {
      setImporting(false);
      // 清空文件输入，允许重复选择同一文件
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>配置导入导出</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-6">
          {/* 导出部分 */}
          <div>
            <h3 className="text-lg font-medium mb-3 flex items-center gap-2">
              <ArrowDownTrayIcon className="h-5 w-5" />
              导出配置
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              导出当前配置为JSON文件，可用于备份或迁移。
            </p>
            <Button
              onClick={handleExport}
              disabled={exporting}
              loading={exporting}
              icon={<DocumentArrowDownIcon className="h-4 w-4" />}
            >
              {exporting ? '导出中...' : '导出配置'}
            </Button>
          </div>

          {/* 导入部分 */}
          <div>
            <h3 className="text-lg font-medium mb-3 flex items-center gap-2">
              <ArrowUpTrayIcon className="h-5 w-5" />
              导入配置
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              从JSON文件导入配置。支持合并模式和替换模式。
            </p>
            <div className="flex items-center gap-4">
              <input
                ref={fileInputRef}
                type="file"
                accept=".json"
                onChange={handleImport}
                disabled={importing}
                className="block text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-primary file:text-white hover:file:bg-primary-600 cursor-pointer"
              />
              {importing && (
                <span className="text-sm text-gray-500">导入中...</span>
              )}
            </div>
          </div>

          {/* 消息提示 */}
          {message && (
            <Message type={message.type}>
              {message.text}
            </Message>
          )}

          {/* 导入结果 */}
          {importResult && (
            <div className="space-y-4">
              <Message type="success">
                <div className="flex items-center gap-2">
                  <CheckCircleIcon className="h-4 w-4" />
                  配置导入成功
                </div>
              </Message>

              <div className="bg-gray-50 dark:bg-gray-900/20 border border-gray-200 dark:border-gray-800 rounded-lg p-4 text-sm">
                <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                  当前 Ban 配置摘要
                </h4>
                <ul className="space-y-1 text-gray-700 dark:text-gray-300">
                  <li>
                    <span className="font-medium">并发数:</span>{' '}
                    {importResult.ban_config.concurrency}
                  </li>
                  <li>
                    <span className="font-medium">暂停时间:</span>{' '}
                    {importResult.ban_config.pause_seconds} 秒
                  </li>
                  <li>
                    <span className="font-medium">最大 Token 数:</span>{' '}
                    {importResult.ban_config.max_tokens}
                  </li>
                  <li>
                    <span className="font-medium">模型列表:</span>{' '}
                    {importResult.ban_config.models.join(', ')}
                  </li>
                </ul>
              </div>
            </div>
          )}

          {/* 使用说明 */}
          <div className="mt-6 p-4 bg-gray-50 dark:bg-gray-900/20 rounded-lg">
            <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">使用说明</h4>
            <ul className="space-y-1 text-sm text-gray-600 dark:text-gray-400">
              <li><strong>导出:</strong> 将当前配置保存为JSON文件，包含所有配置项和元数据</li>
              <li><strong>导入-合并模式:</strong> 将导入的配置与现有配置合并，只更新指定的字段</li>
              <li><strong>导入-替换模式:</strong> 完全替换现有配置，使用导入的全部配置</li>
              <li><strong>备份建议:</strong> 导入前建议先导出当前配置作为备份</li>
            </ul>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

export default ConfigImportExport;
