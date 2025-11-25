/**
 * Unified API client for the ban tool
 * Centralizes authentication, error handling, and request logic
 */

const API_BASE = "/api";

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
      return JSON.parse(text) as T;
    } catch {
      return text as unknown as T;
    }
  }

  private async handleApiError(response: Response): Promise<never> {
    const status = response.status;
    let message: string;

    try {
      const errorData = await response.json();
      message = errorData.error?.message || errorData.message || `Error ${status}`;
    } catch {
      message = `HTTP ${status}: ${response.statusText}`;
    }

    // Map specific status codes to user-friendly errors
    switch (status) {
      case 400:
        throw new Error("Invalid request format");
      case 401:
        throw new Error("Authentication failed. Please check your admin token.");
      case 404:
        throw new Error("Resource not found");
      case 500:
        throw new Error("Server error. Please try again later.");
      default:
        throw new Error(message);
    }
  }

  /**
   * Get application version
   */
  async getVersion(): Promise<string> {
    const response = await fetch(`${API_BASE}/version`);
    return response.text();
  }

  /**
   * Validate authentication token
   */
  async validateToken(token?: string): Promise<boolean> {
    try {
      const authToken = token ?? this.getAuthToken();
      if (!authToken) return false;

      const response = await fetch(`${API_BASE}/auth`, {
        headers: {
          Authorization: `Bearer ${authToken}`,
        },
      });

      return response.ok;
    } catch {
      return false;
    }
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
   * Submit multiple cookies for banning
   */
  async submitMultipleCookies(cookies: string[]): Promise<{
    success: number;
    failed: number;
    results: Array<{
      cookie: string;
      success: boolean;
      message: string;
    }>;
  }> {
    const results = await Promise.allSettled(
      cookies.map((cookie) =>
        this.submitCookie(cookie)
          .then(() => ({
            cookie,
            success: true,
            message: "Cookie submitted successfully",
          }))
          .catch((error) => ({
            cookie,
            success: false,
            message: error instanceof Error ? error.message : "Unknown error",
          }))
      )
    );

    const fulfilledResults = results.map((result) =>
      result.status === "fulfilled" ? result.value : result.reason
    );

    const success = fulfilledResults.filter((r) => r.success).length;
    const failed = fulfilledResults.length - success;

    return {
      success,
      failed,
      results: fulfilledResults,
    };
  }

  /**
   * Get cookie queue status
   */
  async getCookieStatus(): Promise<any> {
    return this.request<any>("/cookies");
  }

  /**
   * Check if a cookie is still alive/banned
   */
  async checkCookieStatus(cookie: string): Promise<{
    alive: boolean;
    banned: boolean;
    lastChecked: string;
    error?: string;
  }> {
    const data = await this.request<any>("/cookie/check", {
      method: "POST",
      body: JSON.stringify({ cookie }),
    });

    return {
      alive: data.alive,
      banned: data.banned,
      lastChecked: data.last_checked ?? data.lastChecked,
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
      | "clear_queue"
      | "clear_banned"
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
  async getConfig(): Promise<{
    ban_config: any;
    server_config: { ip: string; port: number };
    network_config: { proxy: string | null; admin_password: string };
  }> {
    return this.get("/config");
  }

  async updateConfig(payload: unknown) {
    return this.post("/config", payload);
  }

  async resetConfig() {
    return this.post("/config/reset");
  }
}

// Export singleton instance
export const apiClient = new ApiClient();

// Export individual methods for backward compatibility
export const getVersion = () => apiClient.getVersion();
export const validateAuthToken = (token?: string) => apiClient.validateToken(token);
export const postCookie = (cookie: string) => apiClient.submitCookie(cookie);
export const postMultipleCookies = (cookies: string[]) => apiClient.submitMultipleCookies(cookies);
export const getCookieStatus = () => apiClient.getCookieStatus();
export const checkCookieStatus = (cookie: string) => apiClient.checkCookieStatus(cookie);
export const deleteCookie = (cookie: string) => apiClient.deleteCookie(cookie);
export const adminAction = (action: Parameters<ApiClient["adminAction"]>[0], params?: unknown) =>
  apiClient.adminAction(action, params);
export const clearPendingQueue = () => adminAction("clear_queue");
export const clearBannedQueue = () => adminAction("clear_banned");
export const pauseAllWorkers = () => adminAction("pause_all");
export const resumeAllWorkers = () => adminAction("resume_all");
export const emergencyStop = () => adminAction("emergency_stop");
export const getSystemStatus = () => apiClient.getSystemStatus();
export const fetchConfig = () => apiClient.getConfig();
export const saveConfig = (payload: unknown) => apiClient.updateConfig(payload);
export const resetConfig = () => apiClient.resetConfig();
