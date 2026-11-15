import { TokenRecord } from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import { NumberFormat } from '@/components/NumberFormat';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import {
  formatAddress,
  fromMojos,
  getAssetDisplayName,
  isValidAssetId,
} from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CheckIcon, ClipboardPaste } from 'lucide-react';
import { ReactNode, useMemo, useState } from 'react';

export interface TokenPickerProps {
  /** The tokens to choose from, XCH included. */
  tokens: TokenRecord[];
  /** Shows each token's balance, and leaves out tokens without one. */
  offering?: boolean;
  /** The token this picker is changing, if any. */
  value?: string | null;
  /** Tokens already in the offer, which can't be picked again. */
  taken: (string | null)[];
  onSelect: (assetId: string | null) => void;
  /** Called when the picker opens, so the token list can be refreshed. */
  onOpen?: () => void;
  align?: 'start' | 'end';
  /** The button that opens the picker. */
  children: ReactNode;
}

/** A searchable list of tokens that opens from a button. */
export function TokenPicker({
  tokens,
  offering,
  value,
  taken,
  onSelect,
  onOpen,
  align = 'start',
  children,
}: TokenPickerProps) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');

  const query = search.trim().toLowerCase();
  const pastedAssetId = isValidAssetId(query) ? query : null;

  const items = useMemo(
    () =>
      tokens
        .filter((token) => token.visible)
        .filter((token) => !offering || Number(token.balance) !== 0)
        .filter((token) => {
          if (!query) return true;
          if (pastedAssetId) return token.asset_id === pastedAssetId;
          return (
            token.name?.toLowerCase().includes(query) ||
            token.ticker?.toLowerCase().includes(query) ||
            token.asset_id?.startsWith(query)
          );
        }),
    [tokens, offering, query, pastedAssetId],
  );

  // Any CAT can be requested by its asset ID, even one the wallet hasn't seen.
  const canUsePastedId =
    !offering &&
    pastedAssetId !== null &&
    items.length === 0 &&
    !taken.includes(pastedAssetId);

  const handleOpenChange = (next: boolean) => {
    setOpen(next);
    if (next) onOpen?.();
    else setSearch('');
  };

  const select = (assetId: string | null) => {
    onSelect(assetId);
    handleOpenChange(false);
  };

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>{children}</PopoverTrigger>
      <PopoverContent
        className='w-[min(24rem,calc(100vw-2rem))] p-0'
        align={align}
      >
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={
              offering
                ? t`Search tokens`
                : t`Search tokens or paste an asset ID`
            }
            value={search}
            onValueChange={setSearch}
          />
          <CommandList className='max-h-80'>
            <CommandEmpty>
              {offering ? (
                <Trans>You don&apos;t have any tokens that match.</Trans>
              ) : (
                <Trans>
                  No tokens match. Paste an asset ID to request any CAT.
                </Trans>
              )}
            </CommandEmpty>
            {canUsePastedId && (
              <CommandGroup>
                <CommandItem
                  value={pastedAssetId}
                  onSelect={() => select(pastedAssetId)}
                  className='gap-3'
                >
                  <ClipboardPaste
                    className='h-8 w-8 shrink-0 p-1.5 text-muted-foreground'
                    aria-hidden='true'
                  />
                  <span className='flex min-w-0 flex-col'>
                    <span>
                      <Trans>Request this asset ID</Trans>
                    </span>
                    <span className='truncate font-mono text-xs text-muted-foreground'>
                      {formatAddress(pastedAssetId, 8)}
                    </span>
                  </span>
                </CommandItem>
              </CommandGroup>
            )}
            {items.length > 0 && (
              <CommandGroup>
                {items.map((token) => {
                  const id = token.asset_id ?? 'xch';
                  const isCurrent = token.asset_id === value;
                  const isTaken = !isCurrent && taken.includes(token.asset_id);

                  return (
                    <CommandItem
                      key={id}
                      value={id}
                      disabled={isTaken}
                      onSelect={() => select(token.asset_id)}
                      className='gap-3'
                    >
                      <TokenIcon token={token} />
                      <span className='flex min-w-0 flex-1 flex-col'>
                        <span className='truncate'>
                          {getAssetDisplayName(
                            token.name,
                            token.ticker,
                            'token',
                          )}
                          {token.ticker && token.name && (
                            <span className='ml-1.5 text-muted-foreground'>
                              {token.ticker}
                            </span>
                          )}
                        </span>
                        {token.asset_id && (
                          <span className='truncate font-mono text-xs text-muted-foreground'>
                            {formatAddress(token.asset_id, 6)}
                          </span>
                        )}
                      </span>
                      {offering && (
                        <span className='shrink-0 text-right text-xs tabular-nums text-muted-foreground'>
                          <NumberFormat
                            value={fromMojos(token.balance, token.precision)}
                            minimumFractionDigits={0}
                            maximumFractionDigits={token.precision}
                          />
                        </span>
                      )}
                      {isTaken && (
                        <span className='shrink-0 text-xs text-muted-foreground'>
                          <Trans>Added</Trans>
                        </span>
                      )}
                      {isCurrent && (
                        <CheckIcon
                          className='h-4 w-4 shrink-0'
                          aria-label={t`Selected`}
                        />
                      )}
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

export function TokenIcon({
  token,
  className,
}: {
  token: TokenRecord | undefined;
  className?: string;
}) {
  return (
    <span
      className={`flex h-8 w-8 shrink-0 items-center justify-center ${className ?? ''}`}
    >
      <AssetIcon
        asset={{
          icon_url: token?.icon_url ?? null,
          kind: 'token',
          revocation_address: token?.revocation_address ?? null,
        }}
        size='md'
      />
    </span>
  );
}
