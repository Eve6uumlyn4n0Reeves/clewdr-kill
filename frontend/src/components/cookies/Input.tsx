import React, { useState, useCallback } from 'react';
import { ClipboardDocumentIcon, PlusIcon } from '@heroicons/react/24/outline';
import { Button, Card, CardHeader, CardTitle, Message } from '../ui';
import { apiClient } from '../../api';
import { getErrorMessage } from '../../utils/errors';

interface CookieInputProps {
  onSubmit?: (cookies: string[]) => void;
}

const CookieInput: React.FC<CookieInputProps> = ({ onSubmit }) => {
  const [inputText, setInputText] = useState('');
  const [cookies, setCookies] = useState<string[]>([]);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info' | 'warning'; text: string } | null>(null);
  const [progress, setProgress] = useState<{ current: number; total: number } | null>(null);

  // Cookie格式验证 - 简化版本
  const validateCookie = useCallback((cookie: string): boolean => {
    // 基本格式检查：以 sk-ant- 开头，长度合理
    if (!cookie.startsWith('sk-ant-')) return false;
    if (cookie.length < 80 || cookie.length > 200) return false;
    // 检查是否只包含有效字符
    return /^[a-zA-Z0-9_-]+$/.test(cookie);
  }, []);

  // 处理输入变化
  const handleInputChange = useCallback((value: string) => {
    setInputText(value);

    const lines = value.split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0);

    const validCookies: string[] = [];
    const errors: string[] = [];

    lines.forEach((cookie, index) => {
      if (validateCookie(cookie)) {
        validCookies.push(cookie);
      } else if (cookie.length > 0) {
        errors.push(`第${index + 1}行: 格式无效`);
      }
    });

    setCookies(validCookies);
    setValidationErrors(errors);
  }, [validateCookie]);

  const handleSubmit = useCallback(async () => {
    if (cookies.length === 0 || submitting) {
      return;
    }

    setSubmitting(true);
    setMessage({ type: 'info', text: '正在提交...' });

    try {
      const result = await apiClient.submitMultipleCookies(cookies);

      if (result.failed === 0) {
        setMessage({ type: 'success', text: `成功提交 ${result.success} 个 Cookie` });
        setInputText('');
        setCookies([]);
        setValidationErrors([]);
        onSubmit?.(cookies);
      } else if (result.success === 0) {
        const errorMsg = result.errors && result.errors.length > 0
          ? `全部提交失败。错误：${result.errors.slice(0, 3).join('; ')}${result.errors.length > 3 ? '...' : ''}`
          : `全部提交失败`;
        setMessage({ type: 'error', text: errorMsg });
      } else {
        const warningMsg = result.errors && result.errors.length > 0
          ? `部分成功：成功 ${result.success}，失败 ${result.failed}。错误示例：${result.errors.slice(0, 2).join('; ')}`
          : `部分成功：成功 ${result.success}，失败 ${result.failed}`;
        setMessage({
          type: 'warning',
          text: warningMsg,
        });
        onSubmit?.(cookies);
      }
    } catch (err) {
      setMessage({ type: 'error', text: getErrorMessage(err, '提交失败，请稍后重试') });
    } finally {
      setSubmitting(false);
      setProgress(null);
    }
  }, [cookies, submitting, onSubmit]);

  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      handleInputChange(text);
    } catch (err) {
      console.error('Failed to read clipboard:', err);
    }
  }, [handleInputChange]);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <PlusIcon className="h-5 w-5 text-primary" />
          添加 Cookie
        </CardTitle>
      </CardHeader>

      <div className="space-y-4">
        <div>
          <div className="flex justify-between items-center mb-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Cookie 列表（每行一个）
            </label>
            <Button
              variant="ghost"
              size="sm"
              onClick={handlePaste}
              icon={<ClipboardDocumentIcon className="h-4 w-4" />}
            >
              粘贴
            </Button>
          </div>
          <textarea
            value={inputText}
            onChange={(e) => handleInputChange(e.target.value)}
            placeholder="sk-ant-sid01-xxx...&#10;sk-ant-sid01-yyy...&#10;..."
            className="w-full h-32 px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 dark:bg-gray-800 dark:border-gray-600 dark:text-white dark:placeholder-gray-400 resize-none"
            disabled={submitting}
          />
        </div>

        {/* 验证状态 */}
        {inputText && (
          <div className="space-y-2">
            <div className="flex gap-4 text-sm">
              <span className="text-success-600">
                有效: {cookies.length} 个
              </span>
              {validationErrors.length > 0 && (
                <span className="text-error-600">
                  无效: {validationErrors.length} 个
                </span>
              )}
            </div>

            {/* 验证错误 */}
            {validationErrors.length > 0 && (
              <div className="bg-error-50 border border-error-200 rounded-lg p-3 dark:bg-error-900/20 dark:border-error-800">
                <p className="text-sm font-medium text-error-600 mb-1 dark:text-error-400">
                  以下 Cookie 格式无效：
                </p>
                <ul className="text-sm text-error-500 space-y-1">
                  {validationErrors.map((error, index) => (
                    <li key={index}>{error}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        {/* 消息提示 */}
        {message && (
          <Message type={message.type}>
            {message.text}
          </Message>
        )}

        {/* 提交按钮 */}
        <Button
          onClick={handleSubmit}
          disabled={cookies.length === 0 || submitting}
          loading={submitting}
          className="w-full"
        >
          {submitting ? '提交中...' : `提交 ${cookies.length} 个 Cookie`}
        </Button>
      </div>
    </Card>
  );
};

export default CookieInput;
