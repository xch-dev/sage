import { CatRecord, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useEffect, useState } from 'react';
import { Input } from '../ui/input';
import { DropdownSelector } from './DropdownSelector';

export interface TokenSelectorProps {
  value: string | null;
  onChange: (value: string) => void;
  disabled?: string[];
  className?: string;
}

export function TokenSelector({
  value,
  onChange,
  disabled = [],
  className,
}: TokenSelectorProps) {
  const { addError } = useErrors();

  const [tokens, setTokens] = useState<CatRecord[]>([]);
  const [selectedToken, setSelectedToken] = useState<CatRecord | null>(null);

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

  return (
    <DropdownSelector
      totalItems={tokens.length}
      loadedItems={tokens}
      page={0}
      isDisabled={(token) => disabled.includes(token.asset_id)}
      onSelect={(token) => {
        onChange(token.asset_id);
        setSelectedToken(token);
      }}
      className={className}
      manualInput={
        <Input
          placeholder='Enter asset id'
          value={value || ''}
          onChange={(e) => {
            onChange(e.target.value);
            setSelectedToken(
              tokens.find((token) => token.asset_id === e.target.value) ?? {
                name: 'Unknown',
                asset_id: e.target.value,
                icon_url: null,
                balance: 0,
                ticker: null,
                description: null,
                visible: true,
              },
            );
          }}
        />
      }
      renderItem={(token) => (
        <div className='flex items-center gap-2 w-full'>
          {token.icon_url && (
            <img
              src={token.icon_url}
              className='w-10 h-10 rounded object-cover'
              alt={token.name ?? 'Unknown'}
            />
          )}
          <div className='flex flex-col truncate'>
            <span className='flex-grow truncate'>{token.name}</span>
            <span className='text-xs text-muted-foreground truncate'>
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
          />
        )}
        <div className='flex flex-col truncate text-left'>
          <span className='truncate'>
            {selectedToken?.name ?? 'Select Token'}
          </span>
          <span className='text-xs text-muted-foreground truncate'>
            {selectedToken?.asset_id}
          </span>
        </div>
      </div>
    </DropdownSelector>
  );
}
