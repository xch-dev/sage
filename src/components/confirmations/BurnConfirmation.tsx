import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Flame } from 'lucide-react';
import { toast } from 'react-toastify';

interface BurnConfirmationProps {
  nfts: NftRecord[];
  nftData: Record<string, NftData | null>;
}

export function BurnConfirmation({ nfts, nftData }: BurnConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded-md text-amber-800 dark:text-amber-300'>
        <div className='font-medium mb-1 flex items-center'>
          <Flame className='h-3 w-3 mr-1' />
          <Trans>Warning</Trans>
        </div>
        <div>
          {nfts.length > 1 ? (
            <Trans>
              This will permanently delete these NFTs by sending them to the
              burn address.
            </Trans>
          ) : (
            <Trans>
              This will permanently delete this NFT by sending it to the burn
              address.
            </Trans>
          )}
        </div>
      </div>

      {nfts.map((nft) => {
        const nftName = nft.name ?? t`Unnamed NFT`;
        const data = nftData[nft.launcher_id] || null;

        return (
          <div
            key={nft.launcher_id}
            className='flex items-start gap-3 border border-neutral-200 dark:border-neutral-800 rounded-md p-3'
          >
            <div className='overflow-hidden rounded-md flex-shrink-0 w-16 h-16 border border-neutral-200 dark:border-neutral-800'>
              <img
                alt={t`NFT artwork for ${nftName}`}
                loading='lazy'
                width='64'
                height='64'
                className='h-full w-full object-cover aspect-square color-[transparent]'
                src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
              />
            </div>
            <div className='break-words whitespace-pre-wrap flex-1'>
              <div className='font-medium'>{nftName}</div>
              <CopyBox
                title={t`Launcher Id`}
                value={nft?.launcher_id ?? ''}
                onCopy={() => toast.success(t`Launcher Id copied to clipboard`)}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}
