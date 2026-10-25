import type { UserBridgeCapability } from 'sage-system-app-sdk';

export type NetworkPermissionScheme = 'http' | 'https' | 'wss';

export interface NetworkPermissionSchemeState {
  scheme: NetworkPermissionScheme;
  key: string;
  required: boolean;
  granted: boolean;
  disabled: boolean;
  visible: boolean;
}

export type PermissionEntry =
  | {
      id: string;
      kind: 'capability';
      key: string;
      capability: UserBridgeCapability;
      label: string;
      description: string | null;
      required: boolean;
      granted: boolean;
      sensitivityRank: number;
    }
  | {
      id: string;
      kind: 'network';
      key: string;
      host: string;
      networkId: string | null;
      label: string;
      description: string | null;
      required: boolean;
      granted: boolean;
      sensitivityRank: number;
      schemes: Record<NetworkPermissionScheme, NetworkPermissionSchemeState>;
    };

export interface PermissionGroupNode {
  id: string;
  label: string;
  children: PermissionGroupNode[];
  entries: PermissionEntry[];
  sensitivityRank: number;
}
