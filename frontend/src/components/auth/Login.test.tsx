import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import Login from './Login';

const loginMock = vi.fn();
const setIsAuthenticatedMock = vi.fn();

vi.mock('../../hooks/useAuth', () => ({
  useAuth: () => ({
    login: loginMock,
  }),
}));

vi.mock('../../context/AppContext', () => ({
  useAppContext: () => ({
    isAuthenticated: false,
    setIsAuthenticated: setIsAuthenticatedMock,
  }),
}));

describe('Login component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('登录成功时调用 onAuthenticated', async () => {
    loginMock.mockResolvedValueOnce(true);
    const onAuthenticated = vi.fn();

    render(<Login onAuthenticated={onAuthenticated} />);

    fireEvent.change(screen.getByPlaceholderText('请输入密码'), {
      target: { value: 'test-pass' },
    });
    fireEvent.click(screen.getByRole('button', { name: '登录' }));

    await waitFor(() => {
      expect(onAuthenticated).toHaveBeenCalledWith(true);
    });
    expect(setIsAuthenticatedMock).toHaveBeenCalledWith(true);
    expect(screen.queryByText(/密码错误/)).toBeNull();
  });

  it('登录失败时展示错误信息', async () => {
    loginMock.mockRejectedValueOnce(new Error('服务异常'));
    const onAuthenticated = vi.fn();

    render(<Login onAuthenticated={onAuthenticated} />);

    fireEvent.change(screen.getByPlaceholderText('请输入密码'), {
      target: { value: 'bad-pass' },
    });
    fireEvent.click(screen.getByRole('button', { name: '登录' }));

    await waitFor(() => {
      expect(screen.getByText('服务异常')).toBeInTheDocument();
    });
    expect(onAuthenticated).not.toHaveBeenCalledWith(true);
    expect(setIsAuthenticatedMock).toHaveBeenCalledWith(false);
  });
});
