import { CatRecord, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';
import { t } from '@lingui/core/macro';
import { isValidAssetId } from '@/lib/utils';
import { usePrices } from '@/hooks/usePrices';

export interface TokenSelectorProps {
  value: string | null;
  onChange: (value: string) => void;
  disabled?: string[];
  className?: string;
  hideZeroBalance?: boolean;
  includeDexieList?: boolean;
}

export function TokenSelector({
  value,
  onChange,
  disabled = [],
  className,
  hideZeroBalance = false,
  includeDexieList = false,
}: TokenSelectorProps) {
  const { addError } = useErrors();
  const { getCatList } = usePrices();

  const [tokens, setTokens] = useState<CatRecord[]>([]);
  const [selectedToken, setSelectedToken] = useState<CatRecord | null>(null);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    let isMounted = true;
    commands
      .getCats({})
      .then((data) => {
        if (!isMounted) return;
        if (tokens.length) return;

        const walletTokens = data.cats;
        let allTokens = walletTokens;

        if (includeDexieList) {
          // Convert CatListItem[] to CatRecord[]
          const dexieTokens: CatRecord[] = getCatList().map((cat) => ({
            asset_id: cat.asset_id,
            name: cat.name,
            ticker: cat.ticker,
            description: null,
            icon_url: cat.icon_url,
            visible: true,
            balance: 0,
          }));

          // Merge and deduplicate by asset_id
          const tokenMap = new Map<string, CatRecord>();
          [...walletTokens, ...dexieTokens].forEach((token) => {
            if (!tokenMap.has(token.asset_id)) {
              tokenMap.set(token.asset_id, token);
            }
          });
          allTokens = Array.from(tokenMap.values());

          // Sort by name (nulls last)
          allTokens.sort((a, b) => {
            if (!a.name && !b.name) return 0;
            if (!a.name) return 1;
            if (!b.name) return -1;
            return a.name.localeCompare(b.name);
          });
        }

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
  }, [
    addError,
    tokens.length,
    value,
    selectedToken,
    includeDexieList,
    getCatList,
  ]);

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
                  visible: true,
                },
              );
            }
          }}
        />
      }
      renderItem={(token) => (
        <div className='flex items-center gap-2 w-full'>
          {token.icon_url && (
            <img
              src={token.icon_url}
              className='w-10 h-10 rounded object-cover'
              alt={token.name ?? t`Unknown token`}
              aria-hidden='true'
              loading='lazy'
            />
          )}
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
        {selectedToken?.icon_url && (
          <img
            src={selectedToken.icon_url}
            className='w-8 h-8 rounded object-cover'
            alt={
              selectedToken?.name
                ? `Image of ${selectedToken.name}`
                : 'No token name'
            }
            loading='lazy'
            aria-hidden='true'
          />
        )}
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
