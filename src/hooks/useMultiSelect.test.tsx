// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { useMultiSelect } from './useMultiSelect';

describe('useMultiSelect', () => {
  it('returns null selection state while multi-select is off', () => {
    const { result } = renderHook(() => useMultiSelect());
    expect(result.current.selectionStateFor('a')).toBeNull();
  });

  it('toggles ids and exposes selection state', () => {
    const { result } = renderHook(() => useMultiSelect());
    act(() => result.current.setMultiSelect(true));
    act(() => result.current.selectionStateFor('a')?.[1](true));
    expect(result.current.selected).toEqual(['a']);
    expect(result.current.selectionStateFor('a')?.[0]).toBe(true);
    act(() => result.current.toggle('a', false));
    expect(result.current.selected).toEqual([]);
  });

  it('does not duplicate an id selected twice', () => {
    const { result } = renderHook(() => useMultiSelect());
    act(() => result.current.setMultiSelect(true));
    act(() => result.current.toggle('a', true));
    act(() => result.current.toggle('a', true));
    expect(result.current.selected).toEqual(['a']);
  });

  it('select all replaces the selection', () => {
    const { result } = renderHook(() => useMultiSelect());
    act(() => result.current.setMultiSelect(true));
    act(() => result.current.toggle('old', true));
    act(() => result.current.selectAll(['a', 'b']));
    expect(result.current.selected).toEqual(['a', 'b']);
  });

  it('toggling multi-select clears the selection', () => {
    const { result } = renderHook(() => useMultiSelect());
    act(() => result.current.setMultiSelect(true));
    act(() => result.current.toggle('a', true));
    act(() => result.current.setMultiSelect(false));
    expect(result.current.selected).toEqual([]);
    expect(result.current.multiSelect).toBe(false);
  });

  it('resets when reset deps change but not on unrelated rerenders', () => {
    const { result, rerender } = renderHook(
      ({ dep }) => useMultiSelect([dep]),
      { initialProps: { dep: 'x' } },
    );
    act(() => result.current.setMultiSelect(true));
    act(() => result.current.toggle('a', true));
    rerender({ dep: 'x' });
    expect(result.current.selected).toEqual(['a']);
    rerender({ dep: 'y' });
    expect(result.current.selected).toEqual([]);
    expect(result.current.multiSelect).toBe(false);
  });
});
