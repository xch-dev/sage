import { NftData, NftRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Flame, SendIcon, UserRoundPlus } from 'lucide-react';
import { toast } from 'react-toastify';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface NftConfirmationProps {
  type: 'burn' | 'transfer' | 'edit';
  nfts: NftRecord[];
  nftData: Record<string, NftData | null>;
  address?: string; // Required for transfer
  profileId?: string | null; // Required for edit
}

export function NftConfirmation({
  type,
  nfts,
  nftData,
  address,
  profileId,
}: NftConfirmationProps) {
  const config = {
    burn: {
      icon: Flame,
      title: <Trans>Warning</Trans>,
      variant: 'danger' as const,
      message:
        nfts.length > 1 ? (
          <Trans>
            This will permanently delete these NFTs by sending them to the burn
            address.
          </Trans>
        ) : (
          <Trans>
            This will permanently delete this NFT by sending it to the burn
            address.
          </Trans>
        ),
    },
    transfer: {
      icon: SendIcon,
      title: <Trans>Transfer Details</Trans>,
      variant: 'warning' as const,
      message:
        nfts.length > 1 ? (
          <Trans>These NFTs will be transferred to the address below.</Trans>
        ) : (
          <Trans>This NFT will be transferred to the address below.</Trans>
        ),
    },
    edit: {
      icon: UserRoundPlus,
      title: <Trans>Edit Profile</Trans>,
      variant: 'info' as const,
      message: profileId ? (
        <Trans>
          These NFTs will be assigned to the selected profile. This will
          associate them with your decentralized identity.
        </Trans>
      ) : (
        <Trans>
          These NFTs will be unassigned from their current profile. They will no
          longer be associated with any decentralized identity.
        </Trans>
      ),
    },
  };

  const { icon, title, variant, message } = config[type];

  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert icon={icon} title={title} variant={variant}>
        {message}
      </ConfirmationAlert>

      {type === 'transfer' && address && (
        <CopyBox
          title={t`Recipient Address`}
          value={address}
          onCopy={() => toast.success(t`Address copied to clipboard`)}
        />
      )}

      {type === 'edit' && profileId && (
        <ConfirmationCard
          icon={<UserRoundPlus className='h-8 w-8 text-purple-500' />}
          title={<Trans>Selected Profile</Trans>}
        >
          <CopyBox
            title={t`Profile ID`}
            value={profileId}
            onCopy={() => toast.success(t`Profile ID copied to clipboard`)}
          />
        </ConfirmationCard>
      )}

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
