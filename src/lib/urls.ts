// ── Private base helpers ──────────────────────────────────────────────────────

function spacescanBase(isTestnet: boolean): string {
  return `https://${isTestnet ? 'testnet11.' : ''}spacescan.io`;
}

function dexieBrowseBase(isTestnet: boolean): string {
  return `https://${isTestnet ? 'testnet.' : ''}dexie.space`;
}

function dexieApiBase(isTestnet: boolean): string {
  return `https://api${isTestnet ? '-testnet' : ''}.dexie.space`;
}

function mintGardenBrowseBase(isTestnet: boolean): string {
  return `https://${isTestnet ? 'testnet.' : ''}mintgarden.io`;
}

function mintGardenApiBase(isTestnet: boolean): string {
  return `https://api${isTestnet ? '.testnet' : ''}.mintgarden.io`;
}

// ── Spacescan ─────────────────────────────────────────────────────────────────

export function spacescanCoinUrl(coinId: string, isTestnet: boolean): string {
  return `${spacescanBase(isTestnet)}/coin/0x${coinId}`;
}

export function spacescanNftUrl(
  launcherId: string,
  isTestnet: boolean,
): string {
  return `${spacescanBase(isTestnet)}/nft/${launcherId}`;
}

export function spacescanCollectionUrl(
  collectionId: string,
  isTestnet: boolean,
): string {
  return `${spacescanBase(isTestnet)}/collection/${collectionId}`;
}

// ── Dexie — browser ──────────────────────────────────────────────────────────

export function dexieOfferUrl(offerId: string, isTestnet: boolean): string {
  return `${dexieBrowseBase(isTestnet)}/offers/${offerId}`;
}

export function dexieAssetUrl(assetId: string, isTestnet: boolean): string {
  return `${dexieBrowseBase(isTestnet)}/offers/XCH/${assetId}`;
}

// ── Dexie — API ──────────────────────────────────────────────────────────────
// `path` must include the API version prefix (e.g. 'v1/offers', 'v3/prices/tickers').

export function dexieApiUrl(path: string, isTestnet: boolean): string {
  return `${dexieApiBase(isTestnet)}/${path}`;
}

// ── MintGarden — browser ─────────────────────────────────────────────────────

export function mintGardenOfferUrl(hash: string, isTestnet: boolean): string {
  return `${mintGardenBrowseBase(isTestnet)}/offers/${hash}`;
}

export function mintGardenNftUrl(id: string, isTestnet: boolean): string {
  return `${mintGardenBrowseBase(isTestnet)}/nfts/${id}`;
}

export function mintGardenCollectionUrl(
  id: string,
  isTestnet: boolean,
): string {
  return `${mintGardenBrowseBase(isTestnet)}/collections/${id}`;
}

export function mintGardenDidUrl(did: string, isTestnet: boolean): string {
  return `${mintGardenBrowseBase(isTestnet)}/${did}`;
}

// ── MintGarden — API ─────────────────────────────────────────────────────────
// `path` is the resource path without a leading slash (e.g. 'offer', 'offers/abc123').

export function mintGardenApiUrl(path: string, isTestnet: boolean): string {
  return `${mintGardenApiBase(isTestnet)}/${path}`;
}

// ── ChiaOffer — API ─────────────────────────────────────────────────────────
// `id` is the offer id.

export function chiaOfferApiUrl(id: string, isTestnet: boolean): string {
  if (isTestnet) {
    console.warn(
      'ChiaOffer.com does not have a testnet API, but isTestnet was set to true. Proceeding with mainnet URL.',
    );
  }
  return `https://api.chia-offer.com/get-offer.php?id=${id}`;
}

// ── OfferCo — API ─────────────────────────────────────────────────────────

export function offerCoApiUrl(isTestnet: boolean): string {
  if (isTestnet) {
    console.warn(
      'OfferCo.com does not have a testnet API, but isTestnet was set to true. Proceeding with mainnet URL.',
    );
  }
  return 'https://offerco.de/api/v1/getoffer';
}
