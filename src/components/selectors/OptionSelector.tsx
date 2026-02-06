import { commands, OptionRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { SearchableSelect } from './SearchableSelect';

export interface OptionSelectorProps {
  value: string | null;
  onChange: (value: string) => void;
  disabled?: string[];
  className?: string;
}

export function OptionSelector({
  value,
  onChange,
  disabled = [],
  className,
}: OptionSelectorProps) {
  const { addError } = useErrors();

  const [page, setPage] = useState(0);
  const [options, setOptions] = useState<Record<string, OptionRecord>>({});
  const [searchTerm, setSearchTerm] = useState('');

  const pageSize = 8;

  const isValidOptionId = useMemo(() => {
    return isValidAddress(searchTerm, 'option');
  }, [searchTerm]);

  useEffect(() => {
    const fetchOptions = async () => {
      const options: Record<string, OptionRecord> = {};

      if (value) {
        await commands.getOption({ option_id: value }).then(({ option }) => {
          if (option) options[option.launcher_id] = option;
        });
      }

      if (isValidOptionId && searchTerm) {
        await commands
          .getOption({ option_id: searchTerm })
          .then(({ option }) => {
            if (option) {
              options[option.launcher_id] = option;
            }
          });
      } else {
        await commands
          .getOptions({
            offset: 0,
            limit: 1000, // Large limit to get all options for selector
            find_value: null,
            include_hidden: true,
          })
          .then((data) => {
            for (const option of data.options) {
              options[option.launcher_id] = option;
            }
          })
          .catch(addError);
      }

      // Filter out expired options
      setOptions(
        Object.fromEntries(
          Object.entries(options).filter(
            ([, option]) => option.expiration_seconds * 1000 >= Date.now(),
          ),
        ),
      );
    };

    fetchOptions();
  }, [addError, searchTerm, isValidOptionId, value]);

  // Filter and sort options based on search term
  const filteredOptions = useMemo(() => {
    return Object.values(options)
      .filter((option) => {
        // Filter out expired options
        if (option.expiration_seconds * 1000 < Date.now()) return false;

        if (!searchTerm) return true;

        // Match by launcher ID, name, underlying asset, or strike asset
        return (
          option.launcher_id === searchTerm ||
          option.name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
          option.underlying_asset.name
            ?.toLowerCase()
            .includes(searchTerm.toLowerCase()) ||
          option.underlying_asset.ticker
            ?.toLowerCase()
            .includes(searchTerm.toLowerCase()) ||
          option.underlying_asset.asset_id === searchTerm ||
          option.strike_asset.name
            ?.toLowerCase()
            .includes(searchTerm.toLowerCase()) ||
          option.strike_asset.ticker
            ?.toLowerCase()
            .includes(searchTerm.toLowerCase()) ||
          option.strike_asset.asset_id === searchTerm
        );
      })
      .sort((a, b) => {
        return (
          (a.name ?? '').localeCompare(b.name ?? '') ||
          (a.underlying_asset.name ?? '').localeCompare(
            b.underlying_asset.name ?? '',
          ) ||
          (a.strike_asset.name ?? '').localeCompare(b.strike_asset.name ?? '')
        );
      });
  }, [options, searchTerm]);

  const paginatedOptions = useMemo(() => {
    const start = page * pageSize;
    return filteredOptions.slice(start, start + pageSize);
  }, [filteredOptions, page, pageSize]);

  const handleSelect = useCallback(
    (optionId: string | null) => {
      if (optionId) {
        onChange(optionId);
      }
    },
    [onChange],
  );

  const handleManualInput = useCallback(
    (optionId: string) => {
      onChange(optionId);
    },
    [onChange],
  );

  const handleSearchChange = useCallback(
    (search: string) => {
      setSearchTerm(search);
      if (page !== 0) {
        setPage(0);
      }
    },
    [page],
  );

  const validateOptionId = useCallback((value: string) => {
    return isValidAddress(value, 'option');
  }, []);

  const renderOption = useCallback(
    (option: OptionRecord) => (
      <div className='flex items-center gap-2 min-w-0'>
        <div className='flex flex-col min-w-0'>
          <span className='truncate' role='text'>
            {option.name ?? 'Unknown Option'}
          </span>
          <span
            className='text-xs text-muted-foreground truncate'
            aria-label='Option ID'
          >
            {option.launcher_id}
          </span>
        </div>
      </div>
    ),
    [],
  );

  return (
    <SearchableSelect
      value={value || undefined}
      onSelect={handleSelect}
      items={paginatedOptions}
      getItemId={(opt) => opt.launcher_id}
      renderItem={renderOption}
      onSearchChange={handleSearchChange}
      shouldFilter={false}
      validateManualInput={validateOptionId}
      onManualInput={handleManualInput}
      page={page}
      onPageChange={setPage}
      pageSize={pageSize}
      hasMorePages={filteredOptions.length > (page + 1) * pageSize}
      disabled={disabled}
      className={className}
      placeholder={t`Select option`}
      searchPlaceholder={t`Search by name or enter Option ID`}
      emptyMessage={t`No options found.`}
    />
  );
}
