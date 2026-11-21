// @vitest-environment jsdom

import { TOKEN_SORT_STORAGE_KEY } from '@/hooks/useTokenParams';
import { act, renderHook } from '@testing-library/react';
import { useLocalStorage } from 'usehooks-ts';
import { afterEach, describe, expect, it } from 'vitest';
import {
  resetViewPreferences,
  SIDEBAR_COLLAPSED_STORAGE_KEY,
} from './viewPreferences';

afterEach(() => {
  window.localStorage.clear();
});

describe('resetViewPreferences', () => {
  it('removes the view preferences and leaves other settings alone', () => {
    window.localStorage.setItem(TOKEN_SORT_STORAGE_KEY, '"balance"');
    window.localStorage.setItem(SIDEBAR_COLLAPSED_STORAGE_KEY, 'true');
    window.localStorage.setItem('sage-fiat-currency', '"EUR"');
    window.localStorage.setItem('sage-price-lookups', 'false');

    resetViewPreferences();

    expect(window.localStorage.getItem(TOKEN_SORT_STORAGE_KEY)).toBeNull();
    expect(
      window.localStorage.getItem(SIDEBAR_COLLAPSED_STORAGE_KEY),
    ).toBeNull();
    expect(window.localStorage.getItem('sage-fiat-currency')).toBe('"EUR"');
    expect(window.localStorage.getItem('sage-price-lookups')).toBe('false');
  });

  it('puts mounted preferences back to their defaults', () => {
    const { result } = renderHook(() =>
      useLocalStorage(TOKEN_SORT_STORAGE_KEY, 'name'),
    );

    act(() => result.current[1]('balance'));
    expect(result.current[0]).toBe('balance');

    act(() => resetViewPreferences());
    expect(result.current[0]).toBe('name');
  });
});
