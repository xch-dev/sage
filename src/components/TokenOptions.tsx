import { t } from '@lingui/core/macro';
import {
  ArrowDownAz,
  ArrowUpAz,
  EyeOff,
  SearchIcon,
  Settings2,
  XIcon,
  Filter,
  FilterX,
  Eye,
  Download,
} from 'lucide-react';
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
import { useDebounce } from '@/hooks/useDebounce';
import { useState, useEffect } from 'react';
import { platform } from '@tauri-apps/plugin-os';
import { exportTokens } from '@/lib/exportTokens';
import { commands } from '@/bindings';
import { TokenRecord } from '@/types/TokenViewProps';

interface TokenOptionsProps {
  query: string;
  setQuery: (query: string) => void;
  viewMode: ViewMode;
  setViewMode: (view: ViewMode) => void;
  sortMode: TokenSortMode;
  setSortMode: (mode: TokenSortMode) => void;
  showZeroBalanceTokens: boolean;
  setShowZeroBalanceTokens: (show: boolean) => void;
  handleSearch: (value: string) => void;
  showHiddenCats: boolean;
  setShowHiddenCats: (show: boolean) => void;
  className?: string;
  tokens: TokenRecord[];
}

export function TokenOptions({
  query,
  setQuery,
  viewMode,
  setViewMode,
  sortMode,
  setSortMode,
  showZeroBalanceTokens: showZeroBalances,
  setShowZeroBalanceTokens: setShowZeroBalances,
  handleSearch,
  showHiddenCats,
  setShowHiddenCats,
  className,
  tokens,
}: TokenOptionsProps) {
  const [searchValue, setSearchValue] = useState(query);
  const debouncedSearch = useDebounce(searchValue);
  const isMobile = platform() === 'ios' || platform() === 'android';

  useEffect(() => {
    setSearchValue(query);
  }, [query]);

  useEffect(() => {
    if (debouncedSearch !== query) {
      setQuery(debouncedSearch);
      handleSearch(debouncedSearch);
    }
  }, [debouncedSearch, query, setQuery, handleSearch]);

  const handleInputChange = (value: string) => {
    setSearchValue(value);
  };

  const handleClearSearch = () => {
    handleInputChange('');
  };

  const fetchAllTokens = async () => {
    console.log('Fetching all tokens...');
    const result = await commands.getCats({});
    console.log('getCats result:', result);
    return result.cats;
  };

  return (
    <div
      className={`flex flex-col gap-4 ${className}`}
      role='toolbar'
      aria-label={t`Token filtering and sorting options`}
    >
      <div className='flex items-center gap-4'>
        <div className='relative flex-1' role='search'>
          <div className='relative'>
            <SearchIcon
              className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground'
              aria-hidden='true'
            />
            <Input
              value={searchValue}
              aria-label={t`Search tokens...`}
              title={t`Search tokens...`}
              placeholder={t`Search tokens...`}
              onChange={(e) => handleInputChange(e.target.value)}
              className='w-full pl-8 pr-8'
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
            >
              <XIcon className='h-4 w-4' aria-hidden='true' />
            </Button>
          )}
        </div>

        <div className='flex gap-2'>
          {!isMobile && tokens.length > 0 && (
            <Button
              variant='outline'
              size='icon'
              aria-label={t`Export tokens`}
              title={t`Export tokens`}
              onClick={() => exportTokens(tokens)}
            >
              <Download className='h-4 w-4' aria-hidden='true' />
            </Button>
          )}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='outline'
                size='icon'
                aria-label={t`View options`}
                title={t`View options`}
              >
                <Settings2 className='h-4 w-4' aria-hidden='true' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() => setShowZeroBalances(!showZeroBalances)}
              >
                {showZeroBalances ? (
                  <Filter className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <FilterX className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {showZeroBalances
                  ? t`Hide Zero Balance Tokens`
                  : t`Show Zero Balance Tokens`}
              </DropdownMenuItem>

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() => setShowHiddenCats(!showHiddenCats)}
              >
                {showHiddenCats ? (
                  <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <Eye className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {showHiddenCats
                  ? t`Conceal Hidden Tokens`
                  : t`Reveal Hidden Tokens`}
              </DropdownMenuItem>
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() =>
                  setSortMode(
                    sortMode === TokenSortMode.Name
                      ? TokenSortMode.Balance
                      : TokenSortMode.Name,
                  )
                }
              >
                {sortMode === TokenSortMode.Name ? (
                  <ArrowDownAz className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <ArrowUpAz className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {sortMode === TokenSortMode.Name
                  ? t`Sort by Balance (USD)`
                  : t`Sort by Name`}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <ViewToggle view={viewMode} onChange={setViewMode} />
        </div>
      </div>
    </div>
  );
}
