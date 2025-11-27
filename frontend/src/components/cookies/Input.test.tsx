import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import CookieInput from './Input';

const apiMocks = vi.hoisted(() => ({
  submitMultipleCookies: vi.fn(),
}));

vi.mock('../../api', () => ({
  apiClient: apiMocks,
}));

describe('CookieInput component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const validCookie = `sk-ant-${'A'.repeat(86)}`;

  it('提交成功时显示成功提示并回调', async () => {
    apiMocks.submitMultipleCookies.mockResolvedValueOnce({ success: 1, failed: 0 });
    const onSubmit = vi.fn();

    render(<CookieInput onSubmit={onSubmit} />);

    fireEvent.change(screen.getByPlaceholderText(/sk-ant-sid01/), {
      target: { value: validCookie },
    });

    fireEvent.click(screen.getByRole('button', { name: /提交 1 个 Cookie/ }));

    await waitFor(() => {
      expect(screen.getByText(/成功提交 1 个 Cookie/)).toBeInTheDocument();
    });
    expect(onSubmit).toHaveBeenCalled();
  });

  it('提交失败时显示错误信息', async () => {
    apiMocks.submitMultipleCookies.mockRejectedValueOnce(new Error('网络异常'));

    render(<CookieInput />);

    fireEvent.change(screen.getByPlaceholderText(/sk-ant-sid01/), {
      target: { value: validCookie },
    });

    fireEvent.click(screen.getByRole('button', { name: /提交 1 个 Cookie/ }));

    await waitFor(() => {
      expect(screen.getByText('网络异常')).toBeInTheDocument();
    });
  });

  it('部分成功时展示警告信息', async () => {
    apiMocks.submitMultipleCookies.mockResolvedValueOnce({
      success: 1,
      failed: 1,
      errors: ['foo error'],
    });

    render(<CookieInput />);

    fireEvent.change(screen.getByPlaceholderText(/sk-ant-sid01/), {
      target: { value: `${validCookie}\n${validCookie.replace('A', 'B')}` },
    });

    fireEvent.click(screen.getByRole('button', { name: /提交 2 个 Cookie/ }));

    await waitFor(() => {
      expect(screen.getByText(/部分成功/)).toBeInTheDocument();
      expect(screen.getByText(/foo error/)).toBeInTheDocument();
    });
  });
});
