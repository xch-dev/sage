use chia::{
    protocol::{Bytes32, CoinSpend},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::driver::{HashedPtr, MetadataUpdate, Nft};

use crate::WalletError;

use super::{AssignDid, Id, MintNftAction, SpendAction, TransferNftAction, Wallet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletNftMint {
    pub metadata: NftMetadata,
    pub p2_puzzle_hash: Option<Bytes32>,
    pub royalty_puzzle_hash: Option<Bytes32>,
    pub royalty_ten_thousandths: u16,
}

impl Wallet {
    pub async fn bulk_mint_nfts(
        &self,
        fee: u64,
        did_id: Bytes32,
        mints: Vec<WalletNftMint>,
    ) -> Result<(Vec<CoinSpend>, Vec<Nft<HashedPtr>>), WalletError> {
        let change_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let actions = mints
            .into_iter()
            .map(|mint| {
                SpendAction::MintNft(MintNftAction::new(
                    mint.metadata,
                    mint.royalty_puzzle_hash.unwrap_or(change_puzzle_hash),
                    mint.royalty_ten_thousandths,
                    mint.p2_puzzle_hash.unwrap_or(change_puzzle_hash),
                    Id::Existing(did_id),
                ))
            })
            .collect();

        let result = self.transact(actions, fee).await?;

        Ok((
            result.coin_spends,
            result.new_assets.nfts.into_values().collect(),
        ))
    }

    pub async fn transfer_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let unassign_did = !self.db.is_p2_puzzle_hash(puzzle_hash).await?;

        let actions = nft_ids
            .into_iter()
            .map(|nft_id| {
                SpendAction::TransferNft(TransferNftAction::new(
                    Id::Existing(nft_id),
                    puzzle_hash,
                    if unassign_did {
                        AssignDid::None
                    } else {
                        AssignDid::Existing
                    },
                    None,
                ))
            })
            .collect();

        Ok(self.transact(actions, fee).await?.coin_spends)
    }

    pub async fn assign_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        did_id: Option<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let actions = nft_ids
            .into_iter()
            .map(|nft_id| {
                SpendAction::TransferNft(TransferNftAction::new(
                    Id::Existing(nft_id),
                    puzzle_hash,
                    if let Some(did_id) = did_id {
                        AssignDid::New(Id::Existing(did_id))
                    } else {
                        AssignDid::Existing
                    },
                    None,
                ))
            })
            .collect();

        Ok(self.transact(actions, fee).await?.coin_spends)
    }

    pub async fn add_nft_uri(
        &self,
        nft_id: Bytes32,
        fee: u64,
        uri: MetadataUpdate,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        Ok(self
            .transact(
                vec![SpendAction::TransferNft(TransferNftAction::new(
                    Id::Existing(nft_id),
                    puzzle_hash,
                    AssignDid::Existing,
                    Some(uri),
                ))],
                fee,
            )
            .await?
            .coin_spends)
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

        let (coin_spends, mut nfts) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
                }],
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
                .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0)
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

        let (coin_spends, mut nfts) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
                }],
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let puzzle_hash = test.wallet.p2_puzzle_hash(false, true).await?;

        let nft = nfts.remove(0);

        let coin_spends = test
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0)
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

        let (coin_spends, mut nfts) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                alice_did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
                }],
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let puzzle_hash = bob.wallet.p2_puzzle_hash(false, true).await?;

        let nft = nfts.remove(0);

        let coin_spends = alice
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0)
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
            )
            .await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let coin_spends = bob
            .wallet
            .transfer_nfts(vec![nft.info.launcher_id], puzzle_hash, 0)
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

        let (coin_spends, mut nfts) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
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
}
