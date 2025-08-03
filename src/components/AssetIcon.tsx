import { AssetKind } from '@/bindings';
import { Coins, FilePenLine, Image, UndoDot, User } from 'lucide-react';

export interface AssetIconProps {
  asset: {
    icon_url: string | null;
    kind: AssetKind;
    revocation_address: string | null;
  };
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizeClasses = {
  sm: 'w-6 h-6',
  md: 'w-8 h-8',
  lg: 'w-10 h-10',
};

export function AssetIcon({
  asset,
  size = 'sm',
  className = '',
}: AssetIconProps) {
  const sizeClass = sizeClasses[size];
  const imgClasses = `${sizeClass} rounded object-cover`;

  if (asset.icon_url) {
    return (
      <img
        src={asset.icon_url}
        className={`${imgClasses} ${className}`}
        alt='Asset icon'
        loading='lazy'
        aria-hidden='true'
      />
    );
  }

  const iconClasses = `${sizeClass} rounded stroke-1`;
  return asset.kind === 'token' ? (
    asset.revocation_address ? (
      <UndoDot className={`${iconClasses} ${className}`} aria-hidden='true' />
    ) : (
      <Coins className={`${iconClasses} ${className}`} aria-hidden='true' />
    )
  ) : asset.kind === 'nft' ? (
    <Image className={`${iconClasses} ${className}`} aria-hidden='true' />
  ) : asset.kind === 'option' ? (
    <FilePenLine className={`${iconClasses} ${className}`} aria-hidden='true' />
  ) : (
    <User className={`${iconClasses} ${className}`} aria-hidden='true' />
  );
}
