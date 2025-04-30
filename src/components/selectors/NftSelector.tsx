import { commands, events, NftRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { addressInfo, isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useCallback, useEffect, useMemo, useState } from 'react';
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
  const { addError } = useErrors();

  const [page, setPage] = useState(0);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [selectedNft, setSelectedNft] = useState<NftRecord | null>(null);
  const [nftThumbnails, setNftThumbnails] = useState<Record<string, string>>(
    {},
  );
  const [searchTerm, setSearchTerm] = useState('');

  const pageSize = 8;

  const isValidNftId = useMemo(() => {
    return isValidAddress(searchTerm, 'nft');
  }, [searchTerm]);

  useEffect(() => {
    commands
      .getNfts({
        name: searchTerm && !isValidNftId ? searchTerm : null,
        offset: page * pageSize,
        limit: pageSize,
        include_hidden: false,
        sort_mode: 'name',
        collection_id: null,
        owner_did_id: null,
        minter_did_id: null,
      })
      .then((data) => setNfts(data.nfts))
      .catch(addError);
  }, [addError, page, searchTerm, isValidNftId]);

  useEffect(() => {
    if (isValidNftId) {
      commands
        .getNft({ nft_id: searchTerm })
        .then((data) => {
          setSelectedNft(data.nft);
          onChange(searchTerm);
        })
        .catch(addError);
    }
  }, [isValidNftId, searchTerm, onChange, addError]);

  const updateThumbnails = useCallback(async () => {
    const nftsToFetch = [...nfts.map((nft) => nft.launcher_id)];
    if (
      value &&
      value !== '' &&
      !nfts.find((nft) => nft.launcher_id === value)
    ) {
      try {
        if (addressInfo(value).puzzleHash.length === 64) {
          nftsToFetch.push(value);
        }
      } catch {
        // The checksum failed
      }
    }

    return await Promise.all(
      nftsToFetch.map((nftId) =>
        commands
          .getNftThumbnail({ nft_id: nftId })
          .then((response) => [nftId, response.thumbnail] as const),
      ),
    ).then((thumbnails) => {
      const map: Record<string, string> = {};
      thumbnails.forEach(([id, thumbnail]) => {
        if (thumbnail !== null) map[id] = nftUri('image/png', thumbnail);
      });
      setNftThumbnails(map);
    });
  }, [nfts, value]);

  useEffect(() => {
    updateThumbnails();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (type === 'nft_data') updateThumbnails();
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateThumbnails]);

  // Load NFT record when a value is provided but not found in current nfts list
  useEffect(() => {
    if (
      value &&
      value !== '' &&
      !selectedNft &&
      !nfts.find((nft) => nft.launcher_id === value)
    ) {
      try {
        // Validate the NFT ID format
        if (isValidAddress(value, 'nft')) {
          commands
            .getNft({ nft_id: value })
            .then((data) => {
              setSelectedNft(data.nft);
            })
            .catch(addError);
        }
      } catch (error) {
        // Handle any errors silently
      }
    }
  }, [value, selectedNft, nfts, addError]);

  // Reset selectedNft when value is null or empty
  useEffect(() => {
    if (!value || value === '') {
      setSelectedNft(null);
    }
  }, [value]);

  const defaultNftImage = nftUri(null, null);

  return (
    <DropdownSelector
      loadedItems={nfts}
      page={page}
      setPage={setPage}
      isDisabled={(nft) => disabled.includes(nft.launcher_id)}
      onSelect={(nft) => {
        onChange(nft.launcher_id);
        setSelectedNft(nft);
        setSearchTerm('');
      }}
      className={className}
      manualInput={
        <Input
          placeholder={t`Search by name or enter NFT ID`}
          value={searchTerm}
          onChange={(e) => {
            const newValue = e.target.value;
            setSearchTerm(newValue);

            if (isValidAddress(newValue, 'nft')) {
              onChange(newValue);
            }
          }}
        />
      }
      renderItem={(nft) => (
        <div className='flex items-center gap-2 w-full'>
          <img
            src={nftThumbnails[nft.launcher_id] ?? defaultNftImage}
            className='w-10 h-10 rounded object-cover'
            alt=''
            aria-hidden='true'
            loading='lazy'
          />
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate' role='text'>
              {nft.name}
            </span>
            <span
              className='text-xs text-muted-foreground truncate'
              aria-label='NFT ID'
            >
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
          loading='lazy'
          alt=''
          aria-hidden='true'
          className='w-8 h-8 rounded object-cover'
        />
        <div className='flex flex-col truncate text-left'>
          <span className='truncate' role='text'>
            {selectedNft?.name ?? t`Select NFT`}
          </span>
          <span
            className='text-xs text-muted-foreground truncate'
            aria-label='NFT ID'
          >
            {selectedNft?.launcher_id}
          </span>
        </div>
      </div>
    </DropdownSelector>
  );
}
