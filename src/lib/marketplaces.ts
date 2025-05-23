import { OfferSummary } from '@/bindings';
import { OfferState } from '@/state';
import {
  dexieLink,
  uploadToDexie,
  uploadToMintGarden,
  offerIsOnDexie,
  offerIsOnMintGarden,
  isMintGardenSupportedForSummary,
  mintGardenLink,
  isDexieSupportedForSummary,
  isMintGardenSupported,
  isDexieSupported,
} from '@/lib/offerUpload';
import { MarketplaceConfig } from '@/components/MarketplaceCard';
// need to store mintgarden logo locally because of CORS
import mintGardenLogo from '@/images/mintgarden-logo.svg';

export const marketplaces: MarketplaceConfig[] = [
  {
    id: 'dexie',
    name: 'Dexie',
    logo: 'https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg',
    qrCodeLogo:
      'https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg',
    isSupported: (offer: OfferSummary | OfferState, isSplitting: boolean) => {
      if ('taker' in offer) {
        return isDexieSupportedForSummary(offer);
      }

      return isDexieSupported(offer, isSplitting);
    },
    isOnMarketplace: (_, offerId, isTestnet) =>
      offerIsOnDexie(offerId, isTestnet),
    uploadToMarketplace: uploadToDexie,
    getMarketplaceLink: dexieLink,
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

      return isMintGardenSupported(offer), isSplitting;
    },
    isOnMarketplace: (offer, _, isTestnet) =>
      offerIsOnMintGarden(offer, isTestnet),
    uploadToMarketplace: uploadToMintGarden,
    getMarketplaceLink: mintGardenLink,
  },
];
