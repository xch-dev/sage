import { NftParams, NftView, SetNftParams } from '@/hooks/useNftParams';
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
}

export function NftOptions({
  isCollection,
  params: { page, view, showHidden, query },
  setParams,
  multiSelect,
  setMultiSelect,
  className,
  isLoading,
}: NftOptionsProps) {
  return (
    <div className={`flex flex-col gap-4 ${className}`}>
      <Input
        type='search'
        placeholder={t`Search NFTs...`}
        value={query ?? ''}
        onChange={(e) => setParams({ query: e.target.value, page: 1 })}
        className='w-full'
      />

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
          >
            <ChevronLeftIcon className='h-4 w-4' />
          </Button>
          <p className='text-sm text-muted-foreground font-medium w-[70px] text-center'>
            Page {page}
          </p>
          <Button
            variant='outline'
            size='icon'
            onClick={() => !isLoading && setParams({ page: page + 1 })}
            aria-disabled={isLoading}
            aria-label={t`Next page`}
          >
            <ChevronRightIcon className='h-4 w-4' />
          </Button>
        </div>

        <div className='flex gap-2 items-center'>
          {view !== NftView.Collection && (
            <Button
              variant='outline'
              size='icon'
              onClick={() => setMultiSelect(!multiSelect)}
              aria-label={t`Toggle multi-select`}
            >
              <CopyPlus
                className={`h-4 w-4 ${multiSelect ? 'text-green-600 dark:text-green-400' : ''}`}
              />
            </Button>
          )}

          <Button
            variant='outline'
            size='icon'
            onClick={() => setParams({ showHidden: !showHidden })}
            aria-label={t`Toggle hidden NFTs`}
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
              >
                {view === 'name' ? (
                  <ArrowDownAz className='h-4 w-4' />
                ) : view === 'recent' ? (
                  <Clock2 className='h-4 w-4' />
                ) : (
                  <Images className='h-4 w-4' />
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
                      view: NftView.Name,
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
                      view: NftView.Recent,
                    });
                  }}
                >
                  <Clock2 className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Sort Recent</Trans>
                  </span>
                </DropdownMenuItem>

                {!isCollection && (
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setParams({
                        page: 1,
                        view: NftView.Collection,
                      });
                    }}
                  >
                    <Images className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Group Collections</Trans>
                    </span>
                  </DropdownMenuItem>
                )}
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}
