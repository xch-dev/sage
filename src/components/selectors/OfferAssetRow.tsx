import { MutableNftBadge } from '@/components/MutableNftBadge';
import { Button } from '@/components/ui/button';
import { NftLookup } from '@/hooks/useRequestedNft';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ImageIcon, Loader2, TrashIcon } from 'lucide-react';
import { ReactNode } from 'react';

export interface OfferAssetRowProps {
  /** The inputs, laid out left to right and separated by dividers. */
  children: ReactNode;
  onRemove: () => void;
  removeLabel: string;
  /** Shown inside the frame, below the inputs. */
  footer?: ReactNode;
  tone?: 'error' | 'warning';
}

/** One asset in an offer: its inputs in a single bordered frame. */
export function OfferAssetRow({
  children,
  onRemove,
  removeLabel,
  footer,
  tone,
}: OfferAssetRowProps) {
  return (
    <div
      className={cn(
        'flex min-w-0 flex-col overflow-hidden rounded-lg border border-input bg-input-background transition-colors focus-within:ring-1 focus-within:ring-ring',
        tone === 'error' && 'border-destructive',
        tone === 'warning' && 'border-amber-500',
      )}
    >
      <div className='flex min-w-0 items-stretch divide-x divide-input'>
        {children}
        <Button
          type='button'
          variant='ghost'
          className='h-12 shrink-0 rounded-none px-3'
          onClick={onRemove}
          aria-label={removeLabel}
        >
          <TrashIcon className='h-4 w-4' aria-hidden='true' />
        </Button>
      </div>
      {footer}
    </div>
  );
}

export interface OfferAssetRowFooterProps {
  children: ReactNode;
  tone?: 'error' | 'warning';
  className?: string;
}

/** A thin strip along the bottom of an `OfferAssetRow`. */
export function OfferAssetRowFooter({
  children,
  tone,
  className,
}: OfferAssetRowFooterProps) {
  return (
    <div
      className={cn(
        'flex min-h-7 items-center gap-2 border-t border-input bg-muted/50 px-3 py-1 text-xs text-muted-foreground',
        tone === 'error' && 'text-destructive',
        tone === 'warning' && 'text-amber-600 dark:text-amber-500',
        className,
      )}
    >
      {children}
    </div>
  );
}

export interface RequestedNftPreviewProps {
  lookup: NftLookup;
}

/** What the wallet found for a requested NFT ID, shown under its row. */
export function RequestedNftPreview({ lookup }: RequestedNftPreviewProps) {
  switch (lookup.status) {
    case 'idle':
      return null;
    case 'invalid':
      return (
        <OfferAssetRowFooter tone='error'>
          <Trans>This isn&apos;t a valid NFT ID.</Trans>
        </OfferAssetRowFooter>
      );
    case 'loading':
      return (
        <OfferAssetRowFooter>
          <Loader2 className='h-3 w-3 animate-spin' aria-hidden='true' />
          <Trans>Looking up NFT…</Trans>
        </OfferAssetRowFooter>
      );
    case 'missing':
      return (
        <OfferAssetRowFooter tone='warning'>
          <Trans>
            Couldn&apos;t find this NFT. Check the ID, or try again once
            you&apos;re connected to a peer.
          </Trans>
        </OfferAssetRowFooter>
      );
    case 'found':
      return (
        <OfferAssetRowFooter className='gap-3 py-2 text-foreground'>
          {lookup.thumbnail ? (
            <img
              src={lookup.thumbnail}
              alt=''
              className='h-10 w-10 shrink-0 rounded-md object-cover'
            />
          ) : (
            <span className='flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-muted'>
              <ImageIcon
                className='h-5 w-5 text-muted-foreground'
                aria-hidden='true'
              />
            </span>
          )}
          <div className='flex min-w-0 flex-col'>
            <span className='truncate text-sm font-medium'>
              {lookup.name ?? t`NFT found`}
            </span>
            <MutableNftBadge kind={lookup.kind} className='w-fit' />
          </div>
        </OfferAssetRowFooter>
      );
  }
}
