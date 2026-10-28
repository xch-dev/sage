import { KeyInfo, WalletInfo } from '@/bindings';
import { walletBadge } from '@/lib/walletKind';
import { t } from '@lingui/core/macro';
import { EyeIcon, FlameIcon, SnowflakeIcon, VaultIcon } from 'lucide-react';

export function WalletKindBadge({
  wallet,
  keyInfo,
  iconOnly = false,
}: {
  wallet: WalletInfo;
  /** The keychain entry of a BLS wallet, which decides whether it's hot or cold. */
  keyInfo: KeyInfo | null | undefined;
  iconOnly?: boolean;
}) {
  const badge = walletBadge(wallet, keyInfo);

  const { Icon, label } = {
    hot: { Icon: FlameIcon, label: t`Hot` },
    cold: { Icon: SnowflakeIcon, label: t`Cold` },
    vault: { Icon: VaultIcon, label: t`Vault` },
    watch: { Icon: EyeIcon, label: t`Watch` },
  }[badge];

  if (iconOnly) {
    return (
      <Icon
        className='h-4 w-4 text-muted-foreground'
        aria-label={label}
        role='img'
      >
        <title>{label}</title>
      </Icon>
    );
  }

  return (
    <div className='inline-flex gap-1 items-center rounded-full px-3 py-1.5 text-xs bg-muted'>
      <Icon className='h-4 w-4' aria-hidden='true' />
      <span>{label}</span>
    </div>
  );
}
