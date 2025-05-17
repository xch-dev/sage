import { OfferSummary } from '@/bindings';
import {
    dexieLink,
    uploadToDexie,
    uploadToMintGarden,
    offerIsOnDexie,
    offerIsOnMintGarden,
    isMintGardenSupportedForSummary,
    mintGardenLink,
} from '@/lib/offerUpload';
import { MarketplaceConfig } from '@/components/MarketplaceCard';
// need to store mintgarden logo locally because of CORS
import mintGardenLogo from '@/images/mintgarden-logo.svg';

export const marketplaces: MarketplaceConfig[] = [
    {
        id: 'dexie',
        name: 'Dexie',
        logo: 'https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg',
        qrCodeLogo: 'https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg',
        isSupported: () => true, // Dexie supports all offers
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
        isSupported: (_, offerSummary?: OfferSummary) =>
            offerSummary ? isMintGardenSupportedForSummary(offerSummary) : false,
        isOnMarketplace: (offer, _, isTestnet) =>
            offerIsOnMintGarden(offer, isTestnet),
        uploadToMarketplace: uploadToMintGarden,
        getMarketplaceLink: mintGardenLink,
    },
];
