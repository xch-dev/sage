import { TokenRecord, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { isValidAssetId } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';
import { AssetIcon } from '../AssetIcon';

export interface TokenSelectorProps {
  value: string | null;
  onChange: (value: string) => void;
  disabled?: string[];
  className?: string;
  hideZeroBalance?: boolean;
  showAllCats?: boolean;
}

export function TokenSelector({
  value,
  onChange,
  disabled = [],
  className,
  hideZeroBalance = false,
  showAllCats = false,
}: TokenSelectorProps) {
  const { addError } = useErrors();

  const [tokens, setTokens] = useState<TokenRecord[]>([]);
  const [selectedToken, setSelectedToken] = useState<TokenRecord | null>(null);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    let isMounted = true;
    const command = showAllCats
      ? commands.getAllCats({})
      : commands.getCats({});

    command
      .then((data) => {
        if (!isMounted) return;

        const allTokens = data.cats;

        // Sort by name (nulls last)
        allTokens.sort((a, b) => {
          if (!a.name && !b.name) return 0;
          if (!a.name) return 1;
          if (!b.name) return -1;
          return a.name.localeCompare(b.name);
        });

        setTokens(allTokens);

        if (value && !selectedToken) {
          setSelectedToken(
            allTokens.find((token) => token.asset_id === value) ?? null,
          );
        }
      })
      .catch(addError);
    return () => {
      isMounted = false;
    };
  }, [addError, tokens.length, value, selectedToken, showAllCats]);

  // Filter tokens based on search term or show all if it's a valid asset ID
  const filteredTokens = tokens.filter((token) => {
    if (!token.visible) return false;
    if (hideZeroBalance && token.balance === 0) return false;
    if (!searchTerm) return true;

    if (isValidAssetId(searchTerm)) {
      return token.asset_id.toLowerCase() === searchTerm.toLowerCase();
    }

    // Search by name and ticker
    return (
      token.name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      token.ticker?.toLowerCase().includes(searchTerm.toLowerCase())
    );
  });

  return (
    <DropdownSelector
      loadedItems={filteredTokens}
      page={0}
      isDisabled={(token) => disabled.includes(token.asset_id)}
      onSelect={(token) => {
        onChange(token.asset_id);
        setSelectedToken(token);
        setSearchTerm('');
      }}
      className={className}
      manualInput={
        <Input
          placeholder={t`Search or enter asset id`}
          value={searchTerm}
          onChange={(e) => {
            const newValue = e.target.value;
            setSearchTerm(newValue);

            if (/^[a-fA-F0-9]{64}$/.test(newValue)) {
              onChange(newValue);
              setSelectedToken(
                tokens.find((token) => token.asset_id === newValue) ?? {
                  name: 'Unknown',
                  asset_id: newValue,
                  icon_url: null,
                  balance: 0,
                  ticker: null,
                  description: null,
                  visible: false,
                  precision: 0,
                  is_xch: false,
                },
              );
            }
          }}
        />
      }
      renderItem={(token) => (
        <div className='flex items-center gap-2 w-full'>
          <AssetIcon
            iconUrl={token.icon_url}
            name={token.name}
            kind='token'
            size='lg'
          />
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate' role='text'>
              {token.name}
              {token.ticker && ` (${token.ticker})`}
            </span>
            <span
              className='text-xs text-muted-foreground truncate'
              aria-label='Asset ID'
            >
              {token.asset_id}
            </span>
          </div>
        </div>
      )}
    >
      <div className='flex items-center gap-2 min-w-0'>
        <AssetIcon
          iconUrl={selectedToken?.icon_url}
          name={selectedToken?.name}
          kind='token'
        />
        <div className='flex flex-col truncate text-left'>
          <span className='truncate'>
            {selectedToken?.name ?? t`Select Token`}
            {selectedToken?.ticker && ` (${selectedToken.ticker})`}
          </span>
          <span className='text-xs text-muted-foreground truncate'>
            {selectedToken?.asset_id}
          </span>
        </div>
      </div>
    </DropdownSelector>
  );
}
