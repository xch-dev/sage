import type {
  Agent,
  AgentCapabilityView,
  Capability,
  WalletScope,
} from '@/bindings';
import type {
  PermissionEditorProps,
  RequestedPermissionsView,
} from 'sage-app-ui';

type GrantedPermissions = PermissionEditorProps['grantedPermissions'];
type EditorDefinitions = PermissionEditorProps['capabilityDefinitions'];
// The shared editor is typed with the app bridge's capability keys; the casts between it and
// `Capability` stay in this file.
type EditorCapability = GrantedPermissions['capabilities'][number];

function definitionKeys(definitions: AgentCapabilityView[]): Capability[] {
  return definitions.map((definition) => definition.key as Capability);
}

export function toEditorDefinitions(
  views: AgentCapabilityView[],
): EditorDefinitions {
  return views.map((view) => ({
    key: view.key as EditorCapability,
    label: view.label,
    description: view.description,
    flags: {
      externallyObservable: false,
      accessesSensitiveSecret: false,
      requestableByApp: true,
      userGrantable: true,
    },
  }));
}

export function editorCapabilitiesToAgent(
  capabilities: EditorCapability[],
): Capability[] {
  return capabilities as string[] as Capability[];
}

export function agentRequestedPermissions(
  definitions: AgentCapabilityView[],
): RequestedPermissionsView {
  return {
    capabilities: {
      required: [],
      optional: definitionKeys(definitions) as string[] as EditorCapability[],
    },
  };
}

export function agentGrantedPermissions(
  capabilities: Capability[],
): GrantedPermissions {
  return {
    capabilities: capabilities as string[] as EditorCapability[],
    network: { whitelist: [], whitelistByNetwork: {} },
  };
}

export function initialAgentCapabilities(
  agent: Agent | null,
  definitions: AgentCapabilityView[],
): Capability[] {
  return agent ? agent.capabilities : definitionKeys(definitions);
}

export function defaultAgentWalletScope(
  currentFingerprint?: number,
): WalletScope {
  return {
    kind: 'selectedWallets',
    fingerprints: currentFingerprint === undefined ? [] : [currentFingerprint],
  };
}

export function hasSelectableWallet(
  scope: WalletScope,
  wallets: { fingerprint: number }[],
): boolean {
  if (scope.kind === 'allWallets') return true;
  const known = new Set(wallets.map((wallet) => wallet.fingerprint));
  return scope.fingerprints.some((fingerprint) => known.has(fingerprint));
}

export function hasUnsavedAgentChanges(
  agent: Agent,
  name: string,
  editedCapabilities: Capability[] | null,
  walletScope: WalletScope,
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

// A failed agent load is only worth a toast when the server isn't already showing why it failed.
export function shouldReportAgentLoadError(
  status: { startError: string | null } | null,
): boolean {
  return !status?.startError;
}

// The server only reports a start error while disabled when the agents database couldn't be opened.
export function isAgentStoreUnavailable(
  status: { enabled: boolean; startError: string | null } | null,
): boolean {
  return !!status?.startError && !status.enabled;
}
