import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LinkIcon } from 'lucide-react';
import { toast } from 'react-toastify';

interface AddUrlConfirmationProps {
  nft: NftRecord;
  nftData: NftData | null;
  url: string;
  kind: string;
}

export function AddUrlConfirmation({
  nft,
  nftData,
  url,
  kind,
}: AddUrlConfirmationProps) {
  const nftName = nft.name ?? t`Unnamed NFT`;

  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800 rounded-md text-blue-800 dark:text-blue-300'>
        <div className='font-medium mb-1 flex items-center'>
          <LinkIcon className='h-3 w-3 mr-1' />
          <Trans>URL</Trans>
        </div>
        <div>
          <Trans>
            A new URL will be added to this NFT. URLs cannot be removed once
            added.
          </Trans>
        </div>
      </div>

      <div className='flex items-start gap-3 border border-neutral-200 dark:border-neutral-800 rounded-md p-3'>
        <div className='overflow-hidden rounded-md flex-shrink-0 w-16 h-16 border border-neutral-200 dark:border-neutral-800'>
          <img
            alt={t`NFT artwork for ${nftName}`}
            loading='lazy'
            width='64'
            height='64'
            className='h-full w-full object-cover aspect-square color-[transparent]'
            src={nftUri(nftData?.mime_type ?? null, nftData?.blob ?? null)}
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

      <div className='space-y-2'>
        <div className='font-medium'>
          <Trans>URL Details</Trans>
        </div>
        <CopyBox
          title={t`URL`}
          value={url}
          onCopy={() => toast.success(t`URL copied to clipboard`)}
        />
        <div className='flex items-center justify-between text-sm'>
          <span className='text-muted-foreground'>
            <Trans>Kind</Trans>:
          </span>
          <span className='font-medium capitalize'>{kind}</span>
        </div>
      </div>
    </div>
  );
}
