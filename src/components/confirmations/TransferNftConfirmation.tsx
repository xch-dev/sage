import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { SendIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface TransferNftConfirmationProps {
  nfts: NftRecord[];
  nftData: Record<string, NftData | null>;
  address: string;
}

export function TransferNftConfirmation({
  nfts,
  nftData,
  address,
}: TransferNftConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={SendIcon}
        title={<Trans>Transfer Details</Trans>}
        variant='warning'
      >
        {nfts.length > 1 ? (
          <Trans>These NFTs will be transferred to the address below.</Trans>
        ) : (
          <Trans>This NFT will be transferred to the address below.</Trans>
        )}
      </ConfirmationAlert>

      <CopyBox
        title={t`Recipient Address`}
        value={address}
        onCopy={() => toast.success(t`Address copied to clipboard`)}
      />

      {nfts.map((nft) => {
        const nftName = nft.name ?? t`Unnamed NFT`;
        const data = nftData[nft.launcher_id] || null;

        return (
          <ConfirmationCard
            key={nft.launcher_id}
            icon={
              <img
                alt={t`NFT artwork for ${nftName}`}
                loading='lazy'
                width='64'
                height='64'
                className='h-full w-full object-cover aspect-square color-[transparent]'
                src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
              />
            }
            title={nftName}
          >
            <CopyBox
              title={t`Launcher Id`}
              value={nft?.launcher_id ?? ''}
              onCopy={() => toast.success(t`Launcher Id copied to clipboard`)}
            />
          </ConfirmationCard>
        );
      })}
    </div>
  );
}
