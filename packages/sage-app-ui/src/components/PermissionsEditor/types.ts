import type { UserBridgeCapability } from '@sage-system-app/sdk';

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
      label: string;
      description: string | null;
      required: boolean;
      granted: boolean;
      sensitivityRank: number;
    };

export interface PermissionGroupNode {
  id: string;
  label: string;
  children: PermissionGroupNode[];
  entries: PermissionEntry[];
  sensitivityRank: number;
}
