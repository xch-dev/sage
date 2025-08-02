import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';

const CNI_NFC_PREFIX = 'DT001';

export interface DexieAsset {
  id: string;
  name: string;
  amount: number;
  code: string;
}

export interface DexieOffer {
  id: string;
  offer: string;
  requested: DexieAsset[];
  offered: DexieAsset[];
}

interface DexieOfferResponse {
  success: boolean;
  count: number;
  page: number;
  page_size: number;
  offers: DexieOffer[];
}

export async function resolveOfferData(text: string): Promise<string> {
  try {
    if (isValidHostname(text, 'dexie.space')) {
      const offerId = extractOfferId(text);
      if (offerId) {
        const resolvedOffer = await fetchDexieOffer(offerId);
        if (resolvedOffer) {
          return resolvedOffer;
        }
      }
    }

    if (isValidHostname(text, 'offerco.de')) {
      const offerId = extractOfferId(text);
      if (offerId) {
        const resolvedOffer = await fetchOfferCoOffer(offerId);
        if (resolvedOffer) {
          return resolvedOffer;
        }
      }
    }

    if (isValidHostname(text, 'chia-offer.com')) {
      const offerId = extractOfferId(text);
      if (offerId) {
        const resolvedOffer = await fetchChiaOfferComOffer(offerId);
        if (resolvedOffer) {
          return resolvedOffer;
        }
      }
    }
  } catch {
    throw {
      kind: 'api',
      reason: 'Failed to resolve offer',
    } as CustomError;
  }

  if (text.startsWith(CNI_NFC_PREFIX)) {
    return await fetchCniNfcOffer(text);
  }

  return text;
}

function isValidHostname(url: string, expectedHostname: string) {
  try {
    const parsedUrl = new URL(url);
    return parsedUrl.hostname === expectedHostname;
  } catch {
    return false;
  }
}

function extractOfferId(url: string) {
  try {
    const segments = url.split('/');
    const lastSegment = segments[segments.length - 1];
    return lastSegment;
  } catch {
    return null;
  }
}

async function fetchChiaOfferComOffer(id: string): Promise<string> {
  const response = await fetch(
    `https://api.chia-offer.com/get-offer.php?id=${id}`,
  );
  const data = await response.json();

  if (!data) {
    throw {
      kind: 'api',
      reason:
        'Failed to fetch offer from chia-offer.com: Invalid response data',
    } as CustomError;
  }

  if (data.success && data.data?.full_offer) {
    return data.data.full_offer;
  }

  throw {
    kind: 'api',
    reason:
      'Failed to fetch offer from chia-offer.com: Offer not found or invalid format',
  } as CustomError;
}

export async function fetchOfferedDexieOffersFromNftId(
  id: string,
): Promise<DexieOffer[]> {
  return await fetchDexieOffersFromNftId(id, 'offered', 'price'); // lowest price first
}

export async function fetchRequestedDexieOffersFromNftId(
  id: string,
): Promise<DexieOffer[]> {
  return await fetchDexieOffersFromNftId(id, 'requested', 'price_desc'); // highest price first
}

async function fetchDexieOffersFromNftId(
  id: string,
  type: string,
  sort: string,
): Promise<DexieOffer[]> {
  // this will only get a single page of offers (20 by default) which is fine
  const response = await fetch(
    `https://api.dexie.space/v1/offers?${type}=${id}&status=0&sort=${sort}`,
  );
  const data = (await response.json()) as DexieOfferResponse;
  if (!data) {
    throw {
      kind: 'api',
      reason: 'Failed to fetch nft offers from Dexie: Invalid response data',
    } as CustomError;
  }

  if (data.success && data?.offers) {
    return data.offers;
  }

  return [];
}
async function fetchDexieOffer(id: string): Promise<string> {
  const response = await fetch(`https://api.dexie.space/v1/offers/${id}`);
  const data = await response.json();

  if (!data) {
    throw {
      kind: 'api',
      reason: 'Failed to fetch offer from Dexie: Invalid response data',
    } as CustomError;
  }

  if (data.success && data.offer?.offer) {
    return data.offer.offer;
  }

  throw {
    kind: 'api',
    reason:
      'Failed to fetch offer from Dexie: Offer not found or invalid format',
  } as CustomError;
}

async function fetchOfferCoOffer(id: string): Promise<string> {
  const response = await fetch('https://offerco.de/api/v1/getoffer', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      'X-API-Key': '602307f09cc95d490061bda377079f44',
    },
    body: `short_code=${id}`,
  });

  const data = await response.json();

  if (data.status === 'success' && data.data?.offer_code) {
    return data.data.offer_code;
  }

  throw {
    kind: 'api',
    reason: 'Failed to fetch offer from Offerco.de',
  } as CustomError;
}

async function fetchCniNfcOffer(text: string): Promise<string> {
  if (!text.startsWith(CNI_NFC_PREFIX)) {
    throw {
      kind: 'nfc',
      reason: 'Invalid NFC payload (not following CHIP-0047)',
    } as CustomError;
  }

  text = text.slice(CNI_NFC_PREFIX.length);

  const nftId = text.slice(0, 62);

  if (nftId.length !== 62 || !nftId.startsWith('nft1')) {
    throw {
      kind: 'nfc',
      reason:
        'NFC payload starts with CHIP-0047 prefix but does not have a valid NFT ID',
    } as CustomError;
  }

  text = text.slice(62);

  const offer = await commands.downloadCniOffercode(text);

  if (!offer) {
    throw {
      kind: 'nfc',
      reason: 'Failed to fetch offer from the CNI offercode API',
    } as CustomError;
  }

  return offer;
}
