import { Trans } from '@lingui/react/macro';
import { TriangleAlert } from 'lucide-react';

interface MutableNftWarningProps {
  /** Labels for the mutable NFTs, from either side of an offer, including their IDs. */
  labels: string[];
  className?: string;
}

/**
 * Warns that an offer contains NFTs whose owner can replace their metadata, so what the
 * NFT looks like now may not be what is received.
 */
export function MutableNftWarning({
  labels,
  className,
}: MutableNftWarningProps) {
  if (labels.length === 0) return null;

  return (
    <div
      className={`flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-600 ${className ?? ''}`}
      role='alert'
    >
      <TriangleAlert
        className='mt-0.5 h-4 w-4 flex-shrink-0'
        aria-hidden='true'
      />
      <div className='flex flex-col gap-1'>
        <span className='font-medium'>
          {labels.length === 1 ? (
            <Trans>This offer contains a mutable NFT</Trans>
          ) : (
            <Trans>This offer contains mutable NFTs</Trans>
          )}
        </span>
        <span>
          <Trans>
            The owner of a mutable NFT can replace its metadata, including its
            name and image, at any time. Don&apos;t rely on the name or image to
            identify it; verify the NFT ID instead.
          </Trans>
        </span>
        <span className='break-words font-medium'>{labels.join(', ')}</span>
      </div>
    </div>
  );
}
