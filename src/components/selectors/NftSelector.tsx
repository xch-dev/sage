import { commands, events, NftRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { SearchableSelect } from './SearchableSelect';

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

  const pageSize = 8;

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

  const handleSelect = useCallback(
    (nftId: string | null) => {
      if (nftId) {
        onChange(nftId);
      }
    },
    [onChange],
  );

  const handleManualInput = useCallback(
    (nftId: string) => {
      onChange(nftId);
    },
    [onChange],
  );

  const handleSearchChange = useCallback(
    (search: string) => {
      setSearchTerm(search);
      // Reset to first page when search changes
      if (page !== 0) {
        setPage(0);
      }
    },
    [page],
  );

  // Get the NFT records for the current page, always including the selected NFT
  const nftItems = useMemo(() => {
    const pageItems = pageNftIds
      .map((id) => nfts[id])
      .filter(Boolean) as NftRecord[];
    if (value && nfts[value] && !pageNftIds.includes(value)) {
      return [nfts[value], ...pageItems];
    }
    return pageItems;
  }, [pageNftIds, nfts, value]);

  const renderNft = useCallback(
    (nft: NftRecord) => (
      <div className='flex items-center gap-2 min-w-0'>
        <img
          src={nftThumbnails[nft.launcher_id] ?? defaultNftImage}
          className='w-10 h-10 rounded object-cover flex-shrink-0'
          alt=''
          aria-hidden='true'
          loading='lazy'
        />
        <div className='flex flex-col min-w-0'>
          <span className='truncate' role='text'>
            {nft.name ?? 'Unknown NFT'}
          </span>
          <span
            className='text-xs text-muted-foreground truncate'
            aria-label='NFT ID'
          >
            {nft.launcher_id}
          </span>
        </div>
      </div>
    ),
    [nftThumbnails, defaultNftImage],
  );

  const validateNftId = useCallback((value: string) => {
    return isValidAddress(value, 'nft');
  }, []);

  return (
    <SearchableSelect
      value={value || undefined}
      onSelect={handleSelect}
      items={nftItems}
      getItemId={(nft) => nft.launcher_id}
      renderItem={renderNft}
      onSearchChange={handleSearchChange}
      shouldFilter={false}
      validateManualInput={validateNftId}
      onManualInput={handleManualInput}
      page={page}
      onPageChange={setPage}
      pageSize={pageSize}
      hasMorePages={pageNftIds.length >= pageSize}
      disabled={disabled}
      className={className}
      placeholder={t`Select NFT`}
      searchPlaceholder={t`Search by name or enter NFT ID`}
      emptyMessage={t`No NFTs found.`}
    />
  );
}
