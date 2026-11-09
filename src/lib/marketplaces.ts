import { OfferSummary } from '@/bindings';
import { MarketplaceConfig } from '@/components/MarketplaceCard';
import {
  isDexieSupported,
  isDexieSupportedForSummary,
  isMintGardenSupported,
  isMintGardenSupportedForSummary,
  offerIsOnDexie,
  offerIsOnMintGarden,
  uploadToDexie,
  uploadToMintGarden,
} from '@/lib/offerUpload';
import {
  dexieOfferUrl,
  mintGardenApiUrl,
  mintGardenOfferUrl,
} from '@/lib/urls';
import { OfferState } from '@/state';
// need to store mintgarden logo locally because of CORS
import mintGardenLogo from '@/images/mintgarden-logo.svg';

export const marketplaces: MarketplaceConfig[] = [
  {
    id: 'dexie',
    name: 'Dexie',
    logo: 'https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg',
    qrCodeLogo:
      'https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg',
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    isSupported: (offer: OfferSummary | OfferState, _: boolean) => {
      if ('taker' in offer) {
        return isDexieSupportedForSummary(offer);
      }

      return isDexieSupported(offer);
    },
    isOnMarketplace: (_, offerId, isTestnet) =>
      offerIsOnDexie(offerId, isTestnet),
    uploadToMarketplace: uploadToDexie,
    getMarketplaceLink: dexieOfferUrl,
  },
  {
    id: 'mintgarden',
    name: 'MintGarden',
    logo: mintGardenLogo,
    qrCodeLogo: mintGardenLogo,
    isSupported: (offer: OfferSummary | OfferState, isSplitting: boolean) => {
      if ('taker' in offer) {
        return isMintGardenSupportedForSummary(offer);
      }

      return isMintGardenSupported(offer, isSplitting);
    },
    isOnMarketplace: (offer, _, isTestnet) =>
      offerIsOnMintGarden(offer, isTestnet),
    uploadToMarketplace: uploadToMintGarden,
    getMarketplaceLink: mintGardenOfferUrl,
  },
];

export async function getMintGardenProfile(did: string, isTestnet: boolean) {
  try {
    const response = await fetch(mintGardenApiUrl(`profile/${did}`, isTestnet));
    const data = await response.json();
    if ('detail' in data) {
      return null;
    }
    // always supply a name
    data.name = data.name || `${did.slice(9, 19)}...${did.slice(-4)}`;
    return data;
  } catch {
    return null;
  }
}
