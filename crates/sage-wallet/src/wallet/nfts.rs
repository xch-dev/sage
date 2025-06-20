use chia::{
    protocol::{Bytes32, CoinSpend, Program},
    puzzles::{nft::NftMetadata, Memos},
};
use chia_puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH;
use chia_wallet_sdk::{
    driver::{Did, HashedPtr, Launcher, MetadataUpdate, Nft, NftMint, SpendContext, StandardLayer},
    prelude::TransferNft,
    types::Conditions,
};

use crate::WalletError;

use super::Wallet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletNftMint {
    pub metadata: NftMetadata,
    pub p2_puzzle_hash: Option<Bytes32>,
    pub royalty_puzzle_hash: Option<Bytes32>,
    pub royalty_basis_points: u16,
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

        let total_amount = fee + mints.len() as u64;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u64 = coins.iter().map(|coin| coin.amount).sum();

        let change = selected - total_amount;

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
        let did = did.with_metadata(HashedPtr::from_ptr(&ctx, did_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let mut did_conditions = Conditions::new();
        let mut nfts = Vec::with_capacity(mints.len());

        for (i, mint) in mints.into_iter().enumerate() {
            let mint = NftMint {
                metadata: mint.metadata,
                metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_DEFAULT_HASH.into(),
                royalty_puzzle_hash: mint.royalty_puzzle_hash.unwrap_or(p2_puzzle_hash),
                royalty_basis_points: mint.royalty_basis_points,
                p2_puzzle_hash: mint.p2_puzzle_hash.unwrap_or(p2_puzzle_hash),
                transfer_condition: Some(TransferNft::new(
                    Some(did.info.launcher_id),
                    vec![],
                    Some(did.info.inner_puzzle_hash().into()),
                )),
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
            conditions = conditions.create_coin(p2_puzzle_hash, change, Memos::None);
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        let new_did = new_did.with_metadata(ctx.serialize(&new_did.info.metadata)?);

        Ok((ctx.take(), nfts, new_did))
    }

    pub async fn transfer_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if nft_ids.is_empty() {
            return Err(WalletError::EmptyBulkTransfer);
        }

        let is_external = !self.db.is_p2_puzzle_hash(puzzle_hash).await?;

        let mut nfts = Vec::new();

        for nft_id in nft_ids {
            let Some(nft) = self.db.spendable_nft(nft_id).await? else {
                return Err(WalletError::MissingNft(nft_id));
            };

            nfts.push(nft);
        }

        let coins = if fee > 0 {
            self.select_p2_coins(fee).await?
        } else {
            Vec::new()
        };
        let selected: u64 = coins.iter().map(|coin| coin.amount).sum();
        let change = selected - fee;

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let nft_coin_ids = nfts
            .iter()
            .map(|nft| nft.coin.coin_id())
            .collect::<Vec<_>>();

        for (i, nft) in nfts.into_iter().enumerate() {
            let nft_metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx, nft_metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let mut conditions = Conditions::new();

            if nft_coin_ids.len() > 1 {
                conditions = conditions.assert_concurrent_spend(
                    nft_coin_ids[if i == 0 {
                        nft_coin_ids.len() - 1
                    } else {
                        i - 1
                    }],
                );
            };

            if is_external && nft.info.current_owner.is_some() {
                conditions = conditions.transfer_nft(None, Vec::new(), None);
            }

            let _nft = nft.transfer(&mut ctx, &p2, puzzle_hash, conditions)?;
        }

        if fee > 0 {
            let mut conditions = Conditions::new()
                .assert_concurrent_spend(nft_coin_ids[0])
                .reserve_fee(fee);

            if change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, change, Memos::None);
            }

            self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        }

        Ok(ctx.take())
    }

    pub async fn add_nft_uri(
        &self,
        nft_id: Bytes32,
        fee: u64,
        uri: MetadataUpdate,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Nft<Program>), WalletError> {
        let Some(nft) = self.db.spendable_nft(nft_id).await? else {
            return Err(WalletError::MissingNft(nft_id));
        };

        let coins = if fee > 0 {
            self.select_p2_coins(fee).await?
        } else {
            Vec::new()
        };
        let selected: u64 = coins.iter().map(|coin| coin.amount).sum();
        let change = selected - fee;

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let nft_metadata_ptr = ctx.alloc(&nft.info.metadata)?;
        let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx, nft_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let update_spend = uri.spend(&mut ctx)?;
        let new_nft: Nft<HashedPtr> = nft.transfer_with_metadata(
            &mut ctx,
            &p2,
            nft.info.p2_puzzle_hash,
            update_spend,
            Conditions::new(),
        )?;

        if fee > 0 {
            let mut conditions = Conditions::new()
                .assert_concurrent_spend(nft.coin.coin_id())
                .reserve_fee(fee);

            if change > 0 {
                conditions = conditions.create_coin(p2_puzzle_hash, change, Memos::None);
            }

            self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        }

        let new_nft = new_nft.with_metadata(ctx.serialize(&new_nft.info.metadata)?);

        Ok((ctx.take(), new_nft))
    }

    pub async fn assign_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        did_id: Option<Bytes32>,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if nft_ids.is_empty() {
            return Err(WalletError::EmptyBulkTransfer);
        }

        let mut nfts = Vec::new();

        for nft_id in nft_ids {
            let Some(nft) = self.db.spendable_nft(nft_id).await? else {
                return Err(WalletError::MissingNft(nft_id));
            };

            nfts.push(nft);
        }

        let did = if let Some(did_id) = did_id {
            let did = self
                .db
                .spendable_did(did_id)
                .await?
                .ok_or(WalletError::MissingDid(did_id))?;

            Some(did)
        } else {
            None
        };

        let coins = if fee > 0 {
            self.select_p2_coins(fee).await?
        } else {
            Vec::new()
        };
        let selected: u64 = coins.iter().map(|coin| coin.amount).sum();
        let change = selected - fee;

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did = if let Some(did) = did {
            let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
            Some(did.with_metadata(HashedPtr::from_ptr(&ctx, did_metadata_ptr)))
        } else {
            None
        };

        let nft_coin_ids = nfts
            .iter()
            .map(|nft| nft.coin.coin_id())
            .collect::<Vec<_>>();

        let mut did_conditions = Conditions::new();

        for (i, nft) in nfts.into_iter().enumerate() {
            let nft_metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx, nft_metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let conditions = if nft_coin_ids.len() == 1 {
                Conditions::new()
            } else {
                Conditions::new().assert_concurrent_spend(
                    nft_coin_ids[if i == 0 {
                        nft_coin_ids.len() - 1
                    } else {
                        i - 1
                    }],
                )
            };

            let (parent_conditions, _nft) = nft.assign_owner(
                &mut ctx,
                &p2,
                nft.info.p2_puzzle_hash,
                did.as_ref()
                    .map(|did| {
                        TransferNft::new(
                            Some(did.info.launcher_id),
                            vec![],
                            Some(did.info.inner_puzzle_hash().into()),
                        )
                    })
                    .unwrap_or_default(),
                conditions,
            )?;

            did_conditions = did_conditions.extend(parent_conditions);
        }

        let did_coin_id = did.as_ref().map(|did| did.coin.coin_id());

        if let Some(did) = did {
            let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);
            let _did = did.update(&mut ctx, &p2, did_conditions)?;
        }

        if fee > 0 {
            let mut conditions = Conditions::new()
                .assert_concurrent_spend(nft_coin_ids[0])
                .reserve_fee(fee);

            if change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, change, Memos::None);
            }

            if let Some(did_coin_id) = did_coin_id {
                conditions = conditions.assert_concurrent_spend(did_coin_id);
            }

            self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        }

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::TestWallet;

    use super::*;

    #[test(tokio::test)]
    async fn test_mint_nft() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let puzzle_hash = test.wallet.p2_puzzle_hash(false, true).await?;

        let nft = nfts.remove(0);

        for item in [
            MetadataUpdate::NewDataUri("abc".to_string()),
            MetadataUpdate::NewMetadataUri("xyz".to_string()),
            MetadataUpdate::NewLicenseUri("123".to_string()),
        ] {
            let (coin_spends, _nft) = test
                .wallet
                .add_nft_uri(nft.info.launcher_id, 0, item, false, true)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, false, true)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        let nft = test
            .wallet
            .db
            .nft_row(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(nft.owner_did, Some(did.info.launcher_id));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_nft_internal() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let puzzle_hash = test.wallet.p2_puzzle_hash(false, true).await?;

        let nft = nfts.remove(0);

        let coin_spends = test
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, false, true)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let nft = test
            .wallet
            .db
            .nft_row(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(nft.owner_did, Some(did.info.launcher_id));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_nft_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(1).await?;

        let (coin_spends, alice_did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, bob_did) = bob.wallet.create_did(0).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                alice_did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let puzzle_hash = bob.wallet.p2_puzzle_hash(false, true).await?;

        let nft = nfts.remove(0);

        let coin_spends = alice
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, false, true)
            .await?;
        alice.transact(coin_spends).await?;
        bob.wait_for_puzzles().await;

        let row = bob
            .wallet
            .db
            .nft_row(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(row.owner_did, None);

        let coin_spends = bob
            .wallet
            .assign_nfts(
                vec![nft.info.launcher_id],
                Some(bob_did.info.launcher_id),
                0,
                false,
                true,
            )
            .await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let coin_spends = bob
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, false, true)
            .await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let row = bob
            .wallet
            .db
            .nft_row(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(row.owner_did, Some(bob_did.info.launcher_id));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_assign_nft() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let nft = nfts.remove(0);

        let coin_spends = test
            .wallet
            .assign_nfts(vec![nft.info.launcher_id], None, 0, false, true)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let coin_spends = test
            .wallet
            .assign_nfts(
                vec![nft.info.launcher_id],
                Some(did.info.launcher_id),
                0,
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        Ok(())
    }
}
