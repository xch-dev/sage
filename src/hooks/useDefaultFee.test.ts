// @vitest-environment jsdom

import { cleanup, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { useDefaultFee } from './useDefaultFee';

afterEach(() => {
  cleanup();
  localStorage.clear();
});

function defaultFee() {
  return renderHook(() => useDefaultFee()).result.current.fee;
}

describe('useDefaultFee', () => {
  it('calculates the fee automatically by default', () => {
    expect(defaultFee()).toBe('auto:standard');
  });

  it("doesn't carry over the old default of 0, which is too low to confirm", () => {
    localStorage.setItem('defaultFee', JSON.stringify({ fee: '0' }));
    expect(defaultFee()).toBe('auto:standard');
  });

  it('carries over a fee someone chose before', () => {
    localStorage.setItem('defaultFee', JSON.stringify({ fee: '0.0001' }));
    expect(defaultFee()).toBe('0.0001');
  });

  it('ignores an unreadable old default', () => {
    localStorage.setItem('defaultFee', '{not json');
    expect(defaultFee()).toBe('auto:standard');
  });

  it('keeps a default set since, over the old one', () => {
    localStorage.setItem('defaultFee', JSON.stringify({ fee: '0.0001' }));
    localStorage.setItem(
      'defaultNetworkFee',
      JSON.stringify({ fee: 'auto:7' }),
    );
    expect(defaultFee()).toBe('auto:7');
  });

  it('keeps a tier chosen as the default', () => {
    localStorage.setItem(
      'defaultNetworkFee',
      JSON.stringify({ fee: 'auto:economy' }),
    );
    expect(defaultFee()).toBe('auto:economy');
  });
});
