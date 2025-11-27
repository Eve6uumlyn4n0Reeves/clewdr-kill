import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import AdvancedConfig from './AdvancedConfig';

const apiMocks = vi.hoisted(() => ({
  getConfig: vi.fn(),
  updateConfig: vi.fn(),
  resetConfig: vi.fn(),
  adminAction: vi.fn(),
}));

vi.mock('../../api', () => ({
  apiClient: apiMocks,
}));

const baseConfig = {
  ban_config: {
    concurrency: 2,
    pause_seconds: 1,
    prompts_dir: './prompts',
    models: ['claude-3-5-haiku-20241022'],
    max_tokens: 1024,
    request_timeout: 5000,
  },
};

describe('AdvancedConfig component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    apiMocks.getConfig.mockResolvedValue(baseConfig);
    apiMocks.updateConfig.mockResolvedValue({});
    apiMocks.resetConfig.mockResolvedValue(baseConfig);
  });

  it('保存成功时提示成功信息', async () => {
    render(<AdvancedConfig />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('2')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }));

    await waitFor(() => {
      expect(screen.getByText('配置保存成功')).toBeInTheDocument();
    });
    expect(apiMocks.updateConfig).toHaveBeenCalled();
  });

  it('保存失败时展示错误信息', async () => {
    apiMocks.updateConfig.mockRejectedValueOnce(new Error('更新失败'));

    render(<AdvancedConfig />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('2')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }));

    await waitFor(() => {
      expect(screen.getByText('更新失败')).toBeInTheDocument();
    });
  });

  it('重置失败时展示错误信息', async () => {
    apiMocks.resetConfig.mockRejectedValueOnce(new Error('重置失败'));

    render(<AdvancedConfig />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('2')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: '重置为默认配置' }));

    await waitFor(() => {
      expect(screen.getByText('重置失败')).toBeInTheDocument();
    });
  });
});
