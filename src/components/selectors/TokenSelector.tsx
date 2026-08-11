import { TokenRecord, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { getAssetDisplayName, isValidAssetId } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { AssetIcon } from '../AssetIcon';
import { SearchableSelect } from './SearchableSelect';

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

  // Filter tokens based on search term and visibility/balance settings
  const filteredTokens = useMemo(() => {
    return Object.values(tokens).filter((token) => {
      if (!token.visible) return false;
      if (hideZeroBalance && token.balance === 0) return false;
      if (!searchTerm) return true;
      if (isValidAssetId(searchTerm)) {
        return token.asset_id?.toLowerCase() === searchTerm.toLowerCase();
      }

      return (
        token.name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        token.ticker?.toLowerCase().includes(searchTerm.toLowerCase())
      );
    });
  }, [tokens, hideZeroBalance, searchTerm]);

  const handleSelect = useCallback(
    (assetId: string | null) => {
      // Convert 'xch' sentinel back to null
      onChange(assetId === 'xch' ? null : assetId);
    },
    [onChange],
  );

  const handleManualInput = useCallback(
    (assetId: string) => {
      onChange(assetId);
    },
    [onChange],
  );

  // Convert disabled array to handle null -> 'xch' conversion
  const disabledIds = useMemo(() => {
    return disabled.map((id) => (id === null ? 'xch' : id));
  }, [disabled]);

  const renderToken = useCallback(
    (token: TokenRecord) => (
      <div className='flex items-center gap-2 min-w-0'>
        <AssetIcon
          asset={{
            icon_url: token.icon_url ?? null,
            kind: 'token',
            revocation_address: token.revocation_address ?? null,
          }}
          size='md'
          className='flex-shrink-0'
        />
        <div className='flex flex-col min-w-0'>
          <span className='truncate' role='text'>
            {getAssetDisplayName(token.name, token.ticker, 'token')}
            {token.ticker && ` (${token.ticker})`}
          </span>
          <span
            className='text-xs text-muted-foreground truncate'
            aria-label={t`Asset ID`}
          >
            {token.asset_id === null ? null : token.asset_id}
          </span>
        </div>
      </div>
    ),
    [],
  );

  return (
    <SearchableSelect
      value={value === null ? 'xch' : value}
      onSelect={handleSelect}
      items={filteredTokens}
      getItemId={(token) => token.asset_id ?? 'xch'}
      renderItem={renderToken}
      onSearchChange={setSearchTerm}
      shouldFilter={false}
      validateManualInput={isValidAssetId}
      onManualInput={handleManualInput}
      disabled={disabledIds}
      className={className}
      placeholder={t`Select asset`}
      searchPlaceholder={t`Search or enter asset id`}
      emptyMessage={t`No tokens found.`}
    />
  );
}
