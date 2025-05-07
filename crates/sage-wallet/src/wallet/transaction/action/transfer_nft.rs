use chia::protocol::{Bytes, Bytes32};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, Spends, Summary, WalletError};

use super::Hint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferNftAction {
    pub nft_id: Id,
    pub puzzle_hash: Bytes32,
    pub include_hint: Hint,
    pub memos: Option<Vec<Bytes>>,
    pub settlement_nonce: Option<Bytes32>,
}

impl TransferNftAction {
    pub fn new(
        nft_id: Id,
        puzzle_hash: Bytes32,
        include_hint: Hint,
        memos: Option<Vec<Bytes>>,
        settlement_nonce: Option<Bytes32>,
    ) -> Self {
        Self {
            nft_id,
            puzzle_hash,
            include_hint,
            memos,
            settlement_nonce,
        }
    }
}

impl Action for TransferNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let item = spends
            .nfts
            .get_mut(&self.nft_id)
            .ok_or(WalletError::MissingAsset)?;

        if let Some(nonce) = self.settlement_nonce {
            if item.current().p2().is_standard() {
                item.set_p2_puzzle_hash(SETTLEMENT_PAYMENT_HASH.into());
                item.set_did_owner(ctx, None, Vec::new())?;
                item.recreate(ctx)?;
            }
            item.set_settlement_nonce(nonce);
        }

        item.set_p2_puzzle_hash(self.puzzle_hash);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{SpendAction, TestWallet, WalletNftMint};

    use chia::puzzles::nft::NftMetadata;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_action_transfer_nft() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, nfts) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: None,
                    royalty_ten_thousandths: 0,
                }],
            )
            .await?;
        let nft_id = nfts[0].info.launcher_id;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(vec![SpendAction::transfer_nft(nft_id, bob.puzzle_hash)], 0)
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_nft(nft_id).await?.is_none());
        assert!(bob.wallet.db.spendable_nft(nft_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_transfer_multiple_nfts() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, nfts) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: None,
                        royalty_ten_thousandths: 0,
                    },
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: None,
                        royalty_ten_thousandths: 0,
                    },
                ],
            )
            .await?;
        let nft_one = nfts[0].info.launcher_id;
        let nft_two = nfts[1].info.launcher_id;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::transfer_nft(nft_one, bob.puzzle_hash),
                    SpendAction::transfer_nft(nft_two, bob.puzzle_hash),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_nft(nft_one).await?.is_none());
        assert!(alice.wallet.db.spendable_nft(nft_two).await?.is_none());
        assert!(bob.wallet.db.spendable_nft(nft_one).await?.is_some());
        assert!(bob.wallet.db.spendable_nft(nft_two).await?.is_some());

        Ok(())
    }
}
