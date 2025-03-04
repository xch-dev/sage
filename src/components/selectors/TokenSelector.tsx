import { CatRecord, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';
import { t } from '@lingui/core/macro';
import { isValidAssetId } from '@/lib/utils';

export interface TokenSelectorProps {
  value: string | null;
  onChange: (value: string) => void;
  disabled?: string[];
  className?: string;
  hideZeroBalance?: boolean;
}

export function TokenSelector({
  value,
  onChange,
  disabled = [],
  className,
  hideZeroBalance = false,
}: TokenSelectorProps) {
  const { addError } = useErrors();

  const [tokens, setTokens] = useState<CatRecord[]>([]);
  const [selectedToken, setSelectedToken] = useState<CatRecord | null>(null);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    commands
      .getCats({})
      .then((data) => {
        if (tokens.length) return;

        setTokens(data.cats);

        if (value && !selectedToken) {
          setSelectedToken(
            data.cats.find((token) => token.asset_id === value) ?? null,
          );
        }
      })
      .catch(addError);
  }, [addError, tokens.length, value, selectedToken]);

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
