import { OfferHandleNft } from '@/bindings';
import { Trans } from '@lingui/react/macro';
import { ShieldAlert, TriangleAlert } from 'lucide-react';

/**
 * Whether an offer can't be taken because of a handle NFT in it. The backend
 * checks this again right before the offer is taken.
 */
export function isOfferHandleBlocked(handleNfts: OfferHandleNft[]): boolean {
  return handleNfts.some(isBlocking);
}

function isBlocking(nft: OfferHandleNft): boolean {
  return nft.status === 'pending_transfer' || nft.status === 'unavailable';
}

/**
 * Explains what the registry says about offered NFTs which claim to be
 * XCHandles handles. A pending transfer means the seller could take the handle
 * back after the trade.
 */
export function OfferHandleNftWarning({
  handleNfts,
  className,
}: {
  handleNfts: OfferHandleNft[];
  className?: string;
}) {
  // This is shown even when XCHandles is hidden, since the backend still blocks
  // taking the offer.
  const blocking = handleNfts.filter(isBlocking);
  const unrelated = handleNfts.filter((nft) => nft.status === 'not_associated');

  return (
    <>
      {blocking.length > 0 && (
        <div
          className={`flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive ${className ?? ''}`}
          role='alert'
        >
          <ShieldAlert
            className='mt-0.5 h-4 w-4 flex-shrink-0'
            aria-hidden='true'
          />
          <div className='flex flex-col gap-1'>
            <span className='font-medium'>
              <Trans>This offer can&apos;t be taken</Trans>
            </span>
            {blocking.map((nft) => {
              const handle = nft.handle;

              return (
                <span key={nft.nft_id} className='break-words'>
                  {nft.status === 'pending_transfer' ? (
                    <Trans>
                      A transfer of @{handle} is pending, so the seller could
                      take the handle back after the trade.
                    </Trans>
                  ) : (
                    <Trans>
                      @{handle} couldn&apos;t be checked for a pending transfer.
                      Try again later.
                    </Trans>
                  )}{' '}
                  {nft.reason}
                </span>
              );
            })}
          </div>
        </div>
      )}

      {unrelated.length > 0 && (
        <div
          className={`flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-600 ${className ?? ''}`}
          role='alert'
        >
          <TriangleAlert
            className='mt-0.5 h-4 w-4 flex-shrink-0'
            aria-hidden='true'
          />
          <div className='flex flex-col gap-1'>
            {unrelated.map((nft) => {
              const handle = nft.handle;

              return (
                <span key={nft.nft_id} className='break-words'>
                  <Trans>
                    An NFT in this offer is named @{handle}, but the XCHandles
                    registry doesn&apos;t list it as the owner of that handle.
                    Buying it won&apos;t give you the handle.
                  </Trans>
                </span>
              );
            })}
          </div>
        </div>
      )}
    </>
  );
}
