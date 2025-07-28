import { commands } from '@/bindings';
import { useBiometric } from '@/hooks/useBiometric';
import { toMojos } from '@/lib/utils';
import { OfferState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { useCallback, useRef, useState } from 'react';

interface UseOfferProcessorProps {
  offerState: OfferState;
  splitNftOffers: boolean;
  onProcessingEnd?: () => void; // Callback for when offer processing (success or fail) is done
  onProgress?: (index: number) => void; // Callback for progress updates
}

interface UseOfferProcessorReturn {
  createdOffers: string[];
  isProcessing: boolean;
  processOffer: () => Promise<void>;
  clearProcessedOffers: () => void;
  cancelProcessing: () => void;
}

export function useOfferProcessor({
  offerState,
  splitNftOffers,
  onProcessingEnd,
  onProgress,
}: UseOfferProcessorProps): UseOfferProcessorReturn {
  const walletState = useWalletState();
  const { promptIfEnabled } = useBiometric();
  const [createdOffers, setCreatedOffers] = useState<string[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const isCancelled = useRef(false);

  const clearProcessedOffers = useCallback(() => {
    setCreatedOffers([]);
  }, []);

  const cancelProcessing = useCallback(() => {
    isCancelled.current = true;
    setIsProcessing(false);
    onProcessingEnd?.();
  }, [onProcessingEnd]);

  const processOffer = useCallback(async () => {
    setIsProcessing(true);
    isCancelled.current = false;
    setCreatedOffers([]); // Direct call instead of using clearProcessedOffers

    let expiresAtSecond: number | null = null;
    if (offerState.expiration !== null) {
      const days = parseInt(offerState.expiration.days) || 0;
      const hours = parseInt(offerState.expiration.hours) || 0;
      const minutes = parseInt(offerState.expiration.minutes) || 0;
      const totalSeconds = days * 24 * 60 * 60 + hours * 60 * 60 + minutes * 60;

      if (totalSeconds <= 0) {
        throw new Error(t`Expiration must be at least 1 second in the future`);
      }
      expiresAtSecond = Math.ceil(Date.now() / 1000) + totalSeconds;
    }

    if (!(await promptIfEnabled())) {
      throw new Error(t`Biometric authentication failed or was cancelled`);
    }

    try {
      if (
        splitNftOffers &&
        offerState.offered.nfts.filter((n) => n).length > 1
      ) {
        const newOffers: string[] = [];
        const nfts = offerState.offered.nfts.filter((n) => n);

        for (const [index, nft] of nfts.entries()) {
          if (isCancelled.current) {
            break;
          }

          onProgress?.(index);

          const data = await commands.makeOffer({
            offered_assets: {
              xch: toMojos(
                (offerState.offered.xch || '0').toString(),
                walletState.sync.unit.decimals,
              ),
              cats: offerState.offered.cats.map((cat) => ({
                asset_id: cat.asset_id,
                amount: toMojos((cat.amount || '0').toString(), 3),
              })),
              nfts: [nft],
            },
            requested_assets: {
              xch: toMojos(
                (offerState.requested.xch || '0').toString(),
                walletState.sync.unit.decimals,
              ),
              cats: offerState.requested.cats.map((cat) => ({
                asset_id: cat.asset_id,
                amount: toMojos((cat.amount || '0').toString(), 3),
              })),
              nfts: offerState.requested.nfts.filter((n) => n),
            },
            fee: toMojos(
              (offerState.fee || '0').toString(),
              walletState.sync.unit.decimals,
            ),
            expires_at_second: expiresAtSecond,
          });
          if (!isCancelled.current) {
            newOffers.push(data.offer);
          }
        }
        if (!isCancelled.current) {
          setCreatedOffers(newOffers);
        }
      } else {
        onProgress?.(0);
        const data = await commands.makeOffer({
          offered_assets: {
            xch: toMojos(
              (offerState.offered.xch || '0').toString(),
              walletState.sync.unit.decimals,
            ),
            cats: offerState.offered.cats.map((cat) => ({
              asset_id: cat.asset_id,
              amount: toMojos((cat.amount || '0').toString(), 3),
            })),
            nfts: offerState.offered.nfts.filter((n) => n),
          },
          requested_assets: {
            xch: toMojos(
              (offerState.requested.xch || '0').toString(),
              walletState.sync.unit.decimals,
            ),
            cats: offerState.requested.cats.map((cat) => ({
              asset_id: cat.asset_id,
              amount: toMojos((cat.amount || '0').toString(), 3),
            })),
            nfts: offerState.requested.nfts.filter((n) => n),
          },
          fee: toMojos(
            (offerState.fee || '0').toString(),
            walletState.sync.unit.decimals,
          ),
          expires_at_second: expiresAtSecond,
        });
        if (!isCancelled.current) {
          setCreatedOffers([data.offer]);
        }
      }
    } catch (err) {
      if (!isCancelled.current) {
        throw err;
      }
    } finally {
      if (!isCancelled.current) {
        setIsProcessing(false);
        onProcessingEnd?.();
      }
    }
  }, [
    offerState,
    splitNftOffers,
    walletState.sync.unit.decimals,
    promptIfEnabled,
    onProcessingEnd,
    onProgress,
  ]); // Removed clearProcessedOffers from dependency array

  return {
    createdOffers,
    isProcessing,
    processOffer,
    clearProcessedOffers,
    cancelProcessing,
  };
}
