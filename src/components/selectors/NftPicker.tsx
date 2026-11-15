import { NftRecord, commands } from '@/bindings';
import { MutableNftBadge } from '@/components/MutableNftBadge';
import { useNftThumbnails } from '@/hooks/useOwnedNft';
import { nftUri } from '@/lib/nftUri';
import { formatAddress, isValidAddress } from '@/lib/utils';
import { plural, t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ReactNode, useCallback, useState } from 'react';
import { MultiAssetPicker } from './MultiAssetPicker';

export interface NftPickerProps {
  /** NFTs already in the offer, shown checked. */
  selected: string[];
  /** Adds the NFT to the offer, or removes it if it's already there. */
  onToggle: (nftId: string) => void;
  /** The button that opens the picker. */
  children: ReactNode;
}

/** Picks NFTs from the wallet, several at a time. */
export function NftPicker({ selected, onToggle, children }: NftPickerProps) {
  const [listed, setListed] = useState<string[]>([]);
  const thumbnails = useNftThumbnails(listed);

  const load = useCallback(async (query: string, limit: number) => {
    const page = isValidAddress(query, 'nft')
      ? await commands.getNft({ nft_id: query }).then(({ nft }) => ({
          items: nft ? [nft] : [],
          hasMore: false,
        }))
      : await commands
          .getNfts({
            name: query || null,
            offset: 0,
            limit,
            include_hidden: false,
            sort_mode: 'name',
            collection_id: null,
            owner_did_id: null,
            minter_did_id: null,
          })
          .then((data) => ({
            items: data.nfts,
            hasMore: data.nfts.length >= limit,
          }));

    setListed(page.items.map((nft) => nft.launcher_id));
    return page;
  }, []);

  return (
    <MultiAssetPicker<NftRecord>
      selected={selected}
      onToggle={onToggle}
      load={load}
      getId={(nft) => nft.launcher_id}
      searchPlaceholder={t`Search by name or paste an NFT ID`}
      isId={(query) => isValidAddress(query, 'nft')}
      idNotFoundMessage={<Trans>This NFT isn&apos;t in your wallet.</Trans>}
      emptyMessage={<Trans>No NFTs found.</Trans>}
      countLabel={(count) =>
        plural(count, { one: '# NFT added', other: '# NFTs added' })
      }
      renderItem={(nft) => (
        <>
          <img
            src={thumbnails[nft.launcher_id] ?? nftUri(null, null)}
            alt=''
            className='h-10 w-10 shrink-0 rounded-md object-cover'
            loading='lazy'
          />
          <span className='flex min-w-0 flex-1 flex-col'>
            <span className='flex min-w-0 items-center gap-1.5'>
              <span className='truncate'>{nft.name ?? t`Unnamed NFT`}</span>
              <MutableNftBadge
                kind={nft.metadata_updater_kind}
                className='shrink-0'
              />
            </span>
            <span className='truncate font-mono text-xs text-muted-foreground'>
              {formatAddress(nft.launcher_id, 10, 6)}
            </span>
          </span>
        </>
      )}
    >
      {children}
    </MultiAssetPicker>
  );
}
