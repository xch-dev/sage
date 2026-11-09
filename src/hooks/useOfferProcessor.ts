import { commands, MakeOfferResponse, OfferAmount } from '@/bindings';
import { useBiometric } from '@/hooks/useBiometric';
import { futureExpirationTimestamp } from '@/lib/expiration';
import { toMojos } from '@/lib/utils';
import { OfferState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { useCallback, useRef, useState } from 'react';

export interface CreatedOffer {
  offer: string;
  offerId: string;
  /** Whether the offer's coins are still being split, so it can't be shared yet. */
  isPending: boolean;
}

interface UseOfferProcessorProps {
  offerState: OfferState;
  splitNftOffers: boolean;
  /** The fee for splitting the offered coins to the exact amounts, if they should be. */
  preSplitFee: string | null;
  onProcessingEnd?: () => void; // Callback for when offer processing (success or fail) is done
  onProgress?: (index: number) => void; // Callback for progress updates
}

interface UseOfferProcessorReturn {
  createdOffers: CreatedOffer[];
  isProcessing: boolean;
  processOffer: () => Promise<void>;
  clearProcessedOffers: () => void;
  cancelProcessing: () => void;
}

export function useOfferProcessor({
  offerState,
  splitNftOffers,
  preSplitFee,
  onProcessingEnd,
  onProgress,
}: UseOfferProcessorProps): UseOfferProcessorReturn {
  const walletState = useWalletState();
  const { promptIfEnabled } = useBiometric();
  const [createdOffers, setCreatedOffers] = useState<CreatedOffer[]>([]);
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
      expiresAtSecond = futureExpirationTimestamp(offerState.expiration);
    }

    if (!(await promptIfEnabled())) {
      throw new Error(t`Biometric authentication failed or was cancelled`);
    }

    const offeredTokens = offerState.offered.tokens.map((token) => ({
      asset_id: token.asset_id,
      amount: toMojos(token.amount.toString(), token.asset_id ? 3 : 12),
    }));

    const requestedTokens = offerState.requested.tokens.map((token) => ({
      asset_id: token.asset_id,
      amount: toMojos(token.amount.toString(), token.asset_id ? 3 : 12),
    }));

    const fee = toMojos(
      (offerState.fee || '0').toString(),
      walletState.sync.unit.precision,
    );
    const pre_split_fee =
      preSplitFee === null
        ? null
        : toMojos(preSplitFee || '0', walletState.sync.unit.precision);

    const createdOffer = (data: MakeOfferResponse): CreatedOffer => ({
      offer: data.offer,
      offerId: data.offer_id,
      isPending: data.pre_split_transaction_id !== null,
    });

    try {
      if (
        splitNftOffers &&
        offerState.offered.nfts.filter((n) => n).length > 1
      ) {
        const newOffers: CreatedOffer[] = [];
        const nfts = offerState.offered.nfts.filter((n) => n);

        for (const [index, nft] of nfts.entries()) {
          if (isCancelled.current) {
            break;
          }

          onProgress?.(index);

          const offeredAssets: OfferAmount[] = [
            ...offeredTokens,
            { asset_id: nft, amount: 1 },
            ...offerState.offered.options.map((option) => ({
              asset_id: option,
              amount: 1,
            })),
          ];

          const requestedAssets: OfferAmount[] = [
            ...requestedTokens,
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
            fee,
            expires_at_second: expiresAtSecond,
            pre_split_fee,
          });
          if (!isCancelled.current) {
            newOffers.push(createdOffer(data));
          }
        }
        if (!isCancelled.current) {
          setCreatedOffers(newOffers);
        }
      } else {
        onProgress?.(0);

        const offeredAssets: OfferAmount[] = [
          ...offeredTokens,
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
          ...requestedTokens,
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
          fee,
          expires_at_second: expiresAtSecond,
          pre_split_fee,
        });
        if (!isCancelled.current) {
          setCreatedOffers([createdOffer(data)]);
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
    preSplitFee,
    walletState.sync.unit.precision,
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
