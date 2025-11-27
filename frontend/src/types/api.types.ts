// Frontend API-facing types
// 这些类型与后端实际返回结构对齐，供前端使用

// 队列与 Cookie 相关 --------------------------------------------

export interface BanCookie {
  cookie: string;
  submitted_at?: string | null;
  last_used_at?: string | null;
  requests_sent: number;
  is_banned: boolean;
  error_message?: string | null;
}

export type QueueCookie = BanCookie;

export interface QueueStatusResponse {
  pending: QueueCookie[];
  processing: QueueCookie[];
  banned: QueueCookie[];
  total_requests: number;
}

export interface BatchSubmitRequest {
  cookies: string[];
  batch_size?: number;
}

export interface BatchSubmitResult {
  success: number;
  failed: number;
  total: number;
  errors: string[];
}

export interface AuthLoginResponse {
  token: string;
  expires_at: string;
}

// 配置相关 ------------------------------------------------------

export interface BanConfig {
  concurrency: number;
  pause_seconds: number;
  prompts_dir: string;
  models: string[];
  max_tokens: number;
  request_timeout: number;
}

export interface ServerConfig {
  ip: string;
  port: number;
}

export interface NetworkConfig {
  proxy: string | null;
  admin_password: string;
}

export interface ConfigResponse {
  ban_config: BanConfig;
  server_config: ServerConfig;
  network_config: NetworkConfig;
}

export interface ConfigValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  details?: Array<{
    section: string;
    field: string;
    message: string;
    level: "error" | "warning";
  }>;
}

export interface ConfigExport {
  ban: BanConfig;
  server: ServerConfig;
  network: NetworkConfig;
  exported_at: string;
  version: string;
}

export interface ConfigTemplate {
  name: string;
  description: string;
  config: Partial<BanConfig>;
}

export interface ConfigTemplates {
  aggressive: {
    name: string;
    description: string;
    config: Partial<BanConfig>;
  };
  stealth: {
    name: string;
    description: string;
    config: Partial<BanConfig>;
  };
  balanced: {
    name: string;
    description: string;
    config: Partial<BanConfig>;
  };
  [key: string]: {
    name: string;
    description: string;
    config: Partial<BanConfig>;
  };
}

// 统计相关 ------------------------------------------------------

export interface SystemStats {
  total_cookies: number;
  pending_cookies: number;
  banned_cookies: number;
  total_requests: number;
  requests_per_minute: number;
  success_rate: number;
  average_response_time: number;
  workers_active: number;
  uptime_seconds: number;
  last_update: string;
  error_distribution: Record<string, number>;
  performance_metrics: PerformanceMetrics;
}

export interface PerformanceMetrics {
  cpu_usage: number;
  memory_usage: number;
  network_latency: number;
  queue_processing_time: number;
  strategy_effectiveness: number;
}

export interface CookieMetrics {
  cookie_id: string;
  requests_sent: number;
  successful_requests: number;
  failed_requests: number;
  average_response_time: number;
  last_request_time?: string | null;
  consecutive_errors: number;
  adaptive_delay: number;
  status: string;
}

export interface HistoricalStats {
  timestamps: string[];
  request_counts: number[];
  success_rates: number[];
  response_times: number[];
  error_rates: number[];
}

export interface HistoricalStatsParams {
  interval_minutes: number;
  points?: number;
  start_time?: string;
  end_time?: string;
}

// Prompt 管理 ---------------------------------------------------

export interface PromptFile {
  name: string;
  content: string;
  created_at: string;
  modified_at: string;
  size: number;
}

export interface CookieCheckResponse {
  status: "alive" | "banned" | "invalid" | "error";
  alive: boolean;
  banned: boolean;
  lastChecked: string;
  error?: string;
}

// 后端统一错误码（与 backend/api/response.rs 同步）
export type ErrorCode =
  | "AUTH_FAILED"
  | "AUTH_RATE_LIMITED"
  | "INVALID_INPUT"
  | "COOKIE_FORMAT_INVALID"
  | "COOKIE_DUPLICATE"
  | "RATE_LIMITED"
  | "PROMPT_MISSING"
  | "PROMPT_IO_ERROR"
  | "CLAUDE_ERROR"
  | "CLAUDE_RATE_LIMITED"
  | "CLAUDE_BANNED"
  | "DB_ERROR"
  | "CONFIG_INVALID"
  | "CONFIG_SAVE_FAILED"
  | "NOT_FOUND"
  | "INTERNAL";
