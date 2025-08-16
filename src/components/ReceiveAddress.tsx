import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { AddressItem } from './AddressItem';

export function ReceiveAddress({ className }: { className?: string }) {
  const { receive_address } = useWalletState().sync;

  if (!receive_address || receive_address === 'Unknown') {
    return <div className={className}>Connecting to wallet...</div>;
  }

  return (
    <AddressItem
      address={receive_address}
      className={className}
      label={t`Receive Address`}
      hideLabel={true}
    />
  );
}
