use chia_wallet_sdk::{
    chia::puzzle_types::nft::NftMetadata,
    driver::{MetadataUpdate, TransferNftById},
    prelude::*,
    puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH,
};
use sage_database::{SerializePrimitive, SerializedNft};

use crate::{
    wallet::memos::{calculate_memos, Hint},
    WalletError,
};

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
    ) -> Result<(Vec<CoinSpend>, Vec<SerializedNft>), WalletError> {
        let default_royalty_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for mint in mints {
            let index = actions.len();

            let metadata = ctx.alloc_hashed(&mint.metadata)?;

            actions.push(Action::mint_nft_from_did(
                Id::Existing(did_id),
                metadata,
                NFT_METADATA_UPDATER_DEFAULT_HASH.into(),
                mint.royalty_puzzle_hash
                    .unwrap_or(default_royalty_puzzle_hash),
                mint.royalty_basis_points,
                1,
            ));

            actions.push(Action::update_nft(
                Id::New(index),
                vec![],
                Some(TransferNftById::new(Some(Id::Existing(did_id)), vec![])),
            ));

            if let Some(p2_puzzle_hash) = mint.p2_puzzle_hash {
                let hint = ctx.hint(p2_puzzle_hash)?;
                actions.push(Action::send(Id::New(index), p2_puzzle_hash, 1, hint));
            }
        }

        let outputs = self.spend(&mut ctx, vec![], &actions).await?;

        Ok((
            ctx.take(),
            outputs
                .nfts
                .into_values()
                .map(|nft| nft.serialize(&ctx))
                .collect::<Result<_, DriverError>>()?,
        ))
    }

    pub async fn transfer_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
        clawback: Option<u64>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let sender_puzzle_hash = self.change_p2_puzzle_hash().await?;
        let is_external = !self.db.is_custody_p2_puzzle_hash(puzzle_hash).await?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for nft_id in nft_ids {
            let amount = self
                .db
                .nft(nft_id)
                .await?
                .ok_or(WalletError::MissingNft(nft_id))?
                .coin
                .amount;

            let clawback = clawback.map(|seconds| {
                ClawbackV2::new(sender_puzzle_hash, puzzle_hash, seconds, amount, true)
            });

            let memos = calculate_memos(
                &mut ctx,
                if let Some(clawback) = clawback {
                    Hint::Clawback(clawback)
                } else {
                    Hint::P2PuzzleHash(puzzle_hash)
                },
                vec![],
            )?;

            let p2_puzzle_hash = if let Some(clawback) = clawback {
                clawback.tree_hash().into()
            } else {
                puzzle_hash
            };

            if is_external {
                actions.push(Action::update_nft(
                    Id::Existing(nft_id),
                    vec![],
                    Some(TransferNftById::default()),
                ));
            }

            actions.push(Action::send(
                Id::Existing(nft_id),
                p2_puzzle_hash,
                amount,
                memos,
            ));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }

    pub async fn add_nft_uri(
        &self,
        nft_id: Bytes32,
        fee: u64,
        uri: MetadataUpdate,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let spend = uri.spend(&mut ctx)?;

        self.spend(
            &mut ctx,
            vec![],
            &[
                Action::fee(fee),
                Action::update_nft(Id::Existing(nft_id), vec![spend], None),
            ],
        )
        .await?;

        Ok(ctx.take())
    }

    pub async fn assign_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        did_id: Option<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for nft_id in nft_ids {
            actions.push(Action::update_nft(
                Id::Existing(nft_id),
                vec![],
                Some(TransferNftById::new(did_id.map(Id::Existing), vec![])),
            ));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chia_wallet_sdk::driver::UriKind;
    use test_log::test;
    use tokio::time::sleep;

    use crate::TestWallet;

    use super::*;

    #[test(tokio::test)]
    async fn test_mint_nft() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts) = test
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
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let puzzle_hash = test.wallet.change_p2_puzzle_hash().await?;

        let nft = nfts.remove(0);

        for item in [
            MetadataUpdate {
                kind: UriKind::Data,
                uri: "abc".to_string(),
            },
            MetadataUpdate {
                kind: UriKind::Metadata,
                uri: "xyz".to_string(),
            },
            MetadataUpdate {
                kind: UriKind::License,
                uri: "123".to_string(),
            },
        ] {
            let coin_spends = test
                .wallet
                .add_nft_uri(nft.info.launcher_id, 0, item)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, None)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        let nft = test
            .wallet
            .db
            .nft(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(nft.info.current_owner, Some(did.info.launcher_id));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_nft_internal() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts) = test
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
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let puzzle_hash = test.wallet.change_p2_puzzle_hash().await?;

        let nft = nfts.remove(0);

        let coin_spends = test
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, None)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let nft = test
            .wallet
            .db
            .nft(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(nft.info.current_owner, Some(did.info.launcher_id));

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

        let (coin_spends, mut nfts) = alice
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
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let puzzle_hash = bob.wallet.change_p2_puzzle_hash().await?;

        let nft = nfts.remove(0);

        let coin_spends = alice
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, None)
            .await?;
        alice.transact(coin_spends).await?;
        bob.wait_for_puzzles().await;

        let row = bob
            .wallet
            .db
            .nft(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(row.info.current_owner, None);

        let coin_spends = bob
            .wallet
            .assign_nfts(
                vec![nft.info.launcher_id],
                Some(bob_did.info.launcher_id),
                0,
            )
            .await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let coin_spends = bob
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0, None)
            .await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let row = bob
            .wallet
            .db
            .nft(nft.info.launcher_id)
            .await?
            .expect("missing nft");
        assert_eq!(row.info.current_owner, Some(bob_did.info.launcher_id));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_assign_nft() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts) = test
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
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let nft = nfts.remove(0);

        let coin_spends = test
            .wallet
            .assign_nfts(vec![nft.info.launcher_id], None, 0)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let coin_spends = test
            .wallet
            .assign_nfts(vec![nft.info.launcher_id], Some(did.info.launcher_id), 0)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_nft_with_clawback_self() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts) = test
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
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let nft_id = nfts.remove(0).info.launcher_id;

        let timestamp = test.new_block_with_current_time().await?;

        let coin_spends = test
            .wallet
            .transfer_nfts(vec![nft_id], test.puzzle_hash, 0, Some(timestamp + 5))
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert!(test.wallet.db.spendable_nft(nft_id).await?.is_some());

        sleep(Duration::from_secs(6)).await;
        test.new_block_with_current_time().await?;

        assert!(test.wallet.db.spendable_nft(nft_id).await?.is_some());

        let coin_spends = test
            .wallet
            .transfer_nfts(vec![nft_id], test.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert!(test.wallet.db.nft(nft_id).await?.is_some());
        assert!(test.wallet.db.spendable_nft(nft_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_nft_with_clawback_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft_id = nfts.remove(0).info.launcher_id;

        let timestamp = alice.new_block_with_current_time().await?;

        let coin_spends = alice
            .wallet
            .transfer_nfts(vec![nft_id], bob.puzzle_hash, 0, Some(timestamp + 5))
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert!(alice.wallet.db.spendable_nft(nft_id).await?.is_some());
        assert!(bob.wallet.db.spendable_nft(nft_id).await?.is_none());

        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_nft(nft_id).await?.is_some());
        assert!(bob.wallet.db.spendable_nft(nft_id).await?.is_none());

        sleep(Duration::from_secs(6)).await;
        bob.new_block_with_current_time().await?;

        assert!(alice.wallet.db.spendable_nft(nft_id).await?.is_none());
        assert!(bob.wallet.db.spendable_nft(nft_id).await?.is_some());

        let coin_spends = bob
            .wallet
            .transfer_nfts(vec![nft_id], alice.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_nft(nft_id).await?.is_some());
        assert!(bob.wallet.db.spendable_nft(nft_id).await?.is_none());

        Ok(())
    }
}
