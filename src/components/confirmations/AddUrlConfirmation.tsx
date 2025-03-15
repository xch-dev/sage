import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LinkIcon } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

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
      <ConfirmationAlert
        icon={LinkIcon}
        title={<Trans>Add URL</Trans>}
        variant='info'
      >
        <Trans>
          You are adding a URL to this NFT. This will be stored on-chain and can
          be used to link to external content.
        </Trans>
      </ConfirmationAlert>

      <ConfirmationCard
        icon={
          <img
            alt={t`NFT artwork for ${nftName}`}
            loading='lazy'
            width='64'
            height='64'
            className='h-full w-full object-cover aspect-square color-[transparent]'
            src={nftUri(nftData?.mime_type ?? null, nftData?.blob ?? null)}
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

      <ConfirmationCard title={<Trans>URL Details</Trans>}>
        <div className='space-y-2'>
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
      </ConfirmationCard>
    </div>
  );
}
