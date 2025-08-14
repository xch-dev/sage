import ProfileCard from '@/components/ProfileCard';
import { t } from '@lingui/core/macro';
import { AddressItem } from './AddressItem';
export interface DidInfoProps {
  did?: string | null;
  title: string;
  className?: string;
}

export function DidInfo({ did, title, className = '' }: DidInfoProps) {
  if (!did) {
    return (
      <AddressItem label={title} address={t`None`} className={className} />
    );
  }

  return (
    <div className={className}>
      <AddressItem label={title} address={did} />
      <ProfileCard did={did} variant='compact' className='mt-1' />
    </div>
  );
}
