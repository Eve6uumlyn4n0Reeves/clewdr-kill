import React, { useState } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button, Message } from '../ui';
import { CheckCircleIcon, ExclamationTriangleIcon, DocumentTextIcon } from '@heroicons/react/24/outline';
import { apiClient } from '../../api';
import type { ConfigResponse, ConfigValidationResult } from '../../types/api.types';

interface ConfigValidatorProps {
  config: ConfigResponse | null;
  onValidationComplete?: (result: ConfigValidationResult) => void;
}

const ConfigValidator: React.FC<ConfigValidatorProps> = ({ config, onValidationComplete }) => {
  const [validating, setValidating] = useState(false);
  const [result, setResult] = useState<ConfigValidationResult | null>(null);

  const handleValidate = async () => {
    if (!config) {
      setResult({
        valid: false,
        errors: ['配置尚未加载，无法验证'],
        warnings: [],
      });
      return;
    }

    setValidating(true);
    setResult(null);

    try {
      const validation = await apiClient.validateConfig(config);
      setResult(validation);
      onValidationComplete?.(validation);
    } catch (err) {
      setResult({
        valid: false,
        errors: [err instanceof Error ? err.message : '验证失败'],
        warnings: [],
      });
    } finally {
      setValidating(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <DocumentTextIcon className="h-5 w-5 text-primary" />
          配置验证
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            验证当前配置的有效性，检查潜在问题和冲突。
          </p>

          <Button
            onClick={handleValidate}
            disabled={validating}
            loading={validating}
            className="w-full"
          >
            验证配置
          </Button>

          {result && (
            <div className="space-y-4">
              {result.valid ? (
                <Message type="success">
                  <div className="flex items-center gap-2">
                    <CheckCircleIcon className="h-4 w-4" />
                    配置验证通过
                  </div>
                </Message>
              ) : (
                <Message type="error">
                  <div className="flex items-center gap-2">
                    <ExclamationTriangleIcon className="h-4 w-4" />
                    配置验证失败
                  </div>
                </Message>
              )}

              {/* 错误列表 */}
              {result.errors && result.errors.length > 0 && (
                <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                  <h4 className="font-medium text-red-900 dark:text-red-100 mb-2">错误</h4>
                  <ul className="space-y-1">
                    {result.errors.map((error: string, index: number) => (
                      <li key={index} className="text-sm text-red-700 dark:text-red-300">
                        • {error}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {/* 警告列表 */}
              {result.warnings && result.warnings.length > 0 && (
                <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                  <h4 className="font-medium text-yellow-900 dark:text-yellow-100 mb-2">警告</h4>
                  <ul className="space-y-1">
                    {result.warnings.map((warning: string, index: number) => (
                      <li key={index} className="text-sm text-yellow-700 dark:text-yellow-300">
                        • {warning}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {/* 详细信息 */}
              {result.details && result.details.length > 0 && (
                <div className="space-y-2">
                  <h4 className="font-medium text-gray-900 dark:text-gray-100">详细信息</h4>
                  <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b dark:border-gray-700">
                          <th className="text-left py-2">部分</th>
                          <th className="text-left py-2">字段</th>
                          <th className="text-left py-2">消息</th>
                          <th className="text-left py-2">级别</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.details.map((detail, index: number) => (
                          <tr key={index} className="border-b dark:border-gray-700">
                            <td className="py-2">{detail.section}</td>
                            <td className="py-2 font-mono text-xs">{detail.field}</td>
                            <td className="py-2">{detail.message}</td>
                            <td className="py-2">
                              <span
                                className={`px-2 py-1 rounded text-xs ${
                                  detail.level === 'error'
                                    ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300'
                                    : 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300'
                                }`}
                              >
                                {detail.level}
                              </span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default ConfigValidator;
