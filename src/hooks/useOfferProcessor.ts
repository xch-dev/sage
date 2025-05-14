import { useState, useCallback } from 'react';
import { commands } from '@/bindings';
import { OfferState, useWalletState } from '@/state';
import { useErrors } from '@/hooks/useErrors';
import { useBiometric } from '@/hooks/useBiometric';
import { toMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';

interface UseOfferProcessorProps {
    offerState: OfferState;
    splitNftOffers: boolean;
    onProcessingEnd?: () => void; // Callback for when offer processing (success or fail) is done
}

interface UseOfferProcessorReturn {
    createdOffer: string;
    createdOffers: string[];
    isProcessing: boolean;
    canUploadToMintGarden: boolean;
    processOffer: () => Promise<void>;
    clearProcessedOffers: () => void;
}

export function useOfferProcessor({
    offerState,
    splitNftOffers,
    onProcessingEnd,
}: UseOfferProcessorProps): UseOfferProcessorReturn {
    const walletState = useWalletState();
    const { addError } = useErrors();
    const { promptIfEnabled } = useBiometric();

    const [createdOffer, setCreatedOffer] = useState('');
    const [createdOffers, setCreatedOffers] = useState<string[]>([]);
    const [isProcessing, setIsProcessing] = useState(false);
    const [canUploadToMintGarden, setCanUploadToMintGarden] = useState(false);

    const clearProcessedOffers = useCallback(() => {
        setCreatedOffer('');
        setCreatedOffers([]);
        setCanUploadToMintGarden(false);
    }, []);

    const processOffer = useCallback(async () => {
        setIsProcessing(true);
        clearProcessedOffers(); // Clear previous offers before starting

        const mintgardenSupported =
            (offerState.offered.xch === '0' || !offerState.offered.xch) &&
            offerState.offered.cats.length === 0 &&
            offerState.offered.nfts.length === 1 &&
            (offerState.requested.xch === '0' || !offerState.requested.xch) &&
            offerState.requested.cats.length === 0 &&
            offerState.requested.nfts.length === 0;

        let expiresAtSecond: number | null = null;
        if (offerState.expiration !== null) {
            const days = parseInt(offerState.expiration.days) || 0;
            const hours = parseInt(offerState.expiration.hours) || 0;
            const minutes = parseInt(offerState.expiration.minutes) || 0;
            const totalSeconds = days * 24 * 60 * 60 + hours * 60 * 60 + minutes * 60;
            if (totalSeconds <= 0) {
                addError({
                    kind: 'invalid',
                    reason: t`Expiration must be at least 1 second in the future`,
                });
                setIsProcessing(false);
                onProcessingEnd?.();
                return;
            }
            expiresAtSecond = Math.ceil(Date.now() / 1000) + totalSeconds;
        }

        if (!(await promptIfEnabled())) {
            setIsProcessing(false);
            onProcessingEnd?.();
            return;
        }

        try {
            if (splitNftOffers && offerState.offered.nfts.filter((n) => n).length > 1) {
                const newOffers: string[] = [];
                for (const nft of offerState.offered.nfts.filter((n) => n)) {
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
                    newOffers.push(data.offer);
                }
                setCreatedOffers(newOffers);
                if (newOffers.length > 0) setCreatedOffer(newOffers[0]);
            } else {
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
                setCreatedOffer(data.offer);
            }
            setCanUploadToMintGarden(mintgardenSupported);
        } catch (err: any) {
            addError({
                kind: 'invalid',
                reason:
                    err.message || t`An unknown error occurred while creating the offer.`,
            });
        } finally {
            setIsProcessing(false);
            onProcessingEnd?.();
        }
    }, [
        offerState,
        splitNftOffers,
        walletState.sync.unit.decimals,
        addError,
        promptIfEnabled,
        clearProcessedOffers,
        onProcessingEnd,
    ]);

    return {
        createdOffer,
        createdOffers,
        isProcessing,
        canUploadToMintGarden,
        processOffer,
        clearProcessedOffers,
    };
} 