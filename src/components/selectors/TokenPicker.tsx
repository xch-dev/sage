import { TokenRecord } from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import { NumberFormat } from '@/components/NumberFormat';
import { Button } from '@/components/ui/button';
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
  cn,
  formatAddress,
  fromMojos,
  getAssetDisplayName,
  isValidAssetId,
} from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CheckIcon, ClipboardPaste } from 'lucide-react';
import { ReactNode, useMemo, useState } from 'react';

// Every CAT on Dexie can be requested, so the list is cut short until searched.
const pageSize = 50;

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
  const [limit, setLimit] = useState(pageSize);

  const query = search.trim().toLowerCase();
  const pastedAssetId = isValidAssetId(query) ? query : null;

  const matches = useMemo(() => {
    const ranked = tokens
      .filter((token) => token.visible)
      .filter((token) => !offering || Number(token.balance) !== 0)
      .map((token) => ({
        token,
        rank: pastedAssetId
          ? token.asset_id === pastedAssetId
            ? 0
            : null
          : searchRank(token, query),
      }))
      .filter(
        (match): match is { token: TokenRecord; rank: number } =>
          match.rank !== null,
      );

    // Best matches first, then XCH and the tokens you hold. Otherwise the
    // order is kept, which is alphabetical.
    return ranked
      .sort(
        (a, b) =>
          a.rank - b.rank ||
          listOrder(a.token, offering) - listOrder(b.token, offering),
      )
      .map((match) => match.token);
  }, [tokens, offering, query, pastedAssetId]);

  const items = matches.slice(0, limit);

  // Any CAT can be requested by its asset ID, even one the wallet hasn't seen.
  const canUsePastedId =
    !offering &&
    pastedAssetId !== null &&
    matches.length === 0 &&
    !taken.includes(pastedAssetId);

  const handleOpenChange = (next: boolean) => {
    setOpen(next);
    if (next) onOpen?.();
    else {
      setSearch('');
      setLimit(pageSize);
    }
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
            onValueChange={(value) => {
              setSearch(value);
              setLimit(pageSize);
            }}
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
            {matches.length > items.length && (
              <div className='p-1'>
                <Button
                  type='button'
                  variant='ghost'
                  size='sm'
                  className='w-full'
                  onClick={() => setLimit(limit + pageSize)}
                >
                  <Trans>Show more</Trans>
                </Button>
              </div>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

/** How well a token matches the search, lower being better, or null if not. */
function searchRank(token: TokenRecord, query: string): number | null {
  if (!query) return 0;
  const ticker = token.ticker?.toLowerCase() ?? '';
  const name = token.name?.toLowerCase() ?? '';
  if (ticker === query) return 0;
  if (ticker.startsWith(query) || name.startsWith(query)) return 1;
  if (ticker.includes(query) || name.includes(query)) return 2;
  if (token.asset_id?.startsWith(query)) return 3;
  return null;
}

/** XCH first, then tokens with a balance, when requesting. */
function listOrder(token: TokenRecord, offering?: boolean) {
  if (token.asset_id === null) return 0;
  if (!offering && Number(token.balance) !== 0) return 1;
  return 2;
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
      className={cn(
        'flex h-8 w-8 shrink-0 items-center justify-center',
        className,
      )}
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
