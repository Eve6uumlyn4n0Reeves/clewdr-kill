import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import React from "react";

vi.mock("../../api/stats", () => ({
  useSystemStats: () => ({
    stats: {
      total_requests: 10,
      pending_cookies: 2,
      banned_cookies: 1,
      requests_per_minute: 5,
      success_rate: 80,
      average_response_time: 120,
      workers_active: 3,
      uptime_seconds: 90,
      last_update: new Date().toISOString(),
      error_distribution: {},
      performance_metrics: {
        cpu_usage: 10,
        memory_usage: 1024,
        network_latency: 100,
        queue_processing_time: 10,
        strategy_effectiveness: 80,
      },
    },
    loading: false,
    error: null,
    refetch: vi.fn(),
  }),
}));

import { SystemStats } from "./SystemStats";

describe("SystemStats component", () => {
  it("renders stats values", () => {
    render(<SystemStats />);
    expect(screen.getByText("总请求")).toBeInTheDocument();
    expect(screen.getByText("10")).toBeInTheDocument();
    expect(screen.getByText("成功率")).toBeInTheDocument();
    expect(screen.getByText("80.0%")).toBeInTheDocument();
  });
});
