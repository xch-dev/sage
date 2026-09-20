import { commands } from '@/bindings';

export async function deleteOffers(offerIds: string[]): Promise<void> {
  for (const offerId of offerIds) {
    await commands.deleteOffer({ offer_id: offerId });
  }
}
