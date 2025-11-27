import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useDebounce } from './useDebounce';

describe('useDebounce', () => {
  it('延迟更新值', () => {
    vi.useFakeTimers();
    const { result, rerender } = renderHook(({ value }) => useDebounce(value, 50), {
      initialProps: { value: 'a' },
    });

    expect(result.current).toBe('a');

    rerender({ value: 'b' });
    expect(result.current).toBe('a'); // 立即不变

    act(() => {
      vi.advanceTimersByTime(60);
    });

    expect(result.current).toBe('b');
    vi.useRealTimers();
  });
});
