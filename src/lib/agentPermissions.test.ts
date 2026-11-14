import type {
  SageAgent,
  SageAppCapabilityDefinitionView,
  UserBridgeCapability,
} from '@/bindings';
import { describe, expect, it } from 'vitest';
import {
  agentGrantedPermissions,
  agentRequestedPermissions,
  defaultAgentWalletScope,
  hasSelectableWallet,
  initialAgentCapabilities,
} from './agentPermissions';

const flags = {
  externallyObservable: false,
  accessesSensitiveSecret: false,
  requestableByApp: true,
  userGrantable: true,
};

const defA: SageAppCapabilityDefinitionView = {
  key: 'wallet.get_coins',
  label: 'Read coins',
  description: 'Lists coins.',
  flags,
};

const defB: SageAppCapabilityDefinitionView = {
  key: 'wallet.get_transactions',
  label: 'Read transactions',
  description: 'Lists transactions.',
  flags,
};

function agentWith(capabilities: UserBridgeCapability[]): SageAgent {
  return {
    id: 'a',
    name: 'agent',
    capabilities,
    walletScope: { kind: 'allWallets' },
    createdAtMs: 0,
    updatedAtMs: 0,
    lastUsedAtMs: null,
  };
}

describe('agentRequestedPermissions', () => {
  it('requests every definition as optional', () => {
    expect(agentRequestedPermissions([defA, defB])).toEqual({
      capabilities: { required: [], optional: [defA.key, defB.key] },
    });
  });
});

describe('agentGrantedPermissions', () => {
  it('grants the capabilities with no network access', () => {
    expect(agentGrantedPermissions(['wallet.get_coins'])).toEqual({
      capabilities: ['wallet.get_coins'],
      network: { whitelist: [], whitelistByNetwork: {} },
    });
  });
});

describe('initialAgentCapabilities', () => {
  it('grants every definition to a new agent', () => {
    expect(initialAgentCapabilities(null, [defA, defB])).toEqual([
      defA.key,
      defB.key,
    ]);
  });

  it("keeps an existing agent's capabilities", () => {
    expect(
      initialAgentCapabilities(agentWith(['wallet.get_coins']), [defA, defB]),
    ).toEqual(['wallet.get_coins']);
  });
});

describe('defaultAgentWalletScope', () => {
  it('selects the current wallet', () => {
    expect(defaultAgentWalletScope(7)).toEqual({
      kind: 'selectedWallets',
      fingerprints: [7],
    });
  });

  it('selects nothing without a current wallet', () => {
    expect(defaultAgentWalletScope()).toEqual({
      kind: 'selectedWallets',
      fingerprints: [],
    });
  });
});

describe('hasSelectableWallet', () => {
  const wallets = [{ fingerprint: 7 }];

  it('accepts all wallets', () => {
    expect(hasSelectableWallet({ kind: 'allWallets' }, [])).toBe(true);
  });

  it('rejects a scope with only unknown wallets', () => {
    expect(
      hasSelectableWallet(
        { kind: 'selectedWallets', fingerprints: [9] },
        wallets,
      ),
    ).toBe(false);
  });

  it('accepts a scope with a known wallet', () => {
    expect(
      hasSelectableWallet(
        { kind: 'selectedWallets', fingerprints: [7] },
        wallets,
      ),
    ).toBe(true);
  });

  it('rejects an empty selection', () => {
    expect(
      hasSelectableWallet(
        { kind: 'selectedWallets', fingerprints: [] },
        wallets,
      ),
    ).toBe(false);
  });
});
