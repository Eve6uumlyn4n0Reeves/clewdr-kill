export interface BanCookie {
  cookie: string;
  submitted_at?: string | null;
  last_used_at?: string | null;
  requests_sent?: number;
  is_banned?: boolean;
  reset_time?: string | null;
}

export type CookieItem = BanCookie;

export interface BanQueueInfo {
  pending: BanCookie[];
  banned: BanCookie[];
  total_requests: number;
}

export interface CookieFormState {
  cookie: string;
  isSubmitting: boolean;
  status: {
    type: "idle" | "success" | "error";
    message: string;
  };
}
