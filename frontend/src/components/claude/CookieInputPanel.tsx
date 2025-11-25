import React, { useState, useCallback } from "react";
import Button from "../common/Button";
import FormInput from "../common/FormInput";
import StatusMessage from "../common/StatusMessage";
import { postMultipleCookies } from "../../api";

interface CookieInputPanelProps {
  onSubmit?: (cookies: string[]) => void;
  isSubmitting?: boolean;
}

const CookieInputPanel: React.FC<CookieInputPanelProps> = ({
  onSubmit,
  isSubmitting = false,
}) => {
  const [inputText, setInputText] = useState("");
  const [cookies, setCookies] = useState<string[]>([]);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error" | "info" | "warning" | null; text: string }>({
    type: null,
    text: "",
  });

  // Cookie格式验证函数
  const validateCookie = useCallback((cookie: string): boolean => {
    // 支持多种格式：
    // 1. 完整格式：sk-ant-sid01-xxx...AAA
    // 2. 简化格式：xxx...AAA
    const cleanCookie = cookie.replace(/[^0-9A-Za-z_-]/g, "");
    const fullPattern = /^sk-ant-sid01-([0-9A-Za-z_-]{86}-[0-9A-Za-z_-]{6}AA)$/;
    const shortPattern = /^([0-9A-Za-z_-]{86}-[0-9A-Za-z_-]{6}AA)$/;

    return fullPattern.test(cookie) || shortPattern.test(cleanCookie);
  }, []);

  // 处理输入变化
  const handleInputChange = useCallback((value: string) => {
    setInputText(value);

    // 解析输入的cookies
    const lines = value.split('\n').map(line => line.trim()).filter(line => line.length > 0);
    const validCookies: string[] = [];
    const errors: string[] = [];

    lines.forEach((cookie, index) => {
      if (validateCookie(cookie)) {
        validCookies.push(cookie);
      } else if (cookie.length > 0) {
        errors.push(`第${index + 1}行: "${cookie.substring(0, 20)}..." 格式无效`);
      }
    });

    setCookies(validCookies);
    setValidationErrors(errors);
  }, [validateCookie]);

  const handleSubmit = useCallback(async () => {
    if (cookies.length === 0 || submitting || isSubmitting) {
      return;
    }
    setSubmitting(true);
    setMessage({ type: "info", text: "正在提交..." });
    try {
      const result = await postMultipleCookies(cookies);
      if (result.failed === 0) {
        setMessage({ type: "success", text: `已提交 ${result.success} 个 Cookie` });
        setInputText("");
        setCookies([]);
        setValidationErrors([]);
      } else if (result.success === 0) {
        setMessage({ type: "error", text: `全部失败，失败 ${result.failed} 个` });
      } else {
        setMessage({
          type: "warning",
          text: `部分成功：成功 ${result.success}，失败 ${result.failed}`,
        });
      }
      onSubmit?.(cookies);
    } catch (err) {
      const msg = err instanceof Error ? err.message : "提交失败";
      setMessage({ type: "error", text: msg });
    } finally {
      setSubmitting(false);
    }
  }, [cookies, submitting, isSubmitting, onSubmit]);

  // 清空输入
  const handleClear = useCallback(() => {
    setInputText("");
    setCookies([]);
    setValidationErrors([]);
  }, []);

  // 示例Cookie格式
  const exampleCookie = "sk-ant-sid01----------------------------EXAMPLE_COOKIE_HERE----------------------------------------AAAAAA";

  return (
    <div className="space-y-4">
      {/* 格式说明 */}
      <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
        <h4 className="text-white font-medium mb-2">📝 Cookie格式说明</h4>
        <div className="space-y-2 text-sm text-gray-300">
          <p>支持以下格式：</p>
          <div className="bg-gray-900 rounded p-2 font-mono text-xs break-all">
            {exampleCookie}
          </div>
          <p className="text-gray-400">• 支持完整格式或核心部分</p>
          <p className="text-gray-400">• 每行一个Cookie，支持批量提交</p>
          <p className="text-gray-400">• 自动过滤无效格式并提示</p>
        </div>
      </div>

      {/* 输入区域 */}
      <FormInput
        id="enhanced-cookie-input"
        name="cookie"
        value={inputText}
        onChange={(e) => handleInputChange(e.target.value)}
        placeholder="在此粘贴Cookie（每行一个）..."
        label="目标Cookie"
        isTextarea={true}
        rows={8}
        onClear={handleClear}
        disabled={isSubmitting || submitting}
      />

      {/* 实时统计 */}
      <div className="flex flex-wrap gap-4 text-sm">
        <div className="flex items-center gap-2">
          <span className="text-gray-400">解析结果：</span>
          <span className="text-green-400 font-medium">
            ✓ 有效: {cookies.length}
          </span>
        </div>
        {validationErrors.length > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-red-400 font-medium">
              ✗ 无效: {validationErrors.length}
            </span>
          </div>
        )}
      </div>

      {/* 验证错误提示 */}
      {validationErrors.length > 0 && (
        <div className="bg-red-900/30 border border-red-800 rounded-lg p-3">
          <h5 className="text-red-400 font-medium mb-2">格式错误：</h5>
          <ul className="space-y-1 text-xs text-red-300">
            {validationErrors.map((error, index) => (
              <li key={index} className="flex items-start gap-2">
                <span>•</span>
                <span>{error}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 预览有效Cookie */}
      {cookies.length > 0 && (
        <div className="bg-green-900/30 border border-green-800 rounded-lg p-3">
          <h5 className="text-green-400 font-medium mb-2">即将提交的Cookie：</h5>
          <div className="space-y-1 max-h-32 overflow-y-auto">
            {cookies.map((cookie, index) => (
              <div key={index} className="text-xs text-green-300 font-mono">
                {index + 1}. {cookie.substring(0, 50)}...
              </div>
            ))}
          </div>
        </div>
      )}

      {message.type && (
        <StatusMessage type={message.type} message={message.text} />
      )}

      {/* 提交按钮 */}
      <div className="flex gap-3">
        <Button
          onClick={handleSubmit}
          disabled={cookies.length === 0 || isSubmitting || submitting}
          isLoading={isSubmitting || submitting}
          className="flex-1"
        >
          {isSubmitting || submitting ? "处理中..." : `添加 ${cookies.length} 个Cookie到队列`}
        </Button>
        <Button
          onClick={handleClear}
          variant="secondary"
          disabled={isSubmitting || submitting}
        >
          清空
        </Button>
      </div>
    </div>
  );
};

export default CookieInputPanel;
