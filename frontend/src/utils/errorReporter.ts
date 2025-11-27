/**
 * 错误上报工具
 * 用于将前端错误上报到后端审计日志
 * 自用场景,简化为后端日志收集,不引入第三方服务
 */

interface ErrorReport {
  message: string;
  stack?: string;
  componentStack?: string;
  url: string;
  userAgent: string;
  timestamp: string;
}

class ErrorReporter {
  private endpoint: string;
  private queue: ErrorReport[] = [];
  private isReporting = false;
  private maxQueueSize = 50;

  constructor() {
    const baseUrl = import.meta.env.VITE_API_BASE_URL ?? "/api";
    this.endpoint = `${baseUrl}/admin/error-report`;
  }

  /**
   * 上报错误到后端
   */
  async report(error: Error, componentStack?: string): Promise<void> {
    const errorReport: ErrorReport = {
      message: error.message,
      stack: error.stack,
      componentStack,
      url: window.location.href,
      userAgent: navigator.userAgent,
      timestamp: new Date().toISOString(),
    };

    // 添加到队列
    this.queue.push(errorReport);

    // 超过队列大小限制,移除最旧的
    if (this.queue.length > this.maxQueueSize) {
      this.queue.shift();
    }

    // 立即上报(如果未在上报中)
    if (!this.isReporting) {
      await this.flush();
    }
  }

  /**
   * 批量上报队列中的错误
   */
  private async flush(): Promise<void> {
    if (this.isReporting || this.queue.length === 0) {
      return;
    }

    this.isReporting = true;

    try {
      const reports = [...this.queue];
      this.queue = [];

      const token = localStorage.getItem("authToken");
      if (!token) {
        // 未登录,仅console输出
        console.error("[ErrorReporter] Not authenticated, errors not reported:", reports);
        return;
      }

      const response = await fetch(this.endpoint, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({ errors: reports }),
      });

      if (!response.ok) {
        // 上报失败,重新加入队列(最多保留10条)
        console.error("[ErrorReporter] Failed to report errors:", response.statusText);
        this.queue.unshift(...reports.slice(0, 10));
      } else {
        console.log(`[ErrorReporter] Successfully reported ${reports.length} error(s)`);
      }
    } catch (err) {
      // 网络错误,静默失败
      console.error("[ErrorReporter] Network error:", err);
    } finally {
      this.isReporting = false;
    }
  }

  /**
   * 清空错误队列
   */
  clear(): void {
    this.queue = [];
  }
}

// 单例导出
export const errorReporter = new ErrorReporter();
