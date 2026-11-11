import { NftUpdaterKind } from '@/bindings';
import { isMutableNft, mutableNftExplanation } from '@/lib/nftUpdater';
import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { PencilLine } from 'lucide-react';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';

interface MutableNftBadgeProps {
  kind: NftUpdaterKind | null | undefined;
  className?: string;
}

/**
 * Marks NFTs whose metadata isn't append-only. Regular NFTs render nothing.
 */
export function MutableNftBadge({ kind, className }: MutableNftBadgeProps) {
  if (!kind || !isMutableNft(kind)) return null;

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            className={cn(
              'inline-flex items-center gap-1 rounded-full bg-amber-500/90 px-1.5 py-0.5 text-xs font-medium text-white shadow-sm',
              className,
            )}
            aria-label={t`Mutable NFT`}
            onClick={(e) => e.stopPropagation()}
          >
            <PencilLine className='h-3 w-3' aria-hidden='true' />
            <Trans>Mutable</Trans>
          </span>
        </TooltipTrigger>
        <TooltipContent className='max-w-xs'>
          <p>{mutableNftExplanation(kind)}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
