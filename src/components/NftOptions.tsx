import { NftParams, NftView, SetNftParams } from '@/hooks/useNftParams';
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
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';

export interface NftOptionsProps {
  totalPages: number;
  isCollection?: boolean;
  params: NftParams;
  setParams: SetNftParams;
  multiSelect: boolean;
  setMultiSelect: (value: boolean) => void;
  className?: string;
}

export function NftOptions({
  totalPages,
  isCollection,
  params: { page, view, showHidden },
  setParams,
  multiSelect,
  setMultiSelect,
  className,
}: NftOptionsProps) {
  return (
    <div className={`flex items-center justify-between ${className}`}>
      <div className='flex gap-3 items-center'>
        <Button
          variant='outline'
          size='icon'
          onClick={() => setParams({ page: Math.max(page - 1, 1) })}
          disabled={page === 1}
          aria-label={t`Previous page`}
        >
          <ChevronLeftIcon className='h-4 w-4' />
        </Button>
        <p className='text-sm text-muted-foreground font-medium'>
          {page} / {totalPages}
        </p>
        <Button
          variant='outline'
          size='icon'
          onClick={() => setParams({ page: Math.min(page + 1, totalPages) })}
          disabled={page === totalPages}
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
            <Button variant='outline' size='icon' aria-label={t`Sort options`}>
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
  );
}
