import { OfferFindSide } from '@/bindings';
import {
  OfferParams,
  OfferStatusFilter,
  SetOfferParams,
} from '@/hooks/useOfferParams';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { AnimatePresence, motion } from 'framer-motion';
import {
  ArrowDownNarrowWide,
  ArrowUpNarrowWide,
  CalendarClock,
  Clock2,
  FilterIcon,
  Settings2,
} from 'lucide-react';
import { DebouncedSearchInput } from './DebouncedSearchInput';
import { MultiSelectToggle } from './MultiSelectToggle';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';

export interface OfferOptionsProps {
  params: OfferParams;
  setParams: SetOfferParams;
  multiSelect: boolean;
  setMultiSelect: (value: boolean) => void;
  renderPagination: () => React.ReactNode;
  className?: string;
}

const optionsPaginationVariants = {
  enter: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 20, transition: { duration: 0.15 } },
};

export function OfferOptions({
  params: { query, status, findSide, sort, ascending },
  setParams,
  multiSelect,
  setMultiSelect,
  renderPagination,
  className,
}: OfferOptionsProps) {
  const statusLabels: Record<OfferStatusFilter, string> = {
    all: t`All Offers`,
    pending: t`Pending`,
    active: t`Active`,
    completed: t`Completed`,
    cancelled: t`Cancelled`,
    expired: t`Expired`,
  };
  const statusOptions: OfferStatusFilter[] = [
    'all',
    'active',
    'completed',
    'cancelled',
    'expired',
  ];
  const currentStatusLabel = statusLabels[status];
  const statusLabel = t`Filter by status: ${currentStatusLabel}`;
  const sortLabel = t`Sort options`;

  return (
    <div
      className={cn('flex flex-col gap-4', className)}
      role='toolbar'
      aria-label={t`Offer filtering and sorting options`}
    >
      <div className='flex gap-2'>
        <DebouncedSearchInput
          value={query}
          onChange={(value) => setParams({ query: value, page: 1 })}
          placeholder={t`Search by asset, ticker, or ID`}
        />
        <Select
          value={findSide}
          onValueChange={(value) =>
            setParams({ findSide: value as OfferFindSide, page: 1 })
          }
        >
          <SelectTrigger
            className='w-[130px] shrink-0'
            aria-label={t`Search side`}
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value='any'>
              <Trans>Any side</Trans>
            </SelectItem>
            <SelectItem value='offered'>
              <Trans>Offered</Trans>
            </SelectItem>
            <SelectItem value='requested'>
              <Trans>Requested</Trans>
            </SelectItem>
          </SelectContent>
        </Select>
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
          aria-label={t`Offer view options`}
        >
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='outline'
                size='icon'
                className='relative'
                aria-label={statusLabel}
                title={statusLabel}
              >
                <FilterIcon className='h-4 w-4' aria-hidden='true' />
                {status !== 'all' && (
                  <span
                    className='absolute right-1 top-1 h-2 w-2 rounded-full bg-primary'
                    aria-hidden='true'
                  />
                )}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuLabel>
                <Trans>Status</Trans>
              </DropdownMenuLabel>
              <DropdownMenuRadioGroup
                value={status}
                onValueChange={(value) =>
                  setParams({ status: value as OfferStatusFilter, page: 1 })
                }
              >
                {statusOptions.map((option) => (
                  <DropdownMenuRadioItem
                    key={option}
                    value={option}
                    className='cursor-pointer'
                  >
                    {statusLabels[option]}
                  </DropdownMenuRadioItem>
                ))}
              </DropdownMenuRadioGroup>
            </DropdownMenuContent>
          </DropdownMenu>

          <MultiSelectToggle
            active={multiSelect}
            onToggle={() => setMultiSelect(!multiSelect)}
          />

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='outline'
                size='icon'
                aria-label={sortLabel}
                title={sortLabel}
              >
                <Settings2 className='h-4 w-4' aria-hidden='true' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuLabel>
                <Trans>Sort by</Trans>
              </DropdownMenuLabel>
              <DropdownMenuRadioGroup
                value={sort}
                onValueChange={(value) =>
                  setParams({
                    sort: value as OfferParams['sort'],
                    page: 1,
                  })
                }
              >
                <DropdownMenuRadioItem
                  value='created'
                  className='cursor-pointer'
                >
                  <Clock2 className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>Created</Trans>
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem
                  value='expiration'
                  className='cursor-pointer'
                >
                  <CalendarClock className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>Expiration</Trans>
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() => setParams({ ascending: !ascending, page: 1 })}
              >
                {ascending ? (
                  <ArrowUpNarrowWide
                    className='mr-2 h-4 w-4'
                    aria-hidden='true'
                  />
                ) : (
                  <ArrowDownNarrowWide
                    className='mr-2 h-4 w-4'
                    aria-hidden='true'
                  />
                )}
                <span>
                  {ascending ? (
                    <Trans>Ascending</Trans>
                  ) : (
                    <Trans>Descending</Trans>
                  )}
                </span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}
