import { commands, events, NftRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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
  const [nfts, setNfts] = useState<Record<string, NftRecord>>({});
  const [pageNftIds, setPageNftIds] = useState<string[]>([]);
  const [nftThumbnails, setNftThumbnails] = useState<Record<string, string>>(
    {},
  );
  const [searchTerm, setSearchTerm] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const pageSize = 8;

  // Restore focus after NFT list updates
  useEffect(() => {
    if (searchTerm && inputRef.current) {
      inputRef.current.focus();
    }
  }, [nfts, searchTerm]);

  const isValidNftId = useMemo(() => {
    return isValidAddress(searchTerm, 'nft');
  }, [searchTerm]);

  useEffect(() => {
    const fetchNfts = async () => {
      const nfts: Record<string, NftRecord> = {};

      if (value) {
        await commands
          .getNft({ nft_id: value })
          .then(({ nft }) => {
            if (nft) nfts[nft.launcher_id] = nft;
          })
          .catch(() => null);
      }

      // If we have a valid NFT ID, only fetch that specific NFT
      if (isValidNftId && searchTerm) {
        await commands
          .getNft({ nft_id: searchTerm })
          .then(({ nft }) => {
            if (nft) {
              nfts[nft.launcher_id] = nft;
              setPageNftIds([nft.launcher_id]);
            }
          })
          .catch(() => null);
      } else {
        // Otherwise, fetch NFTs based on search term
        await commands
          .getNfts({
            name: searchTerm || null,
            offset: page * pageSize,
            limit: pageSize,
            include_hidden: false,
            sort_mode: 'name',
            collection_id: null,
            owner_did_id: null,
            minter_did_id: null,
          })
          .then((data) => {
            for (const nft of data.nfts) {
              nfts[nft.launcher_id] = nft;
            }
            setPageNftIds(data.nfts.map((nft) => nft.launcher_id));
          })
          .catch(addError);
      }

      setNfts(nfts);
    };

    fetchNfts();
  }, [addError, page, searchTerm, isValidNftId, value]);

  const updateThumbnails = useCallback(async () => {
    const nftsToFetch = Object.keys(nfts);

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
  }, [nfts]);

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

  const defaultNftImage = nftUri(null, null);

  return (
    <DropdownSelector
      loadedItems={pageNftIds}
      page={page}
      setPage={setPage}
      value={value || undefined}
      setValue={(nftId) => {
        onChange(nftId);
        // Only clear search term if it's not a valid NFT ID (i.e., user clicked on an item from the list)
        if (!isValidAddress(searchTerm, 'nft')) {
          setSearchTerm('');
        }
      }}
      isDisabled={(nft) => disabled.includes(nft)}
      className={className}
      manualInput={
        <Input
          ref={inputRef}
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
      renderItem={(nftId) => (
        <div className='flex items-center gap-2 w-full'>
          <img
            src={nftThumbnails[nftId] ?? defaultNftImage}
            className='w-10 h-10 rounded object-cover'
            alt=''
            aria-hidden='true'
            loading='lazy'
          />
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate' role='text'>
              {nfts[nftId]?.name ?? 'Unknown NFT'}
            </span>
            <span
              className='text-xs text-muted-foreground truncate'
              aria-label='NFT ID'
            >
              {nftId}
            </span>
          </div>
        </div>
      )}
    />
  );
}
