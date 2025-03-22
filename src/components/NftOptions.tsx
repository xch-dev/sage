import {
  NftGroupMode,
  NftParams,
  NftSortMode,
  SetNftParams,
  CardSize,
} from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ArrowDownAz,
  ArrowLeftIcon,
  Clock2,
  CopyPlus,
  EyeIcon,
  EyeOff,
  LayoutGrid,
  LibraryBigIcon,
  Paintbrush,
  SearchIcon,
  UserIcon,
  XIcon,
  Maximize2,
  Minimize2,
  Settings2,
} from 'lucide-react';
import { useNavigate, useParams } from 'react-router-dom';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { Input } from './ui/input';
import { motion, AnimatePresence } from 'framer-motion';
import { useState, useEffect, useCallback, useRef } from 'react';
import { useDebounce } from '@/hooks/useDebounce';

export interface NftOptionsProps {
  isCollection?: boolean;
  params: NftParams;
  setParams: SetNftParams;
  multiSelect: boolean;
  setMultiSelect: (value: boolean) => void;
  className?: string;
  isLoading?: boolean;
  canLoadMore: boolean;
  total: number;
  renderPagination: () => React.ReactNode;
}

const optionsPaginationVariants = {
  enter: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 20, transition: { duration: 0.15 } },
};

