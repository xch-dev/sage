// @vitest-environment jsdom

import type { UserBridgeCapability } from '@/bindings';
import { cleanup, render, screen } from '@testing-library/react';
import { PermissionEditor, WalletScopeEditor } from 'sage-app-ui';
import { afterEach, describe, expect, it, vi } from 'vitest';

afterEach(cleanup);

const flags = {
  externallyObservable: false,
  accessesSensitiveSecret: false,
  requestableByApp: true,
  userGrantable: true,
};

const definitions = [
  {
    key: 'wallet.get_coins',
    label: 'Read coins',
    description: 'Lists coins.',
    flags,
  },
  {
    key: 'wallet.get_transactions',
    label: 'Read transactions',
    description: 'Lists transactions.',
    flags,
  },
];

const optional: UserBridgeCapability[] = [
  'wallet.get_coins',
  'wallet.get_transactions',
];

function renderEditor(
  props: Partial<Parameters<typeof PermissionEditor>[0]> = {},
) {
  return render(
    <PermissionEditor
      requestedPermissions={{ capabilities: { required: [], optional } }}
      grantedPermissions={{
        capabilities: ['wallet.get_coins'],
        network: { whitelist: [], whitelistByNetwork: {} },
      }}
      capabilityDefinitions={definitions}
      {...props}
    />,
  );
}

describe('PermissionEditor', () => {
  it('collapses ungranted optional entries by default', () => {
    renderEditor();

    expect(screen.queryByText('Optional permissions')).not.toBeNull();
    expect(screen.queryByText('Read coins')).not.toBeNull();
    expect(screen.queryByText('Read transactions')).toBeNull();
    expect(
      screen.queryByRole('button', { name: 'Show 1 more optional permission' }),
    ).not.toBeNull();
  });

  it('shows every optional entry when not collapsing', () => {
    renderEditor({
      collapseUngrantedOptional: false,
      optionalSectionTitle: 'Capabilities',
    });

    expect(screen.queryByText('Capabilities')).not.toBeNull();
    expect(screen.queryByText('Read transactions')).not.toBeNull();
    expect(screen.queryByRole('button', { name: /Show/ })).toBeNull();
  });
});

describe('WalletScopeEditor', () => {
  const wallets = [{ fingerprint: 7, name: '', emoji: null }];
  const scope = { kind: 'selectedWallets' as const, fingerprints: [7] };

  it('keeps its English defaults', () => {
    render(
      <WalletScopeEditor
        wallets={wallets}
        walletScope={scope}
        onWalletScopeChange={vi.fn()}
      />,
    );

    expect(screen.queryByText('Wallet availability')).not.toBeNull();
    expect(
      screen.queryByText('Choose which wallets can use this app.'),
    ).not.toBeNull();
    expect(screen.queryByText('Wallet 7')).not.toBeNull();
    expect(screen.queryByText('Fingerprint 7')).not.toBeNull();
  });

  it('uses provided labels', () => {
    render(
      <WalletScopeEditor
        wallets={wallets}
        walletScope={scope}
        onWalletScopeChange={vi.fn()}
        labels={{
          heading: 'Wallets',
          description: 'Pick wallets.',
          allWallets: 'Every wallet',
          selectedWallets: 'Only these',
          fingerprint: (f) => `FP ${f}`,
          unnamedWallet: (f) => `Unnamed ${f}`,
        }}
      />,
    );

    expect(screen.queryByText('Wallets')).not.toBeNull();
    expect(screen.queryByText('Pick wallets.')).not.toBeNull();
    expect(
      screen.queryByRole('button', { name: 'Every wallet' }),
    ).not.toBeNull();
    expect(screen.queryByRole('button', { name: 'Only these' })).not.toBeNull();
    expect(screen.queryByText('Unnamed 7')).not.toBeNull();
    expect(screen.queryByText('FP 7')).not.toBeNull();
  });

  it('uses the provided empty text', () => {
    render(
      <WalletScopeEditor
        wallets={[]}
        walletScope={scope}
        onWalletScopeChange={vi.fn()}
        labels={{ noWallets: 'Nothing here.' }}
      />,
    );

    expect(screen.queryByText('Nothing here.')).not.toBeNull();
  });
});
