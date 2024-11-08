use chia::{
    protocol::{Bytes32, CoinSpend, Program},
    puzzles::nft::{NftMetadata, NFT_METADATA_UPDATER_PUZZLE_HASH},
};
use chia_wallet_sdk::{
    Conditions, Did, DidOwner, HashedPtr, Launcher, Nft, NftMint, SpendContext, StandardLayer,
};

use crate::WalletError;

use super::Wallet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletNftMint {
    pub metadata: NftMetadata,
    pub royalty_puzzle_hash: Option<Bytes32>,
    pub royalty_ten_thousandths: u16,
}

impl Wallet {
    pub async fn bulk_mint_nfts(
        &self,
        fee: u64,
        did_id: Bytes32,
        mints: Vec<WalletNftMint>,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Vec<Nft<NftMetadata>>, Did<Program>), WalletError> {
        let Some(did) = self.db.spendable_did(did_id).await? else {
            return Err(WalletError::MissingDid(did_id));
        };

        let total_amount = fee as u128 + 1;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
        let did = did.with_metadata(HashedPtr::from_ptr(&ctx.allocator, did_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let mut did_conditions = Conditions::new();
        let mut nfts = Vec::with_capacity(mints.len());

        for (i, mint) in mints.into_iter().enumerate() {
            let mint = NftMint {
                metadata: mint.metadata,
                metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_PUZZLE_HASH.into(),
                royalty_puzzle_hash: mint.royalty_puzzle_hash.unwrap_or(p2_puzzle_hash),
                royalty_ten_thousandths: mint.royalty_ten_thousandths,
                p2_puzzle_hash,
                owner: Some(DidOwner::from_did_info(&did.info)),
            };

            let (mint_nft, nft) = Launcher::new(did.coin.coin_id(), i as u64 * 2)
                .with_singleton_amount(1)
                .mint_nft(&mut ctx, mint)?;

            did_conditions = did_conditions.extend(mint_nft);
            nfts.push(nft);
        }

        let new_did = did.update(&mut ctx, &p2, did_conditions)?;

        let mut conditions = Conditions::new().assert_concurrent_spend(did.coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        let new_did = new_did.with_metadata(ctx.serialize(&new_did.info.metadata)?);

        Ok((ctx.take(), nfts, new_did))
    }

    pub async fn transfer_nft(
        &self,
        nft_id: Bytes32,
        puzzle_hash: Bytes32,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Nft<Program>), WalletError> {
        let Some(nft) = self.db.spendable_nft(nft_id).await? else {
            return Err(WalletError::MissingNft(nft_id));
        };

        let total_amount = fee as u128 + 1;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let nft_metadata_ptr = ctx.alloc(&nft.info.metadata)?;
        let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx.allocator, nft_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let new_nft = nft.transfer(&mut ctx, &p2, puzzle_hash, Conditions::new())?;

        let mut conditions = Conditions::new().assert_concurrent_spend(nft.coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        let new_nft = new_nft.with_metadata(ctx.serialize(&new_nft.info.metadata)?);

        Ok((ctx.take(), new_nft))
    }
}
