// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from '@testing-library/react';
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import { DebouncedSearchInput } from './DebouncedSearchInput';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

function input() {
  return screen.getByPlaceholderText('Search');
}

describe('DebouncedSearchInput', () => {
  it('reports the value once after the debounce delay', () => {
    const onChange = vi.fn();
    render(
      <DebouncedSearchInput
        value={null}
        onChange={onChange}
        placeholder='Search'
      />,
    );

    fireEvent.change(input(), { target: { value: 'sp' } });
    fireEvent.change(input(), { target: { value: 'space' } });
    expect(onChange).not.toHaveBeenCalled();

    act(() => vi.advanceTimersByTime(400));
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith('space');
  });

  it('reports null for whitespace and after clearing', () => {
    const onChange = vi.fn();
    const { rerender } = render(
      <DebouncedSearchInput
        value={null}
        onChange={onChange}
        placeholder='Search'
      />,
    );

    fireEvent.change(input(), { target: { value: 'space' } });
    act(() => vi.advanceTimersByTime(400));
    rerender(
      <DebouncedSearchInput
        value='space'
        onChange={onChange}
        placeholder='Search'
      />,
    );

    fireEvent.click(screen.getByLabelText('Clear search'));
    act(() => vi.advanceTimersByTime(400));
    expect(onChange).toHaveBeenLastCalledWith(null);

    rerender(
      <DebouncedSearchInput
        value={null}
        onChange={onChange}
        placeholder='Search'
      />,
    );
    fireEvent.change(input(), { target: { value: '   ' } });
    act(() => vi.advanceTimersByTime(400));
    expect(onChange).toHaveBeenCalledTimes(2);
  });

  it('external clear during pending debounce does not resurrect the old query', () => {
    const onChange = vi.fn();
    const { rerender } = render(
      <DebouncedSearchInput
        value='space'
        onChange={onChange}
        placeholder='Search'
      />,
    );

    fireEvent.change(input(), { target: { value: 'spacex' } });
    // Parent clears filters before the debounce fires.
    rerender(
      <DebouncedSearchInput
        value={null}
        onChange={onChange}
        placeholder='Search'
      />,
    );
    act(() => vi.advanceTimersByTime(1000));

    expect(input()).toHaveProperty('value', '');
    expect(onChange).not.toHaveBeenCalledWith('spacex');
    expect(onChange).not.toHaveBeenCalledWith('space');
  });
});
