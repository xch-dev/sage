import { commands, OptionRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useMemo, useRef, useState } from 'react';
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
  const [options, setOptions] = useState<Record<string, OptionRecord>>({});
  const [pageOptionIds, setPageOptionIds] = useState<string[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const pageSize = 8;

  // Restore focus after option list updates
  useEffect(() => {
    if (searchTerm && inputRef.current) {
      inputRef.current.focus();
    }
  }, [options, searchTerm]);

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
              setPageOptionIds([option.launcher_id]);
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
            setPageOptionIds(
              data.options
                .filter(
                  (option) => option.expiration_seconds * 1000 >= Date.now(),
                )
                .map((option) => option.launcher_id),
            );
          })
          .catch(addError);
      }

      setOptions(
        Object.fromEntries(
          Object.entries(options).filter(
            ([, option]) => option.expiration_seconds * 1000 >= Date.now(),
          ),
        ),
      );
    };

    fetchOptions();
  }, [addError, page, searchTerm, isValidOptionId, value]);

  const filteredOptionIds = useMemo(() => {
    return pageOptionIds
      .filter((optionId) => {
        const option = options[optionId];

        if (!option) return false;

        // Filter out expired options
        return (
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
            option.strike_asset.asset_id === searchTerm)
        );
      })
      .sort((aId, bId) => {
        const a = options[aId];
        const b = options[bId];

        if (!a || !b) return 0;

        return (
          (a.name ?? '').localeCompare(b.name ?? '') ||
          (a.underlying_asset.name ?? '').localeCompare(
            b.underlying_asset.name ?? '',
          ) ||
          (a.strike_asset.name ?? '').localeCompare(b.strike_asset.name ?? '')
        );
      })

      .slice(page * pageSize, (page + 1) * pageSize);
  }, [options, searchTerm, page, pageOptionIds]);

  return (
    <DropdownSelector
      loadedItems={filteredOptionIds}
      page={page}
      setPage={setPage}
      value={value || undefined}
      setValue={(optionId) => {
        onChange(optionId);
        // Only clear search term if it's not a valid Option ID (i.e., user clicked on an item from the list)
        if (!isValidAddress(searchTerm, 'option')) {
          setSearchTerm('');
        }
      }}
      isDisabled={(optionId) => disabled.includes(optionId)}
      className={className}
      manualInput={
        <Input
          ref={inputRef}
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
      renderItem={(optionId) => (
        <div className='flex items-center gap-2 w-full'>
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate' role='text'>
              {options[optionId]?.name ?? 'Unknown Option'}
            </span>
            <span
              className='text-xs text-muted-foreground truncate'
              aria-label='Option ID'
            >
              {optionId}
            </span>
          </div>
        </div>
      )}
    />
  );
}
