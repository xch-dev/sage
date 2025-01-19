import {
  NftParams,
  NftSortMode,
  NftGroupMode,
  SetNftParams,
} from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ArrowDownAz,
  ChevronLeftIcon,
  ChevronRightIcon,
  Clock2,
  CopyPlus,
  EyeIcon,
  EyeOff,
  Images,
  UserIcon,
  SearchIcon,
  XIcon,
  LayoutGrid,
  ArrowLeftIcon,
  Paintbrush,
} from 'lucide-react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { Input } from './ui/input';
import { useParams, useNavigate } from 'react-router-dom';

export interface NftOptionsProps {
  isCollection?: boolean;
  params: NftParams;
  setParams: SetNftParams;
  multiSelect: boolean;
  setMultiSelect: (value: boolean) => void;
  className?: string;
  isLoading?: boolean;
  canLoadMore: boolean;
}

export function NftOptions({
  isCollection,
  params: { page, sort, group, showHidden, query },
  setParams,
  multiSelect,
  setMultiSelect,
  className,
  isLoading,
  canLoadMore,
}: NftOptionsProps) {
  const { collection_id, owner_did, minter_did } = useParams();
  const navigate = useNavigate();
  const isFilteredView = Boolean(collection_id || owner_did || minter_did);
  const allowSearch = group === NftGroupMode.None || isFilteredView;
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

  return (
    <div className={`flex flex-col gap-4 ${className}`}>
      <div className='relative flex-1'>
        <div className='relative'>
          <SearchIcon className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground' />
          <Input
            value={query ?? ''}
            aria-label={t`Search NFTs by name`}
            title={t`Search NFTs by name`}
            placeholder={t`Search NFTs by name`}
            onChange={(e) => setParams({ query: e.target.value, page: 1 })}
            className='w-full pl-8 pr-8'
            disabled={!allowSearch}
          />
        </div>
        {query && (
          <Button
            variant='ghost'
            size='icon'
            title={t`Clear search`}
            aria-label={t`Clear search`}
            className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
            onClick={() => setParams({ query: '', page: 1 })}
            disabled={!allowSearch}
          >
            <XIcon className='h-4 w-4' />
          </Button>
        )}
      </div>

      <div className='flex items-center justify-between'>
        <div className='flex gap-3 items-center'>
          <Button
            variant='outline'
            size='icon'
            onClick={() =>
              !isLoading &&
              page > 1 &&
              setParams({ page: Math.max(page - 1, 1) })
            }
            aria-disabled={page === 1 || isLoading}
            aria-label={t`Previous page`}
            title={t`Previous page`}
          >
            <ChevronLeftIcon className='h-4 w-4' />
          </Button>
          <p className='text-sm text-muted-foreground font-medium w-[70px] text-center'>
            Page {page}
          </p>
          <Button
            variant='outline'
            size='icon'
            onClick={() => {
              if (!canLoadMore) return;

              if (!isLoading) setParams({ page: page + 1 });
            }}
            aria-disabled={isLoading}
            aria-label={t`Next page`}
            title={t`Next page`}
          >
            <ChevronRightIcon className='h-4 w-4' />
          </Button>
        </div>

        <div className='flex gap-2 items-center'>
          {isFilteredView && (
            <Button
              variant='outline'
              size='icon'
              onClick={handleBack}
              aria-label={t`Back to groups`}
              title={t`Back to groups`}
            >
              <ArrowLeftIcon className='h-4 w-4' />
            </Button>
          )}

          {(group === NftGroupMode.None || isFilteredView) && (
            <>
              <Button
                variant='outline'
                size='icon'
                onClick={() => setMultiSelect(!multiSelect)}
                aria-label={t`Toggle multi-select`}
                title={t`Toggle multi-select`}
              >
                <CopyPlus
                  className={`h-4 w-4 ${multiSelect ? 'text-green-600 dark:text-green-400' : ''}`}
                />
              </Button>
              <Button
                variant='outline'
                size='icon'
                onClick={() => setParams({ showHidden: !showHidden })}
                aria-label={t`Toggle hidden NFTs`}
                title={t`Toggle hidden NFTs`}
              >
                {showHidden ? (
                  <EyeIcon className='h-4 w-4' />
                ) : (
                  <EyeOff className='h-4 w-4' />
                )}
              </Button>

              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant='outline'
                    size='icon'
                    aria-label={t`Sort options`}
                    title={t`Sort options`}
                  >
                    {sort === NftSortMode.Name ? (
                      <ArrowDownAz className='h-4 w-4' />
                    ) : (
                      <Clock2 className='h-4 w-4' />
                    )}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align='end'>
                  <DropdownMenuGroup>
                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={() =>
                        setParams({ sort: NftSortMode.Name, page: 1 })
                      }
                    >
                      <ArrowDownAz className='mr-2 h-4 w-4' />
                      <span>
                        <Trans>Name</Trans>
                      </span>
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={() =>
                        setParams({ sort: NftSortMode.Recent, page: 1 })
                      }
                    >
                      <Clock2 className='mr-2 h-4 w-4' />
                      <span>
                        <Trans>Recent</Trans>
                      </span>
                    </DropdownMenuItem>
                  </DropdownMenuGroup>
                </DropdownMenuContent>
              </DropdownMenu>
            </>
          )}

          {!isCollection && !isFilteredView && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant='outline'
                  size='icon'
                  aria-label={t`Group options`}
                  title={t`Group options`}
                >
                  {group === NftGroupMode.Collection ? (
                    <Images className='h-4 w-4' />
                  ) : group === NftGroupMode.OwnerDid ? (
                    <UserIcon className='h-4 w-4' />
                  ) : group === NftGroupMode.MinterDid ? (
                    <Paintbrush className='h-4 w-4' />
                  ) : (
                    <LayoutGrid className='h-4 w-4' />
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
                  >
                    <LayoutGrid className='mr-2 h-4 w-4' />
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
                  >
                    <Images className='mr-2 h-4 w-4' />
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
                  >
                    <UserIcon className='mr-2 h-4 w-4' />
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
                  >
                    <Paintbrush className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Group by Minters</Trans>
                    </span>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>
      </div>
    </div>
  );
}
