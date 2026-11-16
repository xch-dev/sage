// @vitest-environment jsdom

import type { AgentCapabilityView, Capability } from '@/bindings';
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { AgentPermissionsPanel } from './AgentPermissionsPanel';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(cleanup);

const definitions: AgentCapabilityView[] = [
  {
    key: 'wallet.get_coins',
    label: 'Read coins',
    description: 'Lists coins.',
  },
  {
    key: 'wallet.get_transactions',
    label: 'Read transactions',
    description: 'Lists transactions.',
  },
];

function renderPanel(onCapabilitiesChange = vi.fn()) {
  render(
    <I18nProvider i18n={i18n}>
      <AgentPermissionsPanel
        definitions={definitions}
        capabilities={['wallet.get_coins'] as Capability[]}
        onCapabilitiesChange={onCapabilitiesChange}
        walletScope={{ kind: 'selectedWallets', fingerprints: [7] }}
        onWalletScopeChange={vi.fn()}
        wallets={[{ fingerprint: 7, name: 'Main', emoji: null }]}
      />
    </I18nProvider>,
  );
  return onCapabilitiesChange;
}

describe('AgentPermissionsPanel', () => {
  it('shows every capability, granted or not', () => {
    renderPanel();

    expect(screen.queryByText('Read coins')).not.toBeNull();
    expect(screen.queryByText('Read transactions')).not.toBeNull();
    expect(screen.queryByRole('button', { name: /Show/ })).toBeNull();
  });

  it('reports capability changes', () => {
    const onCapabilitiesChange = renderPanel();

    fireEvent.click(screen.getByRole('checkbox', { name: /Read coins/ }));

    expect(onCapabilitiesChange).toHaveBeenCalledWith([]);
  });

  it('shows the wallet editor on the Wallets tab', () => {
    renderPanel();

    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Wallets' }));

    expect(
      screen.queryByText('Choose which wallets this agent can use.'),
    ).not.toBeNull();
    expect(screen.queryByText('Main')).not.toBeNull();
  });
});
