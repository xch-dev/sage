import { NftUpdaterKind, OfferAsset } from '@/bindings';
import { t } from '@lingui/core/macro';

/**
 * Whether the owner of an NFT may be able to change its metadata beyond appending URIs.
 * Unknown updaters are treated as mutable, since nothing is known about how they behave.
 */
export function isMutableNft(kind: NftUpdaterKind | null | undefined): boolean {
  return kind === 'any' || kind === 'unknown';
}

export function mutableNftExplanation(kind: NftUpdaterKind): string {
  return kind === 'unknown'
    ? t`This NFT uses an unrecognized metadata updater, so its owner may be able to change or replace its metadata, including its name and image.`
    : t`The owner of this NFT can change or replace all of its metadata, including its name and image, at any time.`;
}

/**
 * Labels a mutable NFT in a warning. Its name can be changed by the owner, so the NFT ID
 * is always included, since that's what identifies it.
 */
export function mutableNftLabel(name: string | null, nftId: string): string {
  const shortId = `${nftId.slice(0, 10)}...${nftId.slice(-4)}`;
  return name ? `${name} (${shortId})` : shortId;
}

/** Labels for the mutable NFTs in one side of an offer. */
export function mutableOfferNftLabels(assets: OfferAsset[]): string[] {
  return assets
    .filter((asset) => isMutableNft(asset.nft_metadata_updater_kind))
    .map((asset) =>
      asset.asset.asset_id
        ? mutableNftLabel(asset.asset.name, asset.asset.asset_id)
        : (asset.asset.name ?? t`Unknown NFT`),
    );
}
