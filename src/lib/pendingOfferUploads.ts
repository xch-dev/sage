import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { marketplaces } from '@/lib/marketplaces';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

/**
 * An offer whose coins are still being split, which should be uploaded to
 * marketplaces once the split is confirmed. Marketplaces reject offers whose
 * coins don't exist yet, so the upload has to wait until then.
 */
export interface PendingOfferUpload {
  fingerprint: number;
  offerId: string;
  offer: string;
  isTestnet: boolean;
  marketplaceIds: string[];
}

const STORAGE_KEY = 'pendingOfferUploads';

function load(): PendingOfferUpload[] {
  try {
    const value = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]');
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}

function save(uploads: PendingOfferUpload[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(uploads));
}

function remove(offerId: string) {
  save(load().filter((upload) => upload.offerId !== offerId));
}

async function isWalletOpen(fingerprint: number) {
  try {
    return (await commands.getWallet({})).wallet?.fingerprint === fingerprint;
  } catch {
    return false;
  }
}

/** Uploads an offer to marketplaces once the transaction splitting its coins is confirmed. */
export function queuePendingOfferUpload(upload: PendingOfferUpload) {
  if (upload.marketplaceIds.length === 0) return;

  save([
    ...load().filter((existing) => existing.offerId !== upload.offerId),
    upload,
  ]);
}

/**
 * Uploads the queued offers of a wallet which are no longer pending. Offers
 * which can never be taken, or which were deleted, are forgotten instead.
 */
export async function processPendingOfferUploads(fingerprint: number) {
  const uploads = load().filter((upload) => upload.fingerprint === fingerprint);

  for (const upload of uploads) {
    let status;

    try {
      status = (await commands.getOffer({ offer_id: upload.offerId })).offer
        .status;
    } catch (error) {
      // The offer was deleted, unless a different wallet was opened meanwhile.
      if (
        (error as CustomError).kind === 'not_found' &&
        (await isWalletOpen(fingerprint))
      ) {
        remove(upload.offerId);
      }
      continue;
    }

    if (status === 'pending') continue;

    // It's forgotten before uploading, so that it's never uploaded twice.
    remove(upload.offerId);

    if (status !== 'active') continue;

    for (const marketplace of marketplaces) {
      if (!upload.marketplaceIds.includes(marketplace.id)) continue;

      const marketplaceName = marketplace.name;

      try {
        await marketplace.uploadToMarketplace(upload.offer, upload.isTestnet);
        toast.success(t`Uploaded offer to ${marketplaceName}`);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        toast.error(
          t`Failed to upload offer to ${marketplaceName}: ${message}`,
        );
      }
    }
  }
}
