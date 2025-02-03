import { t } from '@lingui/core/macro';
import { ArrowDownAz, ArrowUpAz, SearchIcon, Settings2, XIcon } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Pagination } from './Pagination';
import { ViewToggle, ViewMode } from './ViewToggle';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';

interface TransactionOptionsProps {
  query: string;
  setQuery: (query: string) => void;
  page: number;
  setPage: (page: number) => void;
  pageSize: number;
  setPageSize: (pageSize: number) => void;
  total: number;
  isLoading?: boolean;
  view: ViewMode;
  setView: (view: ViewMode) => void;
  ascending: boolean;
  setAscending: (ascending: boolean) => void;
  className?: string;
  handleSearch: (value: string) => void;
}

export function TransactionOptions({
  query,
  setQuery,
  page,
  setPage,
  pageSize,
  setPageSize,
  total,
  isLoading,
  view,
  setView,
  ascending,
  setAscending,
  className,
  handleSearch,
}: TransactionOptionsProps) {
  return (
    <div 
      className={`flex flex-col gap-4 ${className}`}
      role="toolbar"
      aria-label={t`Transaction filtering and sorting options`}
    >
      <div className="relative flex-1" role="search">
        <div className="relative">
          <SearchIcon
            className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden="true"
          />
          <Input
            value={query}
            aria-label={t`Search transactions...`}
            title={t`Search transactions...`}
            placeholder={t`Search transactions...`}
            onChange={(e) => {
              setQuery(e.target.value);
              handleSearch(e.target.value);
            }}
            className="w-full pl-8 pr-8"
          />
        </div>
        {query && (
          <Button
            variant="ghost"
            size="icon"
            title={t`Clear search`}
            aria-label={t`Clear search`}
            className="absolute right-0 top-0 h-full px-2 hover:bg-transparent"
            onClick={() => {
              setQuery('');
              handleSearch('');
            }}
          >
            <XIcon className="h-4 w-4" aria-hidden="true" />
          </Button>
        )}
      </div>

      <div className="flex items-center justify-between">
        <Pagination
          page={page}
          total={total}
          pageSize={pageSize}
          onPageChange={setPage}
          onPageSizeChange={setPageSize}
          pageSizeOptions={[10, 20, 30, 40, 50]}
          compact={true}
          isLoading={isLoading}
        />
        <div className="flex gap-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                aria-label={t`View options`}
                title={t`View options`}
              >
                <Settings2 className="h-4 w-4" aria-hidden="true" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem
                className="cursor-pointer"
                onClick={() => setAscending(!ascending)}
              >
                {ascending ? (
                  <ArrowDownAz className="mr-2 h-4 w-4" aria-hidden="true" />
                ) : (
                  <ArrowUpAz className="mr-2 h-4 w-4" aria-hidden="true" />
                )}
                {ascending ? t`Sort Descending` : t`Sort Ascending`}
              </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
                            <ViewToggle view={view} onChange={setView} />

        </div>
      </div>
    </div>
  );
} 