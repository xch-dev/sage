import { t } from '@lingui/core/macro';
import { ArrowDownAz, ArrowUpAz, EyeIcon, EyeOff, SearchIcon, Settings2, XIcon } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { ViewToggle, ViewMode } from './ViewToggle';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { TokenSortMode } from '@/hooks/useTokenParams';

interface TokenOptionsProps {
  query: string;
  setQuery: (query: string) => void;
  viewMode: ViewMode;
  setViewMode: (view: ViewMode) => void;
  sortMode: TokenSortMode;
  setSortMode: (mode: TokenSortMode) => void;
  showHidden: boolean;
  setShowHidden: (show: boolean) => void;
  handleSearch: (value: string) => void;
  className?: string;
}

export function TokenOptions({
  query,
  setQuery,
  viewMode,
  setViewMode,
  sortMode,
  setSortMode,
  showHidden,
  setShowHidden,
  handleSearch,
  className,
}: TokenOptionsProps) {
  return (
    <div 
      className={`flex flex-col gap-4 ${className}`}
      role="toolbar"
      aria-label={t`Token filtering and sorting options`}
    >
      <div className="flex items-center gap-4">
        <div className="relative flex-1" role="search">
          <div className="relative">
            <SearchIcon
              className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
              aria-hidden="true"
            />
            <Input
              value={query}
              aria-label={t`Search tokens...`}
              title={t`Search tokens...`}
              placeholder={t`Search tokens...`}
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
                onClick={() => setShowHidden(!showHidden)}
              >
                {showHidden ? (
                  <EyeOff className="mr-2 h-4 w-4" aria-hidden="true" />
                ) : (
                  <EyeIcon className="mr-2 h-4 w-4" aria-hidden="true" />
                )}
                {showHidden ? t`Hide Hidden Tokens` : t`Show Hidden Tokens`}
              </DropdownMenuItem>
              <DropdownMenuItem
                className="cursor-pointer"
                onClick={() => setSortMode(sortMode === TokenSortMode.Name ? TokenSortMode.Balance : TokenSortMode.Name)}
              >
                {sortMode === TokenSortMode.Name ? (
                  <ArrowDownAz className="mr-2 h-4 w-4" aria-hidden="true" />
                ) : (
                  <ArrowUpAz className="mr-2 h-4 w-4" aria-hidden="true" />
                )}
                {sortMode === TokenSortMode.Name ? t`Sort by Balance` : t`Sort by Name`}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <ViewToggle view={viewMode} onChange={setViewMode} />
        </div>
      </div>
    </div>
  );
} 