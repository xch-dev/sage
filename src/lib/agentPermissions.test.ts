import type { Agent, AgentCapabilityView, Capability } from '@/bindings';
import { describe, expect, it } from 'vitest';
import {
  agentGrantedPermissions,
  agentRequestedPermissions,
  defaultAgentWalletScope,
  hasSelectableWallet,
  hasUnsavedAgentChanges,
  initialAgentCapabilities,
  isAgentStoreUnavailable,
  isValidAgentPort,
  mcpClientSnippets,
  shouldReportAgentLoadError,
  toEditorDefinitions,
} from './agentPermissions';

const defA: AgentCapabilityView = {
  key: 'wallet.get_coins',
  label: 'Read coins',
  description: 'Lists coins.',
};

const defB: AgentCapabilityView = {
  key: 'wallet.get_transactions',
  label: 'Read transactions',
  description: 'Lists transactions.',
};

function agentWith(capabilities: Capability[]): Agent {
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

describe('toEditorDefinitions', () => {
  it('adapts agent capabilities to editor definitions the user can grant', () => {
    const [d] = toEditorDefinitions([
      { key: 'wallet.get_coins', label: 'L', description: 'D' },
    ]);
    expect(d).toEqual({
      key: 'wallet.get_coins',
      label: 'L',
      description: 'D',
      flags: {
        externallyObservable: false,
        accessesSensitiveSecret: false,
        requestableByApp: true,
        userGrantable: true,
      },
    });
  });
});

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

describe('hasUnsavedAgentChanges', () => {
  const agent = agentWith(['wallet.get_coins']);

  it('is false for the untouched form', () => {
    expect(
      hasUnsavedAgentChanges(agent, 'agent', null, agent.walletScope),
    ).toBe(false);
  });

  it('detects a renamed agent', () => {
    expect(
      hasUnsavedAgentChanges(agent, 'renamed', null, agent.walletScope),
    ).toBe(true);
  });

  it('detects edited capabilities', () => {
    expect(hasUnsavedAgentChanges(agent, 'agent', [], agent.walletScope)).toBe(
      true,
    );
  });

  it('detects a changed wallet scope', () => {
    expect(
      hasUnsavedAgentChanges(agent, 'agent', null, {
        kind: 'selectedWallets',
        fingerprints: [7],
      }),
    ).toBe(true);
  });
});

describe('isValidAgentPort', () => {
  it('accepts ports from 1024 to 65535', () => {
    expect(isValidAgentPort('1024')).toBe(true);
    expect(isValidAgentPort('9258')).toBe(true);
    expect(isValidAgentPort('65535')).toBe(true);
  });

  it('rejects privileged, out of range and non-numeric ports', () => {
    for (const value of ['', '0', '1023', '65536', '12a', '-1', '9258.5']) {
      expect(isValidAgentPort(value), value).toBe(false);
    }
  });
});

describe('mcpClientSnippets', () => {
  const token = `sage_agent_${'ab'.repeat(32)}`;

  it('builds the Claude Code command for the port and token', () => {
    expect(mcpClientSnippets(token, 9300).claudeCode).toBe(
      `claude mcp add --transport http sage http://127.0.0.1:9300/mcp --header "Authorization: Bearer ${token}"`,
    );
  });

  it('builds a JSON client config for the port and token', () => {
    expect(JSON.parse(mcpClientSnippets(token, 9300).json)).toEqual({
      mcpServers: {
        sage: {
          type: 'http',
          url: 'http://127.0.0.1:9300/mcp',
          headers: { Authorization: `Bearer ${token}` },
        },
      },
    });
  });
});

describe('shouldReportAgentLoadError', () => {
  it('stays quiet while the status explains the failure', () => {
    expect(shouldReportAgentLoadError({ startError: 'db locked' })).toBe(false);
  });

  it('reports failures otherwise', () => {
    expect(shouldReportAgentLoadError({ startError: null })).toBe(true);
    expect(shouldReportAgentLoadError(null)).toBe(true);
  });
});

describe('isAgentStoreUnavailable', () => {
  it('is true for a start error while disabled', () => {
    expect(
      isAgentStoreUnavailable({ enabled: false, startError: 'disk full' }),
    ).toBe(true);
  });

  it('is false otherwise', () => {
    expect(
      isAgentStoreUnavailable({ enabled: true, startError: 'address in use' }),
    ).toBe(false);
    expect(isAgentStoreUnavailable({ enabled: false, startError: null })).toBe(
      false,
    );
    expect(isAgentStoreUnavailable(null)).toBe(false);
  });
});
