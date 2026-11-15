// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { OfferChoice, OfferSettingCard } from './OfferSettings';

afterEach(cleanup);

const options = [
  {
    value: 'a',
    label: 'Option A',
    description: 'About A',
    children: <span>details-a</span>,
  },
  { value: 'b', label: 'Option B', children: <span>details-b</span> },
];

function radio(name: string) {
  return screen.getByRole('radio', { name }) as HTMLInputElement;
}

describe('OfferChoice', () => {
  it('names the group and each option, and describes it', () => {
    render(
      <OfferChoice
        label='Choice'
        value='a'
        onValueChange={vi.fn()}
        options={options}
      />,
    );

    expect(screen.getByRole('group', { name: 'Choice' })).toBeTruthy();
    expect(radio('Option A').getAttribute('aria-describedby')).toBeTruthy();
    expect(radio('Option B').getAttribute('aria-describedby')).toBeNull();
  });

  it('shows details only for the selected option', () => {
    render(
      <OfferChoice
        label='Choice'
        value='a'
        onValueChange={vi.fn()}
        options={options}
      />,
    );

    expect(radio('Option A').checked).toBe(true);
    expect(radio('Option B').checked).toBe(false);
    expect(screen.queryByText('details-a')).not.toBeNull();
    expect(screen.queryByText('details-b')).toBeNull();
  });

  it('reports the option that was picked', () => {
    const onValueChange = vi.fn();
    render(
      <OfferChoice
        label='Choice'
        value='a'
        onValueChange={onValueChange}
        options={options}
      />,
    );

    fireEvent.click(screen.getByText('Option B'));
    expect(onValueChange).toHaveBeenCalledWith('b');
  });

  it('disables every option while disabled', () => {
    render(
      <OfferChoice
        label='Choice'
        value='a'
        disabled
        onValueChange={vi.fn()}
        options={options}
      />,
    );

    // jsdom still fires change events on disabled radios, so check the
    // property rather than clicking.
    expect(radio('Option A').disabled).toBe(true);
    expect(radio('Option B').disabled).toBe(true);
    expect(radio('Option A').checked).toBe(true);
  });
});

describe('OfferSettingCard', () => {
  it('shows its content only while the toggle is on', () => {
    const onCheckedChange = vi.fn();
    const { rerender } = render(
      <OfferSettingCard
        title='Setting'
        description='About it'
        toggle={{ checked: false, onCheckedChange }}
      >
        <span>content</span>
      </OfferSettingCard>,
    );

    const toggle = screen.getByRole('switch', { name: 'Setting' });
    expect(toggle.getAttribute('aria-describedby')).toBeTruthy();
    expect(screen.queryByText('content')).toBeNull();

    fireEvent.click(toggle);
    expect(onCheckedChange).toHaveBeenCalledWith(true);

    rerender(
      <OfferSettingCard
        title='Setting'
        toggle={{ checked: true, onCheckedChange }}
      >
        <span>content</span>
      </OfferSettingCard>,
    );
    expect(screen.queryByText('content')).not.toBeNull();
  });

  it('disables the toggle', () => {
    render(
      <OfferSettingCard
        title='Setting'
        toggle={{ checked: false, onCheckedChange: vi.fn(), disabled: true }}
      >
        <span>content</span>
      </OfferSettingCard>,
    );

    expect(
      (screen.getByRole('switch', { name: 'Setting' }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });
});
