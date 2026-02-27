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

    const offeredTokens = offerState.offered.tokens.map((token) => ({
      asset_id: token.asset_id,
      amount: toMojos(token.amount.toString(), token.asset_id ? 3 : 12),
    }));

    const requestedTokens = offerState.requested.tokens.map((token) => ({
      asset_id: token.asset_id,
      amount: toMojos(token.amount.toString(), token.asset_id ? 3 : 12),
      fee_policy:
        token.asset_id && token.fee_policy
          ? {
              recipient: token.fee_policy.recipient,
              fee_basis_points:
                Number.parseInt(token.fee_policy.fee_basis_points || '0', 10) ||
                0,
              min_fee: toMojos(token.fee_policy.min_fee || '0', 3),
              allow_zero_price: token.fee_policy.allow_zero_price,
              allow_revoke_fee_bypass:
                token.fee_policy.allow_revoke_fee_bypass,
            }
          : undefined,
    }));

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
            fee: toMojos(
              (offerState.fee || '0').toString(),
              walletState.sync.unit.precision,
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
          fee: toMojos(
            (offerState.fee || '0').toString(),
            walletState.sync.unit.precision,
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
