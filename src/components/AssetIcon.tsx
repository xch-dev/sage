import { AssetKind } from '@/bindings';
import { Coins, Image, User } from 'lucide-react';

export interface AssetIconProps {
  iconUrl?: string | null;
  name?: string | null;
  kind: AssetKind;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizeClasses = {
  sm: 'w-6 h-6',
  md: 'w-8 h-8',
  lg: 'w-10 h-10',
};

export function AssetIcon({
  iconUrl,
  kind,
  size = 'sm',
  className = '',
}: AssetIconProps) {
  const sizeClass = sizeClasses[size];
  const baseClasses = `${sizeClass} rounded object-cover`;

  if (iconUrl) {
    return (
      <img
        src={iconUrl}
        className={`${baseClasses} ${className}`}
        alt='Asset icon'
        loading='lazy'
        aria-hidden='true'
      />
    );
  }

  return kind === 'token' ? (
    <Coins
      className={`${baseClasses} ${className} stroke-1`}
      aria-hidden='true'
    />
  ) : kind === 'nft' ? (
    <Image
      className={`${baseClasses} ${className} stroke-1`}
      aria-hidden='true'
    />
  ) : (
    <User
      className={`${baseClasses} ${className} stroke-1`}
      aria-hidden='true'
    />
  );
}
