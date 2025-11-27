import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import ErrorBoundary from './ErrorBoundary';

const Boom = () => {
  throw new Error('boom');
};

describe('ErrorBoundary', () => {
  it('renders fallback and calls onError when child throws', () => {
    const onError = vi.fn();
    const fallbackText = 'fallback-ui';

    render(
      <ErrorBoundary fallback={<div>{fallbackText}</div>} onError={onError}>
        <Boom />
      </ErrorBoundary>
    );

    expect(screen.getByText(fallbackText)).toBeInTheDocument();
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError.mock.calls[0][0].message).toContain('boom');
  });
});
