import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Flame } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface BurnNftConfirmationProps {
  nfts: NftRecord[];
  nftData: Record<string, NftData | null>;
}

export function BurnNftConfirmation({
  nfts,
  nftData,
}: BurnNftConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={Flame}
        title={<Trans>Warning</Trans>}
        variant='danger'
      >
        {nfts.length > 1 ? (
          <Trans>
            This will permanently delete these NFTs by sending them to the burn
            address.
          </Trans>
        ) : (
          <Trans>
            This will permanently delete this NFT by sending it to the burn
            address.
          </Trans>
        )}
      </ConfirmationAlert>

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
