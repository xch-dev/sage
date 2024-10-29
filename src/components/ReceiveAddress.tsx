import { useWalletState } from '@/state';
import { CopyBox } from './CopyBox';

export function ReceiveAddress(props: { className?: string }) {
  const walletState = useWalletState();

  return (
    <CopyBox
      title='Receive Address'
      className={props.className}
      content={walletState.sync.receive_address}
    />
  );
}
