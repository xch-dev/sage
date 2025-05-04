use chia::{protocol::Bytes32, puzzles::nft::NftMetadata};
use chia_puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH;
use chia_wallet_sdk::driver::{DidOwner, HashedPtr, NftMint};
use tracing::info;

use crate::{Action, AssetType, Distribution, DistributionCoin, Id, Summary, WalletError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintNftAction {
    pub metadata: NftMetadata,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
    pub p2_puzzle_hash: Bytes32,
    pub minter_did: Id,
}

impl MintNftAction {
    pub fn new(
        metadata: NftMetadata,
        royalty_puzzle_hash: Bytes32,
        royalty_ten_thousandths: u16,
        p2_puzzle_hash: Bytes32,
        minter_did: Id,
    ) -> Self {
        Self {
            metadata,
            royalty_puzzle_hash,
            royalty_ten_thousandths,
            p2_puzzle_hash,
            minter_did,
        }
    }
}

impl Action for MintNftAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        summary.created_nfts.insert(Id::New(index));
        summary.spent_dids.insert(self.minter_did);
        summary.spent_xch += 1;
    }

    fn distribute(
        &self,
        distribution: &mut Distribution<'_>,
        index: usize,
    ) -> Result<(), WalletError> {
        if distribution.asset_id() != Some(self.minter_did)
            || distribution.asset_type() != AssetType::Did
        {
            return Ok(());
        }

        info!("XYZXYXYZYXYZYXYZYXZYYXZYZYXYZYZYXY");

        distribution.create_launcher(|ctx, new_assets, item, launcher, conditions| {
            let DistributionCoin::Did(did) = item.coin else {
                unreachable!()
            };

            info!("ABCABCABCABCABBBBABABABAB");

            let mint = NftMint {
                metadata: self.metadata.clone(),
                metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_DEFAULT_HASH.into(),
                royalty_puzzle_hash: self.royalty_puzzle_hash,
                royalty_ten_thousandths: self.royalty_ten_thousandths,
                p2_puzzle_hash: self.p2_puzzle_hash,
                owner: Some(DidOwner::from_did_info(&did.info)),
            };

            let (mint_nft, nft) = launcher.mint_nft(ctx, mint)?;

            let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(ctx, metadata_ptr));

            new_assets.nfts.insert(Id::New(index), nft);

            Ok(conditions.extend(mint_nft))
        })
    }
}
