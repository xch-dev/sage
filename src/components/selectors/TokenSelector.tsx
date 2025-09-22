import { TokenRecord, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { getAssetDisplayName, isValidAssetId } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useRef, useState } from 'react';
import { AssetIcon } from '../AssetIcon';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';

export interface TokenSelectorProps {
  value: string | null | undefined;
  onChange: (value: string | null) => void;
  disabled?: (string | null)[];
  className?: string;
  hideZeroBalance?: boolean;
  showAllCats?: boolean;
  includeXch?: boolean;
}

export function TokenSelector({
  value,
  onChange,
  disabled = [],
  className,
  hideZeroBalance = false,
  showAllCats = false,
  includeXch = false,
}: TokenSelectorProps) {
  const { addError } = useErrors();

  const [tokens, setTokens] = useState<Record<string, TokenRecord>>({});
  const [searchTerm, setSearchTerm] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // Restore focus after token list updates
  useEffect(() => {
    if (searchTerm && inputRef.current) {
      inputRef.current.focus();
    }
  }, [tokens, searchTerm]);

  useEffect(() => {
    const fetchTokens = async () => {
      const tokens: Record<string, TokenRecord> = {};

      const getCats = showAllCats
        ? commands.getAllCats({})
        : commands.getCats({});

      await getCats
        .then(async (data) => {
          // Sort by name (nulls last)
          data.cats.sort((a, b) => {
            if (!a.name && !b.name) return 0;
            if (!a.name) return 1;
            if (!b.name) return -1;
            return a.name.localeCompare(b.name);
          });

          if (includeXch) {
            const xch = await commands.getToken({ asset_id: null });
            if (xch.token) data.cats.unshift(xch.token);
          }

          for (const cat of data.cats) {
            tokens[cat.asset_id ?? 'xch'] = cat;
          }
        })
        .catch(addError);

      setTokens(tokens);
    };

    fetchTokens();
  }, [addError, includeXch, showAllCats]);

  // Filter tokens based on search term or show all if it's a valid asset ID
  const filteredTokenIds = Object.values(tokens)
    .filter((token) => {
      if (!token.visible) return false;
      if (hideZeroBalance && token.balance === 0) return false;
      if (!searchTerm) return true;

      if (isValidAssetId(searchTerm)) {
        return token.asset_id?.toLowerCase() === searchTerm.toLowerCase();
      }

      // Search by name and ticker
      return (
        token.name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        token.ticker?.toLowerCase().includes(searchTerm.toLowerCase())
      );
    })
    .map((token) => token.asset_id ?? 'xch');

  return (
    <DropdownSelector
      loadedItems={filteredTokenIds}
      page={0}
      value={value === null ? 'xch' : value}
      setValue={(assetId) => {
        onChange(assetId === 'xch' ? null : assetId);
        setSearchTerm('');
      }}
      isDisabled={(token) => disabled.includes(token)}
      className={className}
      manualInput={
        <Input
          ref={inputRef}
          placeholder={t`Search or enter asset id`}
          value={searchTerm}
          onChange={(e) => {
            const newValue = e.target.value;
            setSearchTerm(newValue);

            if (/^[a-fA-F0-9]{64}$/.test(newValue)) {
              onChange(newValue);
            }
          }}
        />
      }
      renderItem={(assetId) => (
        <div className='flex items-center gap-2 w-full'>
          <AssetIcon
            asset={{
              icon_url: tokens[assetId]?.icon_url ?? null,
              kind: 'token',
              revocation_address: null,
              // TODO: Use Asset here and use the actual revocation address
            }}
            size='md'
            className='flex-shrink-0'
          />
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate' role='text'>
              {getAssetDisplayName(
                tokens[assetId]?.name,
                tokens[assetId]?.ticker,
                'token',
              )}
              {tokens[assetId]?.ticker && ` (${tokens[assetId]?.ticker})`}{' '}
            </span>
            <span
              className='text-xs text-muted-foreground truncate'
              aria-label={t`Asset ID`}
            >
              {assetId === 'xch' ? null : assetId}
            </span>
          </div>
        </div>
      )}
    />
  );
}
