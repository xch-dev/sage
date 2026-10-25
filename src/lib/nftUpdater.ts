import { NftRecord, NftUpdaterKind, OfferAsset } from '@/bindings';
import { t } from '@lingui/core/macro';

/**
 * Whether the owner of an NFT may be able to change its metadata beyond appending URIs.
 * Unknown updaters are treated as mutable, since nothing is known about how they behave.
 */
export function isMutableNft(kind: NftUpdaterKind | null | undefined): boolean {
  return kind === 'any' || kind === 'unknown';
}

/**
 * The name to show for an NFT. This prefers the XCHandles display name (`dn`), which is
 * owner-editable display data only and never proof of handle ownership.
 */
export function nftDisplayName(
  nft: Pick<NftRecord, 'name' | 'display_name'>,
): string | null {
  return nft.display_name ?? nft.name;
}

export function mutableNftExplanation(kind: NftUpdaterKind): string {
  return kind === 'unknown'
    ? t`This NFT uses an unrecognized metadata updater, so its owner may be able to change or replace its metadata, including its name and image.`
    : t`The owner of this NFT can change or replace all of its metadata, including its name and image, at any time.`;
}

/** The names of the mutable NFTs in one side of an offer. */
export function mutableOfferNftNames(assets: OfferAsset[]): string[] {
  return assets
    .filter((asset) => isMutableNft(asset.nft_metadata_updater_kind))
    .map(
      (asset) =>
        asset.nft_display_name ??
        asset.asset.name ??
        asset.asset.asset_id ??
        t`Unknown NFT`,
    );
}
