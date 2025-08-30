import { commands, OptionRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useMemo, useState } from 'react';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';

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
  const [options, setOptions] = useState<OptionRecord[]>([]);
  const [selectedOption, setSelectedOption] = useState<OptionRecord | null>(
    null,
  );
  const [searchTerm, setSearchTerm] = useState('');

  const pageSize = 8;

  const isValidOptionId = useMemo(() => {
    return isValidAddress(searchTerm, 'option');
  }, [searchTerm]);

  useEffect(() => {
    commands
      .getOptions({
        offset: 0,
        limit: 1000, // Large limit to get all options for selector
        find_value: null,
        include_hidden: true,
      })
      .then((data) => setOptions(data.options))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    if (isValidOptionId) {
      commands
        .getOption({ option_id: searchTerm })
        .then((data) => {
          // Only set the option if it's not expired
          if (
            data.option &&
            data.option.expiration_seconds * 1000 >= Date.now()
          ) {
            setSelectedOption(data.option);
            onChange(searchTerm);
          } else {
            // Clear the selection if the option is expired
            setSelectedOption(null);
            onChange('');
          }
        })
        .catch(addError);
    }
  }, [isValidOptionId, searchTerm, onChange, addError]);

  // Load option record when a value is provided but not found in current options list
  useEffect(() => {
    if (
      value &&
      value !== '' &&
      !selectedOption &&
      !options.find((option) => option.launcher_id === value)
    ) {
      try {
        // Validate the Option ID format
        if (isValidAddress(value, 'option')) {
          commands
            .getOption({ option_id: value })
            .then((data) => {
              // Only set the option if it's not expired
              if (
                data.option &&
                data.option.expiration_seconds * 1000 >= Date.now()
              ) {
                setSelectedOption(data.option);
              } else {
                // Clear the selection if the option is expired
                setSelectedOption(null);
              }
            })
            .catch(addError);
        }
      } catch {
        // Handle any errors silently
      }
    }
  }, [value, selectedOption, options, addError]);

  // Reset selectedOption when value is null or empty
  useEffect(() => {
    if (!value || value === '') {
      setSelectedOption(null);
    }
  }, [value]);

  const filteredOptions = useMemo(() => {
    return options
      .filter(
        (option) =>
          // Filter out expired options
          option.expiration_seconds * 1000 >= Date.now() &&
          (option.launcher_id === searchTerm ||
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
            option.strike_asset.asset_id === searchTerm),
      )
      .sort(
        (a, b) =>
          (a.name ?? '').localeCompare(b.name ?? '') ||
          (a.underlying_asset.name ?? '').localeCompare(
            b.underlying_asset.name ?? '',
          ) ||
          (a.strike_asset.name ?? '').localeCompare(b.strike_asset.name ?? ''),
      )
      .slice(page * pageSize, (page + 1) * pageSize);
  }, [options, searchTerm, page]);

  return (
    <DropdownSelector
      loadedItems={filteredOptions}
      page={page}
      setPage={setPage}
      isDisabled={(option) => disabled.includes(option.launcher_id)}
      onSelect={(option) => {
        onChange(option.launcher_id);
        setSelectedOption(option);
        // Only clear search term if it's not a valid Option ID (i.e., user clicked on an item from the list)
        if (!isValidAddress(searchTerm, 'option')) {
          setSearchTerm('');
        }
      }}
      className={className}
      manualInput={
        <Input
          placeholder={t`Search by name or enter Option ID`}
          value={searchTerm}
          onChange={(e) => {
            const newValue = e.target.value;
            setSearchTerm(newValue);

            if (isValidAddress(newValue, 'option')) {
              onChange(newValue);
            }
          }}
        />
      }
      renderItem={(option) => (
        <div className='flex items-center gap-2 w-full'>
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate' role='text'>
              {option.name}
            </span>
            <span
              className='text-xs text-muted-foreground truncate'
              aria-label='Option ID'
            >
              {option.launcher_id}
            </span>
          </div>
        </div>
      )}
    >
      <div className='flex items-center gap-2 min-w-0'>
        <div className='flex flex-col truncate text-left'>
          <span className='truncate' role='text'>
            {selectedOption
              ? (selectedOption.name ?? t`Untitled Option`)
              : t`Select Option`}
          </span>
          <span
            className='text-xs text-muted-foreground truncate'
            aria-label='Option ID'
          >
            {selectedOption?.launcher_id}
          </span>
        </div>
      </div>
    </DropdownSelector>
  );
}
