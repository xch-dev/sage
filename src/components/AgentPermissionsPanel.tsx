import type {
  SageAppCapabilityDefinitionView,
  SageAppWalletScope,
  UserBridgeCapability,
} from '@/bindings';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  agentGrantedPermissions,
  agentRequestedPermissions,
} from '@/lib/agentPermissions';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useMemo } from 'react';
import {
  PermissionEditor,
  WalletScopeEditor,
  type WalletScopeEditorWallet,
} from 'sage-app-ui';

interface AgentPermissionsPanelProps {
  definitions: SageAppCapabilityDefinitionView[];
  capabilities: UserBridgeCapability[];
  onCapabilitiesChange: (capabilities: UserBridgeCapability[]) => void;
  walletScope: SageAppWalletScope;
  onWalletScopeChange: (scope: SageAppWalletScope) => void;
  wallets: WalletScopeEditorWallet[];
  disabled?: boolean;
}

export function AgentPermissionsPanel({
  definitions,
  capabilities,
  onCapabilitiesChange,
  walletScope,
  onWalletScopeChange,
  wallets,
  disabled = false,
}: AgentPermissionsPanelProps) {
  const requested = useMemo(
    () => agentRequestedPermissions(definitions),
    [definitions],
  );
  const granted = useMemo(
    () => agentGrantedPermissions(capabilities),
    [capabilities],
  );

  return (
    <Tabs defaultValue='permissions'>
      <TabsList>
        <TabsTrigger value='permissions'>
          <Trans>Permissions</Trans>
        </TabsTrigger>
        <TabsTrigger value='wallets'>
          <Trans>Wallets</Trans>
        </TabsTrigger>
      </TabsList>

      <TabsContent value='permissions' className='pt-2'>
        <PermissionEditor
          requestedPermissions={requested}
          grantedPermissions={granted}
          capabilityDefinitions={definitions}
          onGrantedPermissionsChange={(next) =>
            onCapabilitiesChange(next.capabilities)
          }
          editable={!disabled}
          collapseUngrantedOptional={false}
          optionalSectionTitle={t`Capabilities`}
          emptyText={t`No capabilities are available.`}
        />
      </TabsContent>

      <TabsContent value='wallets' className='pt-2'>
        <WalletScopeEditor
          wallets={wallets}
          walletScope={walletScope}
          disabled={disabled}
          onWalletScopeChange={onWalletScopeChange}
          labels={{
            heading: t`Wallet access`,
            description: t`Choose which wallets this agent can use.`,
            allWallets: t`All wallets`,
            selectedWallets: t`Selected wallets only`,
            noWallets: t`No wallets found.`,
            fingerprint: (fingerprint) => t`Fingerprint ${fingerprint}`,
            unnamedWallet: (fingerprint) => t`Wallet ${fingerprint}`,
          }}
        />
      </TabsContent>
    </Tabs>
  );
}
