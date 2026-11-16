import type {
  ConnectionGrant,
  CredentialCapability,
  WalletInfo,
} from '@/bindings';
import { Input } from '@/components/ui/input';
import { useState } from 'react';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';

export function validGrant(grant: ConnectionGrant) {
  return (
    !!grant.name.trim() &&
    grant.name.length <= 80 &&
    grant.capabilities.length > 0 &&
    (grant.walletScope.kind === 'allWallets' ||
      grant.walletScope.fingerprints.length > 0)
  );
}

export function GrantFields({
  grant,
  onChange,
  capabilities,
  wallets,
  requested,
  disabled,
}: {
  grant: ConnectionGrant;
  onChange: (grant: ConnectionGrant) => void;
  capabilities: CredentialCapability[];
  wallets: WalletInfo[];
  requested?: ConnectionGrant;
  disabled?: boolean;
}) {
  const [search, setSearch] = useState('');
  const scope = grant.walletScope;
  const allowedWallets = wallets.filter(
    (w) =>
      !requested ||
      requested.walletScope.kind === 'allWallets' ||
      requested.walletScope.fingerprints.includes(w.fingerprint),
  );
  return (
    <fieldset disabled={disabled} className='space-y-4 disabled:opacity-60'>
      <label className='block space-y-1 text-sm'>
        <span>
          <Trans>Connection name</Trans>
        </span>
        <Input
          value={grant.name}
          maxLength={80}
          onChange={(e) => onChange({ ...grant, name: e.target.value })}
        />
      </label>
      <label className='block space-y-1 text-sm'>
        <span>
          <Trans>Connection type</Trans>
        </span>
        <select
          className='w-full rounded-md border bg-background p-2'
          value={grant.kind}
          disabled={!!requested}
          onChange={(e) =>
            onChange({
              ...grant,
              kind: e.target.value === 'agent' ? 'agent' : 'service',
            })
          }
        >
          <option value='service'>
            <Trans>External service</Trans>
          </option>
          <option value='agent'>
            <Trans>Agent</Trans>
          </option>
        </select>
      </label>
      <fieldset className='space-y-2'>
        <legend className='mb-2 text-sm font-medium'>
          <Trans>Wallet access</Trans>
        </legend>
        {(!requested || requested.walletScope.kind === 'allWallets') && (
          <label className='flex items-start gap-2 text-sm'>
            <input
              type='checkbox'
              checked={scope.kind === 'allWallets'}
              onChange={(e) =>
                onChange({
                  ...grant,
                  walletScope: e.target.checked
                    ? { kind: 'allWallets' }
                    : { kind: 'selectedWallets', fingerprints: [] },
                })
              }
            />
            <Trans>All wallets, including wallets added later</Trans>
          </label>
        )}
        {scope.kind === 'selectedWallets' &&
          allowedWallets.map((wallet) => (
            <label
              key={wallet.fingerprint}
              className='flex items-start gap-2 text-sm'
            >
              <input
                type='checkbox'
                checked={scope.fingerprints.includes(wallet.fingerprint)}
                onChange={(e) =>
                  onChange({
                    ...grant,
                    walletScope: {
                      kind: 'selectedWallets',
                      fingerprints: e.target.checked
                        ? [...scope.fingerprints, wallet.fingerprint]
                        : scope.fingerprints.filter(
                            (fp) => fp !== wallet.fingerprint,
                          ),
                    },
                  })
                }
              />
              {wallet.name} ({wallet.fingerprint})
            </label>
          ))}
        {scope.kind === 'selectedWallets' && !allowedWallets.length && (
          <p className='text-sm text-muted-foreground'>
            <Trans>No eligible wallets available.</Trans>
          </p>
        )}
      </fieldset>
      <fieldset className='space-y-3'>
        <legend className='mb-2 text-sm font-medium'>
          <Trans>RPC permissions</Trans>
        </legend>
        <p className='text-xs text-muted-foreground'>
          <Trans>
            Sensitive operations require confirmation for each request. Wallet
            operations are limited to the selected wallets. Administration
            remains owner-only.
          </Trans>
        </p>
        <Input
          aria-label={t`Search RPC permissions`}
          placeholder={t`Search operations or categories`}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        {capabilities
          .filter(
            (cap) =>
              (!requested || requested.capabilities.includes(cap.key)) &&
              [cap.label, cap.description, cap.category, ...cap.methods].some(
                (text) => text.toLowerCase().includes(search.toLowerCase()),
              ),
          )
          .map((cap) => (
            <label key={cap.key} className='flex items-start gap-2 text-sm'>
              <input
                type='checkbox'
                className='mt-1'
                checked={grant.capabilities.includes(cap.key)}
                onChange={(e) =>
                  onChange({
                    ...grant,
                    capabilities: e.target.checked
                      ? [...grant.capabilities, cap.key]
                      : grant.capabilities.filter((key) => key !== cap.key),
                  })
                }
              />
              <span>
                {cap.label}
                <span className='ml-2 text-xs text-muted-foreground'>
                  {cap.category}
                  {cap.approval
                    ? t` · Confirmation required`
                    : t` · Read access`}
                </span>
                <span className='block text-xs text-muted-foreground'>
                  {cap.description}
                </span>
              </span>
            </label>
          ))}
      </fieldset>
    </fieldset>
  );
}
