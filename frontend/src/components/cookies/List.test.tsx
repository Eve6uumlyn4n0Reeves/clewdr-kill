import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react';
import CookieList from './List';

const sample = {
  pending: [
    {
      cookie: 'sk-ant-AAA',
      is_banned: false,
      requests_sent: 1,
      last_used_at: null,
      submitted_at: '2025-01-01T00:00:00Z',
      error_message: null,
    },
  ],
  processing: [],
  banned: [
    {
      cookie: 'sk-ant-BBB',
      is_banned: true,
      requests_sent: 2,
      last_used_at: null,
      submitted_at: '2025-01-02T00:00:00Z',
      error_message: 'banned',
    },
  ],
  total_requests: 3,
};

const mocks = vi.hoisted(() => ({
  getCookies: vi.fn(),
  checkCookieStatus: vi.fn(),
  deleteCookie: vi.fn(),
}));

vi.mock('../../api', () => ({
  apiClient: {
    getCookies: (...args: unknown[]) => mocks.getCookies(...(args as [])),
    checkCookieStatus: (...args: unknown[]) => mocks.checkCookieStatus(...(args as [])),
    deleteCookie: (...args: unknown[]) => mocks.deleteCookie(...(args as [])),
  },
}));

vi.mock('../../hooks/useAsyncOperation', () => ({
  useAsyncOperation: () => ({
    execute: async (fn: any) => await fn(),
  }),
}));

describe('CookieList', () => {
  beforeEach(() => {
    mocks.getCookies.mockResolvedValue(sample);
    mocks.checkCookieStatus.mockResolvedValue({ status: 'alive' });
    mocks.deleteCookie.mockResolvedValue(undefined);
  });

  it('加载态显示骨架屏', () => {
    mocks.getCookies.mockReturnValue(new Promise(() => {})); // 永不 resolve
    render(<CookieList />);

    expect(screen.getAllByText((_, node) => node?.classList.contains('animate-pulse')).length).toBeGreaterThan(0);
  });
});
