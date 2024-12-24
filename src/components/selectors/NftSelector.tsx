import { commands, NftRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { addressInfo } from '@/lib/utils';
import { useWalletState } from '@/state';
import { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';

export interface NftSelectorProps {
  value: string | null;
  onChange: (value: string) => void;
  disabled?: string[];
  className?: string;
}

export function NftSelector({
  value,
  onChange,
  disabled = [],
  className,
}: NftSelectorProps) {
  const walletState = useWalletState();
  const { addError } = useErrors();

  const [page, setPage] = useState(0);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [selectedNft, setSelectedNft] = useState<NftRecord | null>(null);
  const [nftThumbnails, setNftThumbnails] = useState<Record<string, string>>(
    {},
  );

  const pageSize = 8;

  useEffect(() => {
    commands
      .getNfts({
        offset: page * pageSize,
        limit: pageSize,
        include_hidden: false,
        collection_id: 'all',
        sort_mode: 'name',
      })
      .then((data) => setNfts(data.nfts))
      .catch(addError);
  }, [addError, page]);

  useEffect(() => {
    if (value && selectedNft?.launcher_id !== value) {
      try {
        if (addressInfo(value).puzzleHash.length !== 64) {
          return setSelectedNft(null);
        }
      } catch {
        return setSelectedNft(null);
      }

      commands
        .getNft({ nft_id: value })
        .then((data) => setSelectedNft(data.nft))
        .catch(addError);
    } else if (!value) {
      setSelectedNft(null);
    }
  }, [value, selectedNft?.launcher_id, addError]);

  useEffect(() => {
    const nftsToFetch = [...nfts.map((nft) => nft.launcher_id)];
    if (value && !nfts.find((nft) => nft.launcher_id === value)) {
      try {
        if (addressInfo(value).puzzleHash.length === 64) {
          nftsToFetch.push(value);
        }
      } catch {
        // The checksum failed
      }
    }

    Promise.all(
      nftsToFetch.map((nftId) =>
        commands
          .getNftData({ nft_id: nftId })
          .then((response) => [nftId, response.data] as const),
      ),
    ).then((thumbnails) => {
      const map: Record<string, string> = {};
      thumbnails.forEach(([id, thumbnail]) => {
        if (thumbnail !== null)
          map[id] = nftUri(thumbnail.mime_type, thumbnail.blob);
      });
      setNftThumbnails(map);
    });
  }, [nfts, value]);

  const defaultNftImage = nftUri(null, null);

  return (
    <DropdownSelector
      totalItems={walletState.nfts.visible_nfts}
      loadedItems={nfts}
      page={page}
      setPage={setPage}
      isDisabled={(nft) => disabled.includes(nft.launcher_id)}
      onSelect={(nft) => {
        onChange(nft.launcher_id);
        setSelectedNft(nft);
      }}
      className={className}
      manualInput={
        <Input
          placeholder='Enter NFT id'
          value={value || ''}
          onChange={(e) => {
            onChange(e.target.value);
            setSelectedNft(
              nfts.find((nft) => nft.launcher_id === e.target.value) ?? null,
            );
          }}
        />
      }
      renderItem={(nft) => (
        <div className='flex items-center gap-2 w-full'>
          <img
            src={nftThumbnails[nft.launcher_id] ?? defaultNftImage}
            className='w-10 h-10 rounded object-cover'
            alt={nft.name ?? 'Unknown'}
          />
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate'>{nft.name}</span>
            <span className='text-xs text-muted-foreground truncate'>
              {nft.launcher_id}
            </span>
          </div>
        </div>
      )}
    >
      <div className='flex items-center gap-2 min-w-0'>
        <img
          src={
            selectedNft
              ? (nftThumbnails[selectedNft.launcher_id] ?? defaultNftImage)
              : defaultNftImage
          }
          className='w-8 h-8 rounded object-cover'
        />
        <div className='flex flex-col truncate text-left'>
          <span className='truncate'>{selectedNft?.name ?? 'Select NFT'}</span>
          <span className='text-xs text-muted-foreground truncate'>
            {selectedNft?.launcher_id}
          </span>
        </div>
      </div>
    </DropdownSelector>
  );
}
