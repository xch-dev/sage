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
  return (
    <div className={`flex flex-col gap-4 ${className}`}>
      <div className='relative flex-1'>
        <div className='relative'>
          <SearchIcon className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground' />
          <Input
            value={query ?? ''}
            aria-label={t`Search NFTs`}
            title={t`Search NFTs`}
            onChange={(e) => setParams({ query: e.target.value, page: 1 })}
            className='w-full pl-8 pr-8'
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
          {group === NftGroupMode.None && (
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
                      onClick={(e) => {
                        e.stopPropagation();
                        setParams({
                          page: 1,
                          sort: NftSortMode.Name,
                        });
                      }}
                    >
                      <ArrowDownAz className='mr-2 h-4 w-4' />
                      <span>
                        <Trans>Sort Alphabetically</Trans>
                      </span>
                    </DropdownMenuItem>

                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        setParams({
                          page: 1,
                          sort: NftSortMode.Recent,
                        });
                      }}
                    >
                      <Clock2 className='mr-2 h-4 w-4' />
                      <span>
                        <Trans>Sort Recent</Trans>
                      </span>
                    </DropdownMenuItem>
                  </DropdownMenuGroup>
                </DropdownMenuContent>
              </DropdownMenu>
            </>
          )}

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

          {!isCollection && (
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
                      });
                    }}
                  >
                    <UserIcon className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Group by Owners</Trans>
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
