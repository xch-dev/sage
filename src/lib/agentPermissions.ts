import type {
  SageAgent,
  SageAppCapabilityDefinitionView,
  SageAppWalletScope,
  UserBridgeCapability,
} from '@/bindings';
import type {
  PermissionEditorProps,
  RequestedPermissionsView,
} from 'sage-app-ui';

type GrantedPermissions = PermissionEditorProps['grantedPermissions'];

function definitionKeys(
  definitions: SageAppCapabilityDefinitionView[],
): UserBridgeCapability[] {
  return definitions.map(
    (definition) => definition.key as UserBridgeCapability,
  );
}

export function agentRequestedPermissions(
  definitions: SageAppCapabilityDefinitionView[],
): RequestedPermissionsView {
  return {
    capabilities: { required: [], optional: definitionKeys(definitions) },
  };
}

export function agentGrantedPermissions(
  capabilities: UserBridgeCapability[],
): GrantedPermissions {
  return {
    capabilities,
    network: { whitelist: [], whitelistByNetwork: {} },
  };
}

export function initialAgentCapabilities(
  agent: SageAgent | null,
  definitions: SageAppCapabilityDefinitionView[],
): UserBridgeCapability[] {
  return agent ? agent.capabilities : definitionKeys(definitions);
}

export function defaultAgentWalletScope(
  currentFingerprint?: number,
): SageAppWalletScope {
  return {
    kind: 'selectedWallets',
    fingerprints: currentFingerprint === undefined ? [] : [currentFingerprint],
  };
}

export function hasSelectableWallet(
  scope: SageAppWalletScope,
  wallets: { fingerprint: number }[],
): boolean {
  if (scope.kind === 'allWallets') return true;
  const known = new Set(wallets.map((wallet) => wallet.fingerprint));
  return scope.fingerprints.some((fingerprint) => known.has(fingerprint));
}

export function hasUnsavedAgentChanges(
  agent: SageAgent,
  name: string,
  editedCapabilities: UserBridgeCapability[] | null,
  walletScope: SageAppWalletScope,
): boolean {
  return (
    name.trim() !== agent.name ||
    editedCapabilities !== null ||
    walletScope !== agent.walletScope
  );
}

export const DEFAULT_AGENT_PORT = 9258;
export const MIN_AGENT_PORT = 1024;
export const MAX_AGENT_PORT = 65535;

// Ports below 1024 need elevated privileges on most systems, so they're not offered.
export function isValidAgentPort(value: string): boolean {
  if (!/^\d+$/.test(value)) return false;
  const port = Number(value);
  return port >= MIN_AGENT_PORT && port <= MAX_AGENT_PORT;
}

export interface McpClientSnippets {
  claudeCode: string;
  json: string;
}

// The setup snippets shown with a new token, for Claude Code and for other MCP clients.
export function mcpClientSnippets(
  token: string,
  port: number,
): McpClientSnippets {
  const url = `http://127.0.0.1:${port}/mcp`;

  return {
    claudeCode: `claude mcp add --transport http sage ${url} --header "Authorization: Bearer ${token}"`,
    json: JSON.stringify({
      mcpServers: {
        sage: {
          type: 'http',
          url,
          headers: { Authorization: `Bearer ${token}` },
        },
      },
    }),
  };
}
