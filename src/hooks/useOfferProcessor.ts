import { commands, OfferAmount } from '@/bindings';
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
    setCreatedOffers([]);

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

          const offeredAssets: OfferAmount[] = [
            ...offerState.offered.tokens,
            { asset_id: nft, amount: 1 },
            ...offerState.offered.options.map((option) => ({
              asset_id: option,
              amount: 1,
            })),
          ];

          const requestedAssets: OfferAmount[] = [
            ...offerState.requested.tokens,
            ...offerState.requested.nfts.map((nft) => ({
              asset_id: nft,
              amount: 1,
            })),
            ...offerState.requested.options.map((option) => ({
              asset_id: option,
              amount: 1,
            })),
          ];

          const data = await commands.makeOffer({
            offered_assets: offeredAssets,
            requested_assets: requestedAssets,
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

        const offeredAssets: OfferAmount[] = [
          ...offerState.offered.tokens,
          ...offerState.offered.nfts.map((nft) => ({
            asset_id: nft,
            amount: 1,
          })),
          ...offerState.offered.options.map((option) => ({
            asset_id: option,
            amount: 1,
          })),
        ];

        const requestedAssets: OfferAmount[] = [
          ...offerState.requested.tokens,
          ...offerState.requested.nfts.map((nft) => ({
            asset_id: nft,
            amount: 1,
          })),
          ...offerState.requested.options.map((option) => ({
            asset_id: option,
            amount: 1,
          })),
        ];

        const data = await commands.makeOffer({
          offered_assets: offeredAssets,
          requested_assets: requestedAssets,
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
  ]);

  return {
    createdOffers,
    isProcessing,
    processOffer,
    clearProcessedOffers,
    cancelProcessing,
  };
}
