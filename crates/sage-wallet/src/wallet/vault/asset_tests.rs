use std::time::Duration;

use chia_wallet_sdk::{
    chia::puzzle_types::nft::NftMetadata,
    driver::{MetadataUpdate, UriKind, VaultVersion},
    prelude::*,
};
use sage_database::{AssetFilter, CoinFilterMode, CoinSortMode};
use test_log::test;
use tokio::time::sleep;

use crate::{TestWallet, VaultSingletonKind, Wallet, WalletNftMint, WalletOptionMint};

use super::tests::{config, mint, vault_transact};

/// A vault with XCH at its first address, minted from alice's wallet.
async fn vault_with_balance(balance: u64) -> anyhow::Result<(TestWallet, TestWallet)> {
    let mut alice = TestWallet::new(balance * 2 + 10).await?;
    let vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        balance,
    )
    .await?;
    Ok((alice, vault))
}

fn nft_mint(p2_puzzle_hash: Option<Bytes32>) -> WalletNftMint {
    WalletNftMint {
        metadata: NftMetadata {
            data_uris: vec!["https://example.com/nft.png".to_string()],
            ..Default::default()
        },
        p2_puzzle_hash,
        royalty_puzzle_hash: None,
        royalty_basis_points: 300,
    }
}

#[test(tokio::test)]
async fn test_vault_dids() -> anyhow::Result<()> {
    let (mut alice, mut vault) = vault_with_balance(1000).await?;

    let (coin_spends, did) = vault.wallet.create_did(10).await?;
    vault_transact(&mut vault, coin_spends).await?;

    let did_id = did.info.launcher_id;
    let row = vault
        .wallet
        .db
        .did(did_id)
        .await?
        .expect("the DID is synced");
    assert_eq!(row.info.p2_puzzle_hash, vault.puzzle_hash);
    assert_eq!(vault.wallet.db.xch_balance().await?, 989);

    let coin_spends = vault.wallet.normalize_dids(vec![did_id], 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    assert!(vault.wallet.db.did(did_id).await?.is_some());

    let coin_spends = vault
        .wallet
        .transfer_dids(vec![did_id], alice.puzzle_hash, 0, None)
        .await?;

    // The vault's approval of the DID's spend is checked, and shows where it goes.
    let summary = vault
        .wallet
        .parse_vault_spend(&coin_spends)
        .await?
        .expect("the vault is spent");
    assert_eq!(summary.singletons.len(), 1);
    assert_eq!(summary.singletons[0].launcher_id, did_id);
    assert_eq!(
        summary.singletons[0].child_p2_puzzle_hash,
        Some(alice.puzzle_hash)
    );

    vault_transact(&mut vault, coin_spends).await?;
    assert!(vault.wallet.db.did(did_id).await?.is_none());

    alice
        .wait_until(|wallet| async move { Ok(wallet.db.did(did_id).await?.is_some()) })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_nfts_from_did() -> anyhow::Result<()> {
    let (mut alice, mut vault) = vault_with_balance(1000).await?;

    let (coin_spends, did) = vault.wallet.create_did(0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    let did_id = did.info.launcher_id;

    let (coin_spends, nfts) = vault
        .wallet
        .bulk_mint_nfts(0, did_id, vec![nft_mint(None), nft_mint(None)])
        .await?;

    // The DID mints the NFTs and is recreated at the vault's address.
    let summary = vault
        .wallet
        .parse_vault_spend(&coin_spends)
        .await?
        .expect("the vault is spent");
    assert_eq!(summary.unaccounted_xch, 0);
    assert_eq!(summary.singletons.len(), 1);
    assert_eq!(summary.singletons[0].kind, VaultSingletonKind::Did);
    assert_eq!(
        summary.singletons[0].child_p2_puzzle_hash,
        Some(vault.puzzle_hash)
    );

    vault_transact(&mut vault, coin_spends).await?;

    let nft_id = nfts[0].info.launcher_id;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.nft(nft_id).await?.is_some()) })
        .await?;
    let row = vault
        .wallet
        .db
        .nft(nft_id)
        .await?
        .expect("the NFT is synced");
    assert_eq!(row.info.p2_puzzle_hash, vault.puzzle_hash);
    assert_eq!(row.info.current_owner, Some(did_id));

    let coin_spends = vault
        .wallet
        .add_nft_uri(
            nft_id,
            0,
            MetadataUpdate {
                kind: UriKind::Metadata,
                uri: "https://example.com/metadata.json".to_string(),
            },
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    let coin_spends = vault.wallet.assign_nfts(vec![nft_id], None, 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move {
            Ok(wallet
                .db
                .nft(nft_id)
                .await?
                .is_some_and(|nft| nft.info.current_owner.is_none()))
        })
        .await?;

    let coin_spends = vault
        .wallet
        .assign_nfts(vec![nft_id], Some(did_id), 0)
        .await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move {
            Ok(wallet
                .db
                .nft(nft_id)
                .await?
                .is_some_and(|nft| nft.info.current_owner == Some(did_id)))
        })
        .await?;

    let coin_spends = vault
        .wallet
        .transfer_nfts(vec![nft_id], alice.puzzle_hash, 0, None)
        .await?;
    vault_transact(&mut vault, coin_spends).await?;
    assert!(vault.wallet.db.nft(nft_id).await?.is_none());

    alice
        .wait_until(|wallet| async move { Ok(wallet.db.nft(nft_id).await?.is_some()) })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_issue_cat() -> anyhow::Result<()> {
    let (mut alice, mut vault) = vault_with_balance(1000).await?;

    let (coin_spends, asset_id) = vault.wallet.issue_cat(400, 10, None).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 400) })
        .await?;
    assert_eq!(vault.wallet.db.xch_balance().await?, 590);

    let coin_spends = vault
        .wallet
        .send_cat(
            asset_id,
            vec![(alice.puzzle_hash, 100)],
            0,
            true,
            vec![],
            None,
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;
    alice
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 100) })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_options() -> anyhow::Result<()> {
    let (mut alice, mut vault) = vault_with_balance(5000).await?;

    let (coin_spends, asset_id) = vault.wallet.issue_cat(1000, 0, None).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 1000) })
        .await?;

    let timestamp = vault.new_block_with_current_time().await?;

    let mint = |amount| WalletOptionMint {
        expiration_seconds: timestamp + 1000,
        underlying_type: OptionType::Cat { asset_id, amount },
        strike_type: OptionType::Xch { amount: 200 },
    };

    // The vault mints two options, and keeps one to exercise itself.
    let (coin_spends, kept) = vault.wallet.mint_option(mint(300), 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    let kept_id = kept.info.launcher_id;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.option(kept_id).await?.is_some()) })
        .await?;

    let (coin_spends, sold) = vault.wallet.mint_option(mint(400), 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    let sold_id = sold.info.launcher_id;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.option(sold_id).await?.is_some()) })
        .await?;

    let coin_spends = vault
        .wallet
        .transfer_options(vec![sold_id], alice.puzzle_hash, 0, None)
        .await?;
    let summary = vault
        .wallet
        .parse_vault_spend(&coin_spends)
        .await?
        .expect("the vault is spent");
    assert_eq!(summary.singletons.len(), 1);
    assert_eq!(summary.singletons[0].kind, VaultSingletonKind::Option);
    assert_eq!(
        summary.singletons[0].child_p2_puzzle_hash,
        Some(alice.puzzle_hash)
    );
    vault_transact(&mut vault, coin_spends).await?;

    // Alice exercises the option she was sold, paying the vault the strike.
    alice
        .wait_until(|wallet| async move { Ok(wallet.db.option(sold_id).await?.is_some()) })
        .await?;
    let coin_spends = alice.wallet.exercise_options(vec![sold_id], 0).await?;
    alice.transact(coin_spends).await?;
    alice
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 400) })
        .await?;

    // The vault exercises the option it kept, which melts it.
    let coin_spends = vault.wallet.exercise_options(vec![kept_id], 0).await?;
    let summary = vault
        .wallet
        .parse_vault_spend(&coin_spends)
        .await?
        .expect("the vault is spent");
    assert_eq!(summary.singletons.len(), 1);
    assert_eq!(summary.singletons[0].child_p2_puzzle_hash, None);
    vault_transact(&mut vault, coin_spends).await?;

    vault
        .wait_until(|wallet| async move {
            // The 300 that was never locked up, and the 300 underlying the exercised option.
            Ok(wallet.db.option(kept_id).await?.is_none()
                && wallet.db.cat_balance(asset_id).await? == 600)
        })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_cancel_option() -> anyhow::Result<()> {
    let (_alice, mut vault) = vault_with_balance(5000).await?;

    let timestamp = vault.new_block_with_current_time().await?;

    let (coin_spends, option) = vault
        .wallet
        .mint_option(
            WalletOptionMint {
                expiration_seconds: timestamp + 1000,
                underlying_type: OptionType::Xch { amount: 1000 },
                strike_type: OptionType::Xch { amount: 200 },
            },
            0,
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;
    let option_id = option.info.launcher_id;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.option(option_id).await?.is_some()) })
        .await?;

    let coin_spends = vault.wallet.cancel_options(vec![option_id], 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move {
            Ok(wallet.db.option(option_id).await?.is_none()
                && wallet.db.xch_balance().await? == 5000)
        })
        .await?;

    Ok(())
}

