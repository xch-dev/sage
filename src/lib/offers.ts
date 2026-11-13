import { commands, OfferAsset, OfferRecord } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { durationUntil, Expiration } from '@/lib/expiration';
import { fromMojos } from '@/lib/utils';
import { Assets, OfferState } from '@/state';

export async function deleteOffers(offerIds: string[]): Promise<void> {
  for (const offerId of offerIds) {
    await commands.deleteOffer({ offer_id: offerId });
  }
}

// Loads current records for the given offers, skipping any that no longer
// exist. Rejections whose kind is not `not_found` are reported via
// `onError` instead of being silently dropped, since those indicate a real
// failure (e.g. a backend or connection error) rather than a stale offer id.
export async function fetchOfferRecords(
  offerIds: string[],
  onError?: (error: CustomError) => void,
): Promise<OfferRecord[]> {
  const results = await Promise.allSettled(
    offerIds.map((offerId) => commands.getOffer({ offer_id: offerId })),
  );

  const records: OfferRecord[] = [];

  for (const result of results) {
    if (result.status === 'fulfilled') {
      records.push(result.value.offer);
      continue;
    }

    const error = result.reason as CustomError;
    if (error?.kind !== 'not_found') {
      onError?.(error);
    }
  }

  return records;
}

// Whether identical copies of an offer can be made. Each copy needs its own
// coins, so an offer containing an NFT or option (on either side) can't be
// copied: the offered ones only exist once, and royalties owed on requested
// NFTs aren't known until the offer is built.
export function canCopyOffer(state: OfferState): boolean {
  return [state.offered, state.requested].every(
    (assets) =>
      assets.nfts.filter((n) => n).length === 0 &&
      assets.options.filter((o) => o).length === 0,
  );
}

// Whether a saved offer can be duplicated. This checks the record itself
// rather than offerStateFromRecord's result, which has nowhere to put a DID
// and would silently drop it.
export function canDuplicateOffer(record: OfferRecord): boolean {
  return [...record.summary.maker, ...record.summary.taker].every(
    (a) => a.asset.kind === 'token',
  );
}

// Rebuilds the make offer form state for a saved offer, so it can be created
// again. Amounts use the same precisions useOfferProcessor converts them back
// with.
//
// A timestamp expiration keeps the original offer's duration, counted from
// when the copy is made. The duration is measured from when the offer was
// saved to this wallet, which for an imported offer is later than when it was
// made; if it had already expired by then, `fallbackExpiration` is used
// instead. A block height expiration can't be expressed and is dropped.
export function offerStateFromRecord(
  record: OfferRecord,
  fallbackExpiration: Expiration | null = null,
): OfferState {
  const { summary } = record;

  const toAssets = (offerAssets: OfferAsset[]): Assets => ({
    tokens: offerAssets
      .filter((a) => a.asset.kind === 'token')
      .map((a) => ({
        asset_id: a.asset.asset_id,
        amount: fromMojos(a.amount, a.asset.asset_id ? 3 : 12).toString(),
      })),
    nfts: offerAssets
      .filter((a) => a.asset.kind === 'nft')
      .map((a) => a.asset.asset_id ?? ''),
    options: offerAssets
      .filter((a) => a.asset.kind === 'option')
      .map((a) => a.asset.asset_id ?? ''),
  });

  let expiration: Expiration | null = null;
  if (summary.expiration_timestamp !== null) {
    expiration =
      summary.expiration_timestamp > record.creation_timestamp
        ? durationUntil(summary.expiration_timestamp, record.creation_timestamp)
        : fallbackExpiration;
  }

  return {
    offered: toAssets(summary.maker),
    requested: toAssets(summary.taker),
    fee: fromMojos(summary.fee, 12).toString(),
    expiration,
  };
}
