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
  /** The marketplaces it hasn't been uploaded to yet. */
  marketplaceIds: string[];
  /** How many times in a row uploading it has failed. */
  failures?: number;
  /** When to try uploading it again after a failure, in milliseconds. */
  retryAt?: number;
}

const STORAGE_KEY = 'pendingOfferUploads';

/** How many times in a row an upload can fail before it's given up on. */
export const MAX_UPLOAD_FAILURES = 5;

/** How long to wait after the first failed upload, doubled after each one. */
const RETRY_DELAY_MS = 30_000;

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

function update(upload: PendingOfferUpload) {
  if (upload.marketplaceIds.length === 0) {
    remove(upload.offerId);
    return;
  }

  save(
    load().map((existing) =>
      existing.offerId === upload.offerId ? upload : existing,
    ),
  );
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
 * Failed uploads are tried again later, with a growing delay, and given up on
 * after failing several times in a row.
 */
export async function processPendingOfferUploads(
  fingerprint: number,
  now = Date.now(),
) {
  const uploads = load().filter((upload) => upload.fingerprint === fingerprint);

  for (const upload of uploads) {
    if (upload.retryAt !== undefined && upload.retryAt > now) continue;

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

    if (status !== 'active') {
      remove(upload.offerId);
      continue;
    }

    let { marketplaceIds } = upload;
    const errors: [string, string][] = [];

    for (const marketplace of marketplaces) {
      if (!marketplaceIds.includes(marketplace.id)) continue;

      const marketplaceName = marketplace.name;

      try {
        await marketplace.uploadToMarketplace(upload.offer, upload.isTestnet);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        console.warn(`Failed to upload offer to ${marketplaceName}`, error);
        errors.push([marketplaceName, message]);
        continue;
      }

      toast.success(t`Uploaded offer to ${marketplaceName}`);

      // It's forgotten as soon as it's uploaded, so that it's never uploaded
      // to the same marketplace twice.
      marketplaceIds = marketplaceIds.filter((id) => id !== marketplace.id);
      update({ ...upload, marketplaceIds });
    }

    if (errors.length === 0) {
      remove(upload.offerId);
      continue;
    }

    const failures = (upload.failures ?? 0) + 1;

    if (failures < MAX_UPLOAD_FAILURES) {
      update({
        ...upload,
        marketplaceIds,
        failures,
        retryAt: now + RETRY_DELAY_MS * 2 ** (failures - 1),
      });
      continue;
    }

    remove(upload.offerId);

    for (const [marketplaceName, message] of errors) {
      toast.error(t`Failed to upload offer to ${marketplaceName}: ${message}`);
    }
  }
}
