import { NftRecord, commands } from '@/bindings';
import { MutableNftBadge } from '@/components/MutableNftBadge';
import { Button } from '@/components/ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { useNftThumbnails } from '@/hooks/useOwnedNft';
import { nftUri } from '@/lib/nftUri';
import { cn, formatAddress, isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { CheckIcon, Loader2 } from 'lucide-react';
import { ReactNode, useEffect, useState } from 'react';

const pageSize = 20;

export interface NftPickerProps {
  /** NFTs already in the offer, shown checked. */
  selected: string[];
  /** Adds the NFT to the offer, or removes it if it's already there. */
  onToggle: (nftId: string) => void;
  /** The button that opens the picker. */
  children: ReactNode;
}

/**
 * A searchable list of the wallet's NFTs that opens from a button. It stays
 * open so that several NFTs can be picked in one go.
 */
export function NftPicker({ selected, onToggle, children }: NftPickerProps) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [limit, setLimit] = useState(pageSize);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const thumbnails = useNftThumbnails(nfts.map((nft) => nft.launcher_id));

  const query = search.trim();
  const pastedNftId = isValidAddress(query, 'nft') ? query : null;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;

    // Wait for typing to pause before searching.
    const timeout = setTimeout(
      () => {
        setIsLoading(true);
        const request = pastedNftId
          ? commands.getNft({ nft_id: pastedNftId }).then(({ nft }) => ({
              nfts: nft ? [nft] : [],
              hasMore: false,
            }))
          : commands
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
                nfts: data.nfts,
                hasMore: data.nfts.length >= limit,
              }));

        request
          .then((result) => {
            if (cancelled) return;
            setNfts(result.nfts);
            setHasMore(result.hasMore);
          })
          .catch(() => {
            if (!cancelled) setNfts([]);
          })
          .finally(() => {
            if (!cancelled) setIsLoading(false);
          });
      },
      query ? 250 : 0,
    );

    return () => {
      cancelled = true;
      clearTimeout(timeout);
    };
  }, [open, query, pastedNftId, limit]);

  const handleOpenChange = (next: boolean) => {
    setOpen(next);
    if (!next) {
      setSearch('');
      setLimit(pageSize);
    }
  };

  const count = selected.length;

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>{children}</PopoverTrigger>
      <PopoverContent
        className='w-[min(26rem,calc(100vw-2rem))] p-0'
        align='start'
      >
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={t`Search by name or paste an NFT ID`}
            value={search}
            onValueChange={(value) => {
              setSearch(value);
              setLimit(pageSize);
            }}
          />
          <CommandList className='max-h-80'>
            {!isLoading && (
              <CommandEmpty>
                {pastedNftId ? (
                  <Trans>This NFT isn&apos;t in your wallet.</Trans>
                ) : (
                  <Trans>No NFTs found.</Trans>
                )}
              </CommandEmpty>
            )}
            {nfts.length > 0 && (
              <CommandGroup>
                {nfts.map((nft) => {
                  const checked = selected.includes(nft.launcher_id);

                  return (
                    <CommandItem
                      key={nft.launcher_id}
                      value={nft.launcher_id}
                      onSelect={() => onToggle(nft.launcher_id)}
                      className='gap-3'
                      aria-checked={checked}
                    >
                      <span
                        aria-hidden='true'
                        className={cn(
                          'flex h-4 w-4 shrink-0 items-center justify-center rounded-sm border',
                          checked
                            ? 'border-primary bg-primary text-primary-foreground'
                            : 'border-muted-foreground',
                        )}
                      >
                        {checked && <CheckIcon className='h-3 w-3' />}
                      </span>
                      <img
                        src={thumbnails[nft.launcher_id] ?? nftUri(null, null)}
                        alt=''
                        className='h-10 w-10 shrink-0 rounded-md object-cover'
                        loading='lazy'
                      />
                      <span className='flex min-w-0 flex-1 flex-col'>
                        <span className='flex min-w-0 items-center gap-1.5'>
                          <span className='truncate'>
                            {nft.name ?? t`Unnamed NFT`}
                          </span>
                          <MutableNftBadge
                            kind={nft.metadata_updater_kind}
                            className='shrink-0'
                          />
                        </span>
                        <span className='truncate font-mono text-xs text-muted-foreground'>
                          {formatAddress(nft.launcher_id, 10, 6)}
                        </span>
                      </span>
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
            {isLoading && (
              <div className='flex items-center justify-center gap-2 py-4 text-sm text-muted-foreground'>
                <Loader2 className='h-4 w-4 animate-spin' aria-hidden='true' />
                <Trans>Loading NFTs…</Trans>
              </div>
            )}
            {!isLoading && hasMore && (
              <div className='p-1'>
                <Button
                  type='button'
                  variant='ghost'
                  size='sm'
                  className='w-full'
                  onClick={() => setLimit(limit + pageSize)}
                >
                  <Trans>Show more</Trans>
                </Button>
              </div>
            )}
          </CommandList>
        </Command>
        <div className='flex items-center justify-between gap-2 border-t px-3 py-2'>
          <span className='text-xs text-muted-foreground'>
            <Plural value={count} one='# NFT added' other='# NFTs added' />
          </span>
          <Button
            type='button'
            size='sm'
            onClick={() => handleOpenChange(false)}
          >
            <Trans>Done</Trans>
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}
