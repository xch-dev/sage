//! Singletons the caller has already selected are spent as selected.
//!
//! `select_spends` skips the wallet lookup for a DID, NFT or option that is already in `spends`,
//! rather than fetching it again and replacing the caller's spend. Taking an offer relies on this
//! for NFTs that come from the offer instead of the wallet, but it applies to every transaction
//! builder that passes selected coins to `Wallet::spend`. These tests cover that path outside of
//! offers.

use chia_wallet_sdk::{chia::puzzle_types::nft::NftMetadata, driver::OptionType, prelude::*};
use test_log::test;

use crate::{TestWallet, WalletNftMint, WalletOptionMint};

#[test(tokio::test)]
async fn test_send_selected_nft() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
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
                royalty_puzzle_hash: None,
                royalty_basis_points: 300,
            }],
        )
        .await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    let nft = nfts.remove(0);
    let launcher_id = nft.info.launcher_id;
    let coin_id = alice
        .wallet
        .db
        .nft(launcher_id)
        .await?
        .expect("missing nft")
        .coin
        .coin_id();

    let mut ctx = SpendContext::new();
    let memos = ctx.hint(bob.puzzle_hash)?;
    alice
        .wallet
        .spend(
            &mut ctx,
            vec![coin_id],
            &[Action::send(
                Id::Existing(launcher_id),
                bob.puzzle_hash,
                1,
                memos,
            )],
        )
        .await?;
    alice.transact(ctx.take()).await?;
    alice.wait_for_coins().await;
    bob.wait_for_puzzles().await;

    assert!(alice.wallet.db.nft(launcher_id).await?.is_none());
    assert!(bob.wallet.db.nft(launcher_id).await?.is_some());

    Ok(())
}

#[test(tokio::test)]
async fn test_send_selected_did() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(1).await?;
    let mut bob = alice.next(0).await?;

    let (coin_spends, did) = alice.wallet.create_did(0).await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    let launcher_id = did.info.launcher_id;
    let coin_id = alice
        .wallet
        .db
        .did(launcher_id)
        .await?
        .expect("missing did")
        .coin
        .coin_id();

    let mut ctx = SpendContext::new();
    let memos = ctx.hint(bob.puzzle_hash)?;
    alice
        .wallet
        .spend(
            &mut ctx,
            vec![coin_id],
            &[Action::send(
                Id::Existing(launcher_id),
                bob.puzzle_hash,
                1,
                memos,
            )],
        )
        .await?;
    alice.transact(ctx.take()).await?;
    alice.wait_for_coins().await;
    bob.wait_for_puzzles().await;

    assert!(alice.wallet.db.did(launcher_id).await?.is_none());
    assert!(bob.wallet.db.did(launcher_id).await?.is_some());

    Ok(())
}

#[test(tokio::test)]
async fn test_send_selected_option() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(1001).await?;
    let mut bob = alice.next(0).await?;

    let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    let timestamp = alice.new_block_with_current_time().await?;

    let (coin_spends, option) = alice
        .wallet
        .mint_option(
            WalletOptionMint {
                expiration_seconds: timestamp + 1000,
                underlying_type: OptionType::Cat {
                    asset_id,
                    amount: 1000,
                },
                strike_type: OptionType::Xch { amount: 2000 },
            },
            0,
        )
        .await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    let launcher_id = option.info.launcher_id;
    let coin_id = alice
        .wallet
        .db
        .option(launcher_id)
        .await?
        .expect("missing option")
        .coin
        .coin_id();

    let mut ctx = SpendContext::new();
    let memos = ctx.hint(bob.puzzle_hash)?;
    alice
        .wallet
        .spend(
            &mut ctx,
            vec![coin_id],
            &[Action::send(
                Id::Existing(launcher_id),
                bob.puzzle_hash,
                1,
                memos,
            )],
        )
        .await?;
    alice.transact(ctx.take()).await?;
    alice.wait_for_coins().await;
    bob.wait_for_puzzles().await;

    assert!(alice.wallet.db.option(launcher_id).await?.is_none());
    assert!(bob.wallet.db.option(launcher_id).await?.is_some());

    Ok(())
}
