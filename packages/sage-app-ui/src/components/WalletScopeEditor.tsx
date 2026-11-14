import type { SageAppWalletScope } from 'sage-system-app-sdk';

export type WalletScopeEditorWallet = {
  fingerprint: number;
  name: string;
  emoji?: string | null;
};

export type WalletScopeEditorLabels = {
  heading: string;
  description: string;
  allWallets: string;
  selectedWallets: string;
  noWallets: string;
  fingerprint: (fingerprint: number) => string;
  unnamedWallet: (fingerprint: number) => string;
};

const defaultLabels: WalletScopeEditorLabels = {
  heading: 'Wallet availability',
  description: 'Choose which wallets can use this app.',
  allWallets: 'All wallets',
  selectedWallets: 'Selected wallets only',
  noWallets: 'No wallets found.',
  fingerprint: (fingerprint) => `Fingerprint ${fingerprint}`,
  unnamedWallet: (fingerprint) => `Wallet ${fingerprint}`,
};

interface WalletScopeEditorProps {
  wallets: WalletScopeEditorWallet[];
  walletScope: SageAppWalletScope;
  disabled?: boolean;
  labels?: Partial<WalletScopeEditorLabels>;
  onWalletScopeChange: (scope: SageAppWalletScope) => void;
}

function selectedFingerprints(scope: SageAppWalletScope): number[] {
  return scope.kind === 'selectedWallets' ? scope.fingerprints : [];
}

function walletLabel(
  wallet: WalletScopeEditorWallet,
  labels: WalletScopeEditorLabels,
): string {
  return wallet.name || labels.unnamedWallet(wallet.fingerprint);
}

export function WalletScopeEditor({
  wallets,
  walletScope,
  disabled = false,
  labels: labelOverrides,
  onWalletScopeChange,
}: WalletScopeEditorProps) {
  const labels = { ...defaultLabels, ...labelOverrides };
  const selected = selectedFingerprints(walletScope);

  function toggleFingerprint(fingerprint: number) {
    if (disabled || walletScope.kind !== 'selectedWallets') {
      return;
    }

    const next = selected.includes(fingerprint)
      ? selected.filter((value) => value !== fingerprint)
      : [...selected, fingerprint].sort((a, b) => a - b);

    onWalletScopeChange({
      kind: 'selectedWallets',
      fingerprints: next,
    });
  }

  return (
    <div className='space-y-4'>
      <div>
        <div className='text-sm font-medium'>{labels.heading}</div>
        <p className='mt-1 text-sm text-muted-foreground'>
          {labels.description}
        </p>
      </div>

      <div className='flex gap-2'>
        <button
          type='button'
          disabled={disabled}
          onClick={() => onWalletScopeChange({ kind: 'allWallets' })}
          className={[
            'rounded-lg border px-3 py-2 text-sm font-medium transition-colors disabled:opacity-60',
            walletScope.kind === 'allWallets'
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border hover:bg-muted',
          ].join(' ')}
        >
          {labels.allWallets}
        </button>

        <button
          type='button'
          disabled={disabled}
          onClick={() =>
            onWalletScopeChange({
              kind: 'selectedWallets',
              fingerprints: selected,
            })
          }
          className={[
            'rounded-lg border px-3 py-2 text-sm font-medium transition-colors disabled:opacity-60',
            walletScope.kind === 'selectedWallets'
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border hover:bg-muted',
          ].join(' ')}
        >
          {labels.selectedWallets}
        </button>
      </div>

      {walletScope.kind === 'selectedWallets' ? (
        <div className='overflow-hidden rounded-xl border border-border'>
          {wallets.length === 0 ? (
            <div className='p-3 text-sm text-muted-foreground'>
              {labels.noWallets}
            </div>
          ) : (
            wallets.map((wallet, index) => {
              const checked = selected.includes(wallet.fingerprint);

              return (
                <label
                  key={wallet.fingerprint}
                  className={[
                    'flex cursor-pointer items-center gap-3 p-3 transition-colors hover:bg-muted/70',
                    index > 0 ? 'border-t border-border' : '',
                    disabled ? 'cursor-not-allowed opacity-60' : '',
                  ].join(' ')}
                >
                  <input
                    type='checkbox'
                    disabled={disabled}
                    checked={checked}
                    onChange={() => toggleFingerprint(wallet.fingerprint)}
                    className='h-4 w-4'
                  />

                  <div className='flex items-center justify-center text-2xl leading-none'>
                    {wallet.emoji ?? '👛'}
                  </div>

                  <div className='min-w-0'>
                    <div className='truncate text-sm font-medium'>
                      {walletLabel(wallet, labels)}
                    </div>
                    <div className='text-xs text-muted-foreground'>
                      {labels.fingerprint(wallet.fingerprint)}
                    </div>
                  </div>
                </label>
              );
            })
          )}
        </div>
      ) : null}
    </div>
  );
}
