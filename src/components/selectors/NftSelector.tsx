import { commands, NftRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { useWalletState } from '@/state';
import { useEffect, useState } from 'react';
import { DropdownSelector } from './DropdownSelector';

export interface NftSelectorProps {
  initialNft: string | null;
  onNftChange: (value: string) => void;
  disabledNfts: string[];
}

export function NftSelector({
  initialNft,
  onNftChange,
  disabledNfts,
}: NftSelectorProps) {
  const walletState = useWalletState();
  const { addError } = useErrors();

  const [page, setPage] = useState(0);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [selectedNftId, setSelectedNftId] = useState<string | null>(initialNft);
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
    if (selectedNftId && selectedNft?.launcher_id !== selectedNftId) {
      commands
        .getNft({ nft_id: selectedNftId })
        .then((data) => setSelectedNft(data.nft))
        .catch(addError);
    }
  }, [selectedNftId, selectedNft?.launcher_id, addError]);

  useEffect(() => {
    const nftsToFetch = [...nfts.map((nft) => nft.launcher_id)];
    if (
      selectedNftId &&
      !nfts.find((nft) => nft.launcher_id === selectedNftId)
    ) {
      nftsToFetch.push(selectedNftId);
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
  }, [nfts, selectedNftId]);

  const defaultNftImage = nftUri(null, null);

  return (
    <DropdownSelector
      totalItems={walletState.nfts.visible_nfts}
      pageSize={pageSize}
      page={page}
      setPage={setPage}
      loadedItems={nfts}
      isDisabled={(nft) => disabledNfts.includes(nft.launcher_id)}
      onSelect={(nft) => {
        onNftChange(nft.launcher_id);
        setSelectedNftId(nft.launcher_id);
        setSelectedNft(nft);
      }}
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
