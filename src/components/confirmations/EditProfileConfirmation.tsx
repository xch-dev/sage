import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { UserRoundPlus } from 'lucide-react';
import { toast } from 'react-toastify';

interface EditProfileConfirmationProps {
  nfts: NftRecord[];
  nftData: Record<string, NftData | null>;
  profileId: string | null;
}

export function EditProfileConfirmation({
  nfts,
  nftData,
  profileId,
}: EditProfileConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-purple-50 dark:bg-purple-950 border border-purple-200 dark:border-purple-800 rounded-md text-purple-800 dark:text-purple-300'>
        <div className='font-medium mb-1 flex items-center'>
          <UserRoundPlus className='h-3 w-3 mr-1' />
          <Trans>Profile Assignment</Trans>
        </div>
        <div>
          {nfts.length > 1 ? (
            profileId ? (
              <Trans>These NFTs will be assigned to the profile below.</Trans>
            ) : (
              <Trans>These NFTs will be unassigned from their profiles.</Trans>
            )
          ) : profileId ? (
            <Trans>This NFT will be assigned to the profile below.</Trans>
          ) : (
            <Trans>This NFT will be unassigned from its profile.</Trans>
          )}
        </div>
      </div>

      {profileId && (
        <CopyBox
          title={t`Profile ID`}
          value={profileId}
          onCopy={() => toast.success(t`Profile ID copied to clipboard`)}
        />
      )}

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
