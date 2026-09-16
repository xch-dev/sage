import { AppModalShell } from 'sage-app-ui';
import type {
  DonationDetails,
  EnvironmentGetNetworkResult,
} from 'sage-system-app-sdk';
import { CircleAlert } from 'lucide-react';
import { appIconFromInline } from '../utils';

interface Props {
  details: DonationDetails;
  network: EnvironmentGetNetworkResult | null;
  onClose: () => void;
}

export function MainnetRequired({ details, network, onClose }: Props) {
  return (
    <AppModalShell
      title='Donation unavailable'
      appName={details.appName}
      appIcon={appIconFromInline(details.appIcon)}
      footer={
        <div className='flex justify-end'>
          <button
            type='button'
            onClick={onClose}
            className='rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted'
          >
            Close
          </button>
        </div>
      }
    >
      <div className='flex flex-col items-center py-5 text-center'>
        <div className='flex h-14 w-14 items-center justify-center rounded-full bg-amber-500/10 text-amber-500'>
          <CircleAlert className='h-8 w-8' aria-hidden='true' />
        </div>
        <h2 className='mt-4 text-lg font-semibold'>
          {network
            ? 'Switch your wallet to Mainnet to donate'
            : 'Could not verify your wallet network'}
        </h2>
        <p className='mt-2 max-w-sm text-sm text-muted-foreground'>
          {network
            ? `Donations use real XCH and are only available on Mainnet. Your wallet is currently using ${network.name}.`
            : 'Sage could not confirm that your wallet is using Mainnet, so a donation cannot be sent safely.'}
        </p>
        {network ? (
          <p className='mt-2 max-w-sm text-xs text-muted-foreground'>
            Switch networks in Sage, then open this support window again.
          </p>
        ) : null}
      </div>
    </AppModalShell>
  );
}