/// The clawback coins the wallet has, whether it's the sender or the receiver.
async fn clawback_coins(test: &TestWallet) -> anyhow::Result<Vec<Coin>> {
    clawback_coins_of(&test.wallet).await
}

async fn clawback_coins_of(wallet: &Wallet) -> anyhow::Result<Vec<Coin>> {
    let (rows, _) = wallet
        .db
        .coin_records(
            AssetFilter::Id(Bytes32::default()),
            100,
            0,
            CoinSortMode::CreatedHeight,
            false,
            CoinFilterMode::Clawback,
        )
        .await?;

    Ok(rows.into_iter().map(|row| row.coin).collect())
}

#[test(tokio::test)]
async fn test_vault_claws_back() -> anyhow::Result<()> {
    let (mut alice, mut vault) = vault_with_balance(5000).await?;

    let timestamp = vault.new_block_with_current_time().await?;
    let coin_spends = vault
        .wallet
        .send_xch(
            vec![(alice.puzzle_hash, 1000)],
            0,
            vec![],
            Some(timestamp + 1000),
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    vault
        .wait_until(|wallet| async move { Ok(wallet.db.xch_balance().await? == 4000) })
        .await?;

    let clawbacks = clawback_coins(&vault).await?;
    assert_eq!(clawbacks.len(), 1);
    alice
        .wait_until(|wallet| async move { Ok(!clawback_coins_of(&wallet).await?.is_empty()) })
        .await?;

    // The vault is the sender, so it can claw the coin back before it expires.
    let coin_spends = vault
        .wallet
        .send_xch_with_coins(
            vec![(vault.puzzle_hash, 1000)],
            0,
            vec![],
            None,
            vec![clawbacks[0].coin_id()],
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    vault
        .wait_until(|wallet| async move { Ok(wallet.db.xch_balance().await? == 5000) })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_receives_clawbacks() -> anyhow::Result<()> {
    let (alice, mut vault) = vault_with_balance(1000).await?;

    let timestamp = alice.new_block_with_current_time().await?;

    // Both are sent in one transaction, since alice's change from a first send might not be
    // synced in time for a second.
    let coin_spends = alice
        .wallet
        .send_xch(
            vec![(vault.puzzle_hash, 300), (vault.puzzle_hash, 400)],
            0,
            vec![],
            Some(timestamp + 5),
        )
        .await?;
    alice.transact(coin_spends).await?;
    vault
        .wait_until(|wallet| async move { Ok(clawback_coins_of(&wallet).await?.len() == 2) })
        .await?;

    sleep(Duration::from_secs(6)).await;
    vault.new_block_with_current_time().await?;

    let clawbacks = clawback_coins(&vault).await?;
    let claimed = clawbacks.iter().find(|coin| coin.amount == 300).copied();
    let pushed = clawbacks.iter().find(|coin| coin.amount == 400).copied();

    // Once it expires, the vault claims one as the receiver.
    let coin_spends = vault
        .wallet
        .send_xch_with_coins(
            vec![(vault.puzzle_hash, 300)],
            0,
            vec![],
            None,
            vec![claimed.expect("the clawback is synced").coin_id()],
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    // And the other is pushed through to it, which doesn't need the vault at all.
    let coin_spends = vault
        .wallet
        .finalize_clawback(vec![pushed.expect("the clawback is synced").coin_id()], 0)
        .await?;
    vault.transact(coin_spends).await?;

    vault
        .wait_until(|wallet| async move { Ok(wallet.db.selectable_xch_balance().await? == 1700) })
        .await?;

    Ok(())
}

/// NFTs minted by the vault without a DID have the vault as their minter.
#[test(tokio::test)]
async fn test_vault_mints_nfts() -> anyhow::Result<()> {
    let (alice, mut vault) = vault_with_balance(1000).await?;
    let mut alice = alice;
    let launcher_id = vault.wallet.vault_launcher_id().expect("a vault wallet");

    let (coin_spends, nfts) = vault
        .wallet
        .mint_vault_nfts(10, vec![nft_mint(None), nft_mint(Some(alice.puzzle_hash))])
        .await?;

    // The launchers' mojos are created by the vault, so none of its XCH is unaccounted for.
    let summary = vault
        .wallet
        .parse_vault_spend(&coin_spends)
        .await?
        .expect("the vault is spent");
    assert!(!summary.changes_custody());
    assert_eq!(summary.reserved_fee, 10);
    assert_eq!(summary.unaccounted_xch, 0);

    vault_transact(&mut vault, coin_spends).await?;

    let kept = nfts[0].info.launcher_id;
    let sent = nfts[1].info.launcher_id;

    vault
        .wait_until(|wallet| async move {
            Ok(wallet
                .db
                .wallet_nft(kept)
                .await?
                .is_some_and(|nft| nft.nft_info.minter_hash == Some(launcher_id)))
        })
        .await?;

    // Two mojos went to the launchers, along with the fee.
    assert_eq!(vault.wallet.db.xch_balance().await?, 988);

    alice
        .wait_until(|wallet| async move {
            Ok(wallet
                .db
                .wallet_nft(sent)
                .await?
                .is_some_and(|nft| nft.nft_info.minter_hash == Some(launcher_id)))
        })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_issues_more_cat() -> anyhow::Result<()> {
    let (_alice, mut vault) = vault_with_balance(1000).await?;

    let (coin_spends, asset_id) = vault.wallet.issue_vault_cat(300, 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 300) })
        .await?;

    let coin_spends = vault.wallet.issue_more_vault_cat(asset_id, 200, 0).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 500) })
        .await?;

    assert_eq!(vault.wallet.db.xch_balance().await?, 500);

    Ok(())
}
