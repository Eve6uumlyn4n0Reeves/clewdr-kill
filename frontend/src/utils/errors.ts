export function getErrorMessage(err: unknown, fallback = '操作失败'): string {
  if (!err) return fallback;
  if (typeof err === 'string') return err || fallback;
  if (err instanceof Error) return err.message || fallback;
  if (typeof err === 'object') {
    const anyErr = err as Record<string, unknown>;
    if (typeof anyErr.message === 'string') return anyErr.message || fallback;
    if (typeof anyErr.error === 'string') return anyErr.error || fallback;
  }
  return fallback;
}

// 根据统一错误码返回友好提示
export function mapErrorCode(code?: string, fallback?: string) {
  if (!code) return fallback;
  switch (code) {
    case 'AUTH_FAILED':
      return '认证失败，请重新登录';
    case 'AUTH_RATE_LIMITED':
      return '登录过于频繁，请稍后再试';
    case 'COOKIE_FORMAT_INVALID':
      return 'Cookie 格式无效';
    case 'COOKIE_DUPLICATE':
      return 'Cookie 已在队列中';
    case 'RATE_LIMITED':
    case 'CLAUDE_RATE_LIMITED':
      return '触发限流，稍后将自动重试';
    case 'PROMPT_MISSING':
      return '提示词缺失，请先创建/导入提示词';
    case 'DB_ERROR':
      return '数据库错误，请稍后重试';
    case 'NOT_FOUND':
      return '资源不存在';
    default:
      return fallback;
  }
}