export function NftOptions({
  isCollection,
  params: { sort, group, showHidden, query, cardSize },
  setParams,
  multiSelect,
  setMultiSelect,
  className,
  renderPagination,
}: NftOptionsProps) {
  const { collection_id, owner_did, minter_did } = useParams();
  const navigate = useNavigate();
  const isFilteredView = Boolean(collection_id || owner_did || minter_did);
  const allowSearch = group === NftGroupMode.None || isFilteredView;
  const [searchValue, setSearchValue] = useState(query ?? '');
  const debouncedSearch = useDebounce(searchValue, 400);
  const prevSearchRef = useRef(query);

  useEffect(() => {
    setSearchValue(query ?? '');
  }, [query]);

  useEffect(() => {
    // Convert empty string, undefined, and null to consistent values for comparison
    const normalizedDebounced = debouncedSearch || null;
    const normalizedQuery = query || null;

    // Check if queries are meaningfully different after normalization
    if (normalizedDebounced !== normalizedQuery) {
      const shouldResetPage = prevSearchRef.current !== debouncedSearch;
      prevSearchRef.current = debouncedSearch;

      setParams({
        query: debouncedSearch || null,
        ...(shouldResetPage && { page: 1 }),
      });
    }
  }, [debouncedSearch, query, setParams]);

  const handleInputChange = useCallback((value: string) => {
    setSearchValue(value);
  }, []);

  const handleClearSearch = useCallback(() => {
    setSearchValue('');
  }, []);

  const handleBack = () => {
    if (collection_id) {
      setParams({ group: NftGroupMode.Collection, page: 1 });
    } else if (owner_did) {
      setParams({ group: NftGroupMode.OwnerDid, page: 1 });
    } else if (minter_did) {
      setParams({ group: NftGroupMode.MinterDid, page: 1 });
    }
    navigate('/nfts');
  };

  // For group button
  const groupMode =
    group === NftGroupMode.Collection
      ? 'collections'
      : group === NftGroupMode.OwnerDid
        ? 'owners'
        : group === NftGroupMode.MinterDid
          ? 'minters'
          : 'no grouping';
  const groupLabel = t`Group options: currently grouped by ${groupMode}`;

  // Add view options label
  const viewLabel = t`View options`;

  return (
    <div
      className={`flex flex-col gap-4 ${className}`}
      role='toolbar'
      aria-label={t`NFT filtering and sorting options`}
    >
      <div className='relative flex-1' role='search'>
        <div className='relative'>
          <SearchIcon
            className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground'
            aria-hidden='true'
          />
          <Input
            value={searchValue}
            aria-label={t`Search NFTs...`}
            title={t`Search NFTs...`}
            placeholder={t`Search NFTs...`}
            onChange={(e) => handleInputChange(e.target.value)}
            className='w-full pl-8 pr-8'
            disabled={!allowSearch}
            aria-disabled={!allowSearch}
          />
        </div>
        {searchValue && (
          <Button
            variant='ghost'
            size='icon'
            title={t`Clear search`}
            aria-label={t`Clear search`}
            className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
            onClick={handleClearSearch}
            disabled={!allowSearch}
          >
            <XIcon className='h-4 w-4' aria-hidden='true' />
          </Button>
        )}
      </div>

      <div className='flex items-center justify-between'>
        <AnimatePresence mode='wait'>
          <motion.div
            key='pagination'
            initial={{ opacity: 0, y: 20 }}
            animate={optionsPaginationVariants.enter}
            exit={optionsPaginationVariants.exit}
          >
            {renderPagination()}
          </motion.div>
        </AnimatePresence>

        <div
          className='flex gap-2 items-center'
          role='toolbar'
          aria-label={t`NFT view options`}
        >
          {isFilteredView && (
            <Button
              variant='outline'
              size='icon'
              onClick={handleBack}
              aria-label={t`Back to groups`}
              title={t`Back to groups`}
            >
              <ArrowLeftIcon className='h-4 w-4' aria-hidden='true' />
            </Button>
          )}

          <Button
            variant='outline'
            size='icon'
            onClick={() => setMultiSelect(!multiSelect)}
            aria-label={t`Toggle multi-select`}
            title={t`Toggle multi-select`}
            disabled={!(group === NftGroupMode.None || isFilteredView)}
          >
            <CopyPlus
              className={`h-4 w-4 ${multiSelect ? 'text-green-600 dark:text-green-400' : ''}`}
              aria-hidden='true'
            />
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='outline'
                size='icon'
                aria-label={viewLabel}
                title={viewLabel}
              >
                <Settings2 className='h-4 w-4' aria-hidden='true' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={() =>
                    setParams({
                      cardSize:
                        cardSize === CardSize.Large
                          ? CardSize.Small
                          : CardSize.Large,
                    })
                  }
                  aria-label={
                    cardSize === CardSize.Large
                      ? t`Switch to small card size`
                      : t`Switch to large card size`
                  }
                >
                  {cardSize === CardSize.Large ? (
                    <Minimize2 className='mr-2 h-4 w-4' aria-hidden='true' />
                  ) : (
                    <Maximize2 className='mr-2 h-4 w-4' aria-hidden='true' />
                  )}
                  <span>
                    {cardSize === CardSize.Large ? (
                      <Trans>Small Cards</Trans>
                    ) : (
                      <Trans>Large Cards</Trans>
                    )}
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={() => setParams({ showHidden: !showHidden })}
                  disabled={
                    !(
                      group === NftGroupMode.None ||
                      isFilteredView ||
                      group === NftGroupMode.Collection
                    )
                  }
                  aria-label={
                    showHidden ? t`Hide hidden items` : t`Show hidden items`
                  }
                >
                  {showHidden ? (
                    <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                  )}
                  <span>
                    {showHidden ? (
                      <Trans>Conceal Hidden Items</Trans>
                    ) : (
                      <Trans>Reveal Hidden Items</Trans>
                    )}
                  </span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={() =>
                    setParams({
                      sort:
                        sort === NftSortMode.Name
                          ? NftSortMode.Recent
                          : NftSortMode.Name,
                      page: 1,
                    })
                  }
                  disabled={!(group === NftGroupMode.None || isFilteredView)}
                  aria-label={
                    sort === NftSortMode.Name
                      ? t`Switch to sort by recent`
                      : t`Switch to sort by name`
                  }
                >
                  {sort === NftSortMode.Name ? (
                    <Clock2 className='mr-2 h-4 w-4' aria-hidden='true' />
                  ) : (
                    <ArrowDownAz className='mr-2 h-4 w-4' aria-hidden='true' />
                  )}
                  <span>
                    {sort === NftSortMode.Name ? (
                      <Trans>Sort by Recent</Trans>
                    ) : (
                      <Trans>Sort by Name</Trans>
                    )}
                  </span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='outline'
                size='icon'
                aria-label={groupLabel}
                title={groupLabel}
                disabled={isCollection || isFilteredView}
              >
                {group === NftGroupMode.Collection ? (
                  <LibraryBigIcon className='h-4 w-4' aria-hidden='true' />
                ) : group === NftGroupMode.OwnerDid ? (
                  <UserIcon className='h-4 w-4' aria-hidden='true' />
                ) : group === NftGroupMode.MinterDid ? (
                  <Paintbrush className='h-4 w-4' aria-hidden='true' />
                ) : (
                  <LayoutGrid className='h-4 w-4' aria-hidden='true' />
                )}
              </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setParams({
                      page: 1,
                      group: NftGroupMode.None,
                    });
                  }}
                  aria-label={t`No Grouping`}
                >
                  <LayoutGrid className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>No Grouping</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setParams({
                      page: 1,
                      group: NftGroupMode.Collection,
                      query: '',
                    });
                  }}
                  aria-label={t`Group by Collections`}
                >
                  <LibraryBigIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>Group by Collections</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setParams({
                      page: 1,
                      group: NftGroupMode.OwnerDid,
                      query: '',
                    });
                  }}
                  aria-label={t`Group by Owners`}
                >
                  <UserIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>Group by Owners</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setParams({
                      page: 1,
                      group: NftGroupMode.MinterDid,
                      query: '',
                    });
                  }}
                  aria-label={t`Group by Minters`}
                >
                  <Paintbrush className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>Group by Minters</Trans>
                  </span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}
