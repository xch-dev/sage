import { useDebounce } from '@/hooks/useDebounce';
import { OptionSortMode } from '@/hooks/useOptionParams';
import { t } from '@lingui/core/macro';
import {
  ArrowDownAz,
  ArrowUpAz,
  Download,
  Eye,
  EyeOff,
  SearchIcon,
  Settings2,
  XIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { Input } from './ui/input';
import { ViewMode, ViewToggle } from './ViewToggle';

interface TokenOptionsProps {
  query: string;
  setQuery: (query: string) => void;
  viewMode: ViewMode;
  setViewMode: (view: ViewMode) => void;
  sortMode: OptionSortMode;
  setSortMode: (mode: OptionSortMode) => void;
  showHiddenOptions: boolean;
  handleSearch: (value: string) => void;
  setShowHiddenOptions: (show: boolean) => void;
  className?: string;
  onExport?: () => void;
}

export function OptionOptions({
  query,
  setQuery,
  viewMode,
  setViewMode,
  sortMode,
  setSortMode,
  showHiddenOptions,
  setShowHiddenOptions,
  handleSearch,
  className,
  onExport,
}: TokenOptionsProps) {
  const [searchValue, setSearchValue] = useState(query);
  const debouncedSearch = useDebounce(searchValue);

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

  return (
    <div
      className={`flex flex-col gap-4 ${className}`}
      role='toolbar'
      aria-label={t`Option contract filtering and sorting options`}
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
              aria-label={t`Search options...`}
              title={t`Search options...`}
              placeholder={t`Search options...`}
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
          <Button
            variant='outline'
            size='icon'
            aria-label={t`Export options`}
            title={t`Export options`}
            onClick={onExport}
          >
            <Download className='h-4 w-4' aria-hidden='true' />
          </Button>
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
                onClick={() => setShowHiddenOptions(!showHiddenOptions)}
              >
                {showHiddenOptions ? (
                  <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <Eye className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {showHiddenOptions
                  ? t`Conceal Hidden Options`
                  : t`Reveal Hidden Options`}
              </DropdownMenuItem>
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() =>
                  setSortMode(
                    sortMode === OptionSortMode.Name
                      ? OptionSortMode.Balance
                      : OptionSortMode.Name,
                  )
                }
              >
                {sortMode === OptionSortMode.Name ? (
                  <ArrowDownAz className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <ArrowUpAz className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {sortMode === OptionSortMode.Name
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
