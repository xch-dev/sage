import { Amount, Asset, OptionRecord, commands } from '@/bindings';
import { fromMojos, getAssetDisplayName, isValidAddress } from '@/lib/utils';
import { plural, t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { FilePenLine } from 'lucide-react';
import { ReactNode, useCallback } from 'react';
import { MultiAssetPicker } from './MultiAssetPicker';

export interface OptionPickerProps {
  /** Options already in the offer, shown checked. */
  selected: string[];
  /** Adds the option to the offer, or removes it if it's already there. */
  onToggle: (optionId: string) => void;
  /** The button that opens the picker. */
  children: ReactNode;
}

/** Picks unexpired options from the wallet, several at a time. */
export function OptionPicker({
  selected,
  onToggle,
  children,
}: OptionPickerProps) {
  const load = useCallback(async (query: string, limit: number) => {
    if (isValidAddress(query, 'option')) {
      const { option } = await commands.getOption({ option_id: query });
      return {
        // The lookup also finds options the wallet once held, so check it
        // still does, as the list below only has owned options.
        items: option && option.is_owned && !isExpired(option) ? [option] : [],
        hasMore: false,
      };
    }

    // Options can't be searched by the backend, so filter them here.
    const { options } = await commands.getOptions({
      offset: 0,
      limit: 1000,
      find_value: null,
      include_hidden: true,
    });
    const matches = options
      .filter((option) => !isExpired(option) && matchesSearch(option, query))
      .sort((a, b) => (a.name ?? '').localeCompare(b.name ?? ''));

    return {
      items: matches.slice(0, limit),
      hasMore: matches.length > limit,
    };
  }, []);

  return (
    <MultiAssetPicker<OptionRecord>
      selected={selected}
      onToggle={onToggle}
      load={load}
      getId={(option) => option.launcher_id}
      searchPlaceholder={t`Search options or paste an option ID`}
      isId={(query) => isValidAddress(query, 'option')}
      idNotFoundMessage={
        <Trans>This option isn&apos;t in your wallet, or has expired.</Trans>
      }
      emptyMessage={<Trans>No options found.</Trans>}
      countLabel={(count) =>
        plural(count, { one: '# option added', other: '# options added' })
      }
      renderItem={(option) => <OptionSummary option={option} />}
    >
      {children}
    </MultiAssetPicker>
  );
}

/** An option's icon, name and terms, for pickers and offer rows. */
export function OptionSummary({ option }: { option: OptionRecord | null }) {
  return (
    <>
      <span className='flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-muted'>
        <FilePenLine className='h-4 w-4' aria-hidden='true' />
      </span>
      <span className='flex min-w-0 flex-1 flex-col'>
        <span className='truncate text-sm'>
          {option?.name ?? t`Unnamed option`}
        </span>
        {option && (
          <span className='truncate text-xs text-muted-foreground'>
            <OptionTerms option={option} />
          </span>
        )}
      </span>
    </>
  );
}

function OptionTerms({ option }: { option: OptionRecord }) {
  const underlying = assetAmount(
    option.underlying_asset,
    option.underlying_amount,
  );
  const strike = assetAmount(option.strike_asset, option.strike_amount);
  const expires = new Date(option.expiration_seconds * 1000).toLocaleDateString(
    undefined,
    { dateStyle: 'medium' },
  );

  return (
    <Trans>
      {underlying} for {strike}, expires {expires}
    </Trans>
  );
}

function assetAmount(asset: Asset, amount: Amount) {
  const name = getAssetDisplayName(asset.name, asset.ticker, asset.kind);
  if (asset.kind !== 'token') return name;
  const value = fromMojos(amount, asset.precision).toFormat();
  return `${value} ${asset.ticker ?? name}`;
}

function isExpired(option: OptionRecord) {
  return option.expiration_seconds * 1000 < Date.now();
}

function matchesSearch(option: OptionRecord, query: string) {
  if (!query) return true;
  const q = query.toLowerCase();
  return [
    option.name,
    option.underlying_asset.name,
    option.underlying_asset.ticker,
    option.strike_asset.name,
    option.strike_asset.ticker,
  ].some((value) => value?.toLowerCase().includes(q));
}
