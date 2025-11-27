/**
 * Unified API client for the ban tool
 * Centralizes authentication, error handling, and request logic
 */

import type {
  QueueStatusResponse,
  ConfigResponse,
  SystemStats,
  CookieMetrics,
  HistoricalStats,
  HistoricalStatsParams,
  ConfigValidationResult,
  ConfigExport,
  ConfigTemplates,
  PromptFile,
  AuthLoginResponse,
  CookieCheckResponse,
  ErrorCode,
} from "../types/api.types";
import { mapErrorCode } from "../utils/errors";

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "/api";

export class ApiClient {
  private getAuthToken(): string {
    return localStorage.getItem("authToken") || "";
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${API_BASE}${endpoint}`;
    const token = this.getAuthToken();

    const defaultHeaders: HeadersInit = {
      "Content-Type": "application/json",
      ...(token && { Authorization: `Bearer ${token}` }),
    };

    const response = await fetch(url, {
      ...options,
      headers: { ...defaultHeaders, ...options.headers },
    });

    if (!response.ok) {
      await this.handleApiError(response);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    const contentLength = response.headers.get("content-length");
    if (contentLength === "0") {
      return undefined as T;
    }

    const text = await response.text();
    if (!text) {
      return undefined as T;
    }

    try {
      const parsed = JSON.parse(text);

      // 如果是标准 ApiResponse 包装，则根据 success 解包 data 字段
      if (
        parsed &&
        typeof parsed === "object" &&
        "success" in parsed &&
        ("data" in parsed || "error" in parsed)
      ) {
        if (!parsed.success) {
          const apiErr = parsed.error ?? {};
          const message =
            apiErr.message ?? parsed.message ?? "Unknown API error";
          const code = apiErr.code as string | undefined;
          const err = new Error(message);
          (err as any).code = code;
          throw err;
        }
        return (parsed.data ?? undefined) as T;
      }

      // 否则按原始数据返回
      return parsed as T;
    } catch {
      return text as unknown as T;
    }
  }

  private async handleApiError(response: Response): Promise<never> {
    const status = response.status;
    let message: string;
    let code: string | undefined;
    let errorData: any;

    try {
      errorData = await response.json();
      message = errorData.error?.message || errorData.message || `Error ${status}`;
      code = errorData.error?.code;
    } catch {
      message = `HTTP ${status}: ${response.statusText}`;
    }

    const friendly = mapErrorCode(code, message);
    const err = new Error(friendly) as any;
    err.code = code as ErrorCode | undefined;
    err.status = status;
    throw err;
  }

  /**
   * Get application version
   */
  async getVersion(): Promise<string> {
    // 通过统一 request 逻辑获取版本信息（解包 ApiResponse）
    return this.get<string>("/version");
  }

  /**
   * Validate authentication token
   */
  async validateToken(token?: string): Promise<boolean> {
    try {
      const authToken = token ?? this.getAuthToken();
      if (!authToken) return false;

      await this.request<void>("/auth", {
        method: "GET",
        headers: {
          Authorization: `Bearer ${authToken}`,
        },
      });

      return true;
    } catch {
      return false;
    }
  }

  /**
   * Login with admin password and receive JWT token
   */
  async login(password: string): Promise<AuthLoginResponse> {
    return this.request<AuthLoginResponse>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ password }),
    });
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "GET" });
  }

  async post<T>(endpoint: string, body?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  }

  async delete<T>(endpoint: string, body?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "DELETE",
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  }

  // ---- Domain-specific helpers ----

  /**
   * Submit a single cookie for banning
   */
  async submitCookie(cookie: string): Promise<void> {
    await this.request<void>("/cookie", {
      method: "POST",
      body: JSON.stringify({ cookie }),
    });
  }

  /**
   * Submit multiple cookies for banning (optimized batch API)
   */
  async submitMultipleCookies(
    cookies: string[],
    batchSize: number = 10
  ): Promise<{
    success: number;
    failed: number;
    errors?: string[];
  }> {
    // Validate cookies before sending
    const validCookies = cookies.filter(cookie =>
      cookie && cookie.trim().length > 0
    );

    if (validCookies.length === 0) {
      return { success: 0, failed: cookies.length };
    }

    try {
      const result = await this.request<{
        success: number;
        failed: number;
        total: number;
        errors: string[];
      }>("/cookies/batch", {
        method: "POST",
        body: JSON.stringify({
          cookies: validCookies,
          batch_size: Math.min(batchSize, 100) // Cap at 100 for safety
        })
      });

      return {
        success: result.success,
        failed: result.failed,
        errors: result.errors
      };
    } catch (error) {
      // Fallback to individual submissions if batch fails
      console.warn("Batch submission failed, falling back to individual submissions");
      return this.submitCookiesIndividually(cookies);
    }
  }

  /**
   * Fallback method: Submit cookies individually
   */
  private async submitCookiesIndividually(cookies: string[]): Promise<{
    success: number;
    failed: number;
  }> {
    let success = 0;
    let failed = 0;

    // Process in smaller parallel batches
    const promises: Promise<void>[] = [];

    for (const cookie of cookies) {
      if (!cookie || !cookie.trim()) {
        failed++;
        continue;
      }

      const promise = this.submitCookie(cookie)
        .then(() => {
          success++;
        })
        .catch(() => {
          failed++;
        });

      promises.push(promise);

      // Process in batches of 5 to avoid overwhelming the server
      if (promises.length >= 5) {
        await Promise.allSettled(promises);
        promises.length = 0;
      }
    }

    // Wait for any remaining promises
    if (promises.length > 0) {
      await Promise.allSettled(promises);
    }

    return { success, failed };
  }

  /**
   * Get cookie queue status
   */
  async getCookieStatus(): Promise<QueueStatusResponse> {
    return this.request<QueueStatusResponse>("/cookies");
  }

  /**
   * Backward compatible alias for cookie queue
   */
  async getCookies(): Promise<QueueStatusResponse> {
    return this.getCookieStatus();
  }

  /**
   * Check if a cookie is still alive/banned
   */
  async checkCookieStatus(cookie: string): Promise<CookieCheckResponse> {
    const data = await this.request<CookieCheckResponse & {
      last_checked?: string;
    }>("/cookie/check", {
      method: "POST",
      body: JSON.stringify({ cookie }),
    });

    return {
      status: data.status,
      alive: data.alive,
      banned: data.banned,
      lastChecked: data.lastChecked ?? data.last_checked ?? data.lastChecked,
      error: data.error,
    };
  }

  /**
   * Delete a cookie from the queue
   */
  async deleteCookie(cookie: string): Promise<void> {
    await this.request<void>("/cookie", {
      method: "DELETE",
      body: JSON.stringify({ cookie }),
    });
  }

  /**
   * Admin actions
   */
  async adminAction(
    action:
      | "pause_all"
      | "resume_all"
      | "reset_stats"
      | "clear_all"
      | "emergency_stop",
    params?: unknown,
  ) {
    return this.post<{ success: boolean; message: string }>("/admin/action", {
      action,
      params,
    });
  }

  async getSystemStatus(): Promise<{
    status: string;
    uptime_seconds: number;
    active_workers: number;
    queue_size: number;
    banned_count: number;
    total_requests: number;
    maintenance_mode: boolean;
  }> {
    return this.get("/admin/status");
  }

  /**
   * Config helpers
   */
  async getConfig(): Promise<ConfigResponse> {
    return this.get<ConfigResponse>("/config");
  }

  async updateConfig(payload: unknown) {
    return this.post("/config", payload);
  }

  async resetConfig() {
    return this.post("/config/reset");
  }

  // ---- Statistics API methods ----

  /**
   * Get system statistics
   */
  async getSystemStats(): Promise<SystemStats> {
    return this.get<SystemStats>("/stats/system");
  }

  /**
   * Get cookie metrics
   */
  async getCookieMetrics(): Promise<CookieMetrics[]> {
    return this.get<CookieMetrics[]>("/stats/cookies");
  }

  /**
   * Get historical statistics
   */
  async getHistoricalStats(params: HistoricalStatsParams): Promise<HistoricalStats> {
    return this.post<HistoricalStats>("/stats/historical", params);
  }

  /**
   * Reset statistics
   */
  async resetStats(): Promise<void> {
    return this.post("/stats/reset");
  }

  // ---- Advanced Config API methods ----

  /**
   * Validate configuration
   */
  async validateConfig(config: unknown): Promise<ConfigValidationResult> {
    return this.post<ConfigValidationResult>("/config/validate", config);
  }

  /**
   * Export configuration
   */
  async exportConfig(): Promise<ConfigExport> {
    return this.get<ConfigExport>("/config/export");
  }

  /**
   * Import configuration
   */
  async importConfig(
    config: unknown,
    mergeMode?: "replace" | "merge",
  ): Promise<ConfigResponse> {
    return this.post<ConfigResponse>("/config/import", {
      config,
      merge_mode: mergeMode,
    });
  }

  /**
   * Get configuration templates
   */
  async getConfigTemplates(): Promise<ConfigTemplates> {
    return this.get<ConfigTemplates>("/config/templates");
  }

  // ---- Prompt Management API methods ----

  /**
   * Get all prompt files
   */
  async getPrompts(): Promise<PromptFile[]> {
    return this.get<PromptFile[]>("/prompts");
  }

  /**
   * Get a specific prompt file
   */
  async getPrompt(name: string): Promise<PromptFile> {
    return this.post<PromptFile>("/prompts/get", { name });
  }

  /**
   * Save (create or update) a prompt file
   */
  async savePrompt(name: string, content: string): Promise<PromptFile> {
    return this.post<PromptFile>("/prompts/save", { name, content });
  }

  /**
   * Delete a prompt file
   */
  async deletePrompt(name: string): Promise<void> {
    return this.post<void>("/prompts/delete", { name });
  }
}

// Export singleton instance
export const apiClient = new ApiClient();
