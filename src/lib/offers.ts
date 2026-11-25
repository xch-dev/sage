import { commands, OfferRecord } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';

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
