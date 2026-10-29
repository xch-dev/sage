use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use chia_wallet_sdk::{prelude::*, utils::Address};
use sage_api::{
    AddWatchAddresses, Amount, CreateDid, DeleteWallet, GetCats, GetDerivations, GetDids, GetKeys,
    GetNfts, GetOptions, GetSyncStatus, GetWallet, GetWalletAddress, ImportWatchWallet,
    IncreaseDerivationIndex, IssueCat, Login, MintOption, NftSortMode, OptionAsset, OptionFilter,
    OptionSortMode, RemoveWatchAddresses, SendXch, SignCoinSpends, TransferNfts, WalletKind,
};
use sage_wallet::SyncEvent;
use tokio::time::sleep;

use super::{TestApp, mint_mutable_nft};

/// What a wallet holds, as shown over the RPC.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Holdings {
    xch: Option<u64>,
    cats: Vec<(Option<String>, Option<u64>)>,
    nfts: Vec<String>,
    dids: Vec<String>,
    options: Vec<String>,
}

async fn holdings(app: &TestApp) -> Result<Holdings> {
    let xch = app
        .get_sync_status(GetSyncStatus {})
        .await?
        .selectable_balance
        .to_u64();

    let mut cats: Vec<_> = app
        .get_cats(GetCats {})
        .await?
        .cats
        .into_iter()
        .map(|cat| (cat.asset_id, cat.balance.to_u64()))
        .collect();
    cats.sort();

    let mut nfts: Vec<_> = app
        .get_nfts(GetNfts {
            collection_id: None,
            minter_did_id: None,
            owner_did_id: None,
            name: None,
            offset: 0,
            limit: 100,
            sort_mode: NftSortMode::Name,
            include_hidden: true,
        })
        .await?
        .nfts
        .into_iter()
        .map(|nft| nft.launcher_id)
        .collect();
    nfts.sort();

    let mut dids: Vec<_> = app
        .get_dids(GetDids {})
        .await?
        .dids
        .into_iter()
        .map(|did| did.launcher_id)
        .collect();
    dids.sort();

    let mut options: Vec<_> = app
        .get_options(GetOptions {
            offset: 0,
            limit: 100,
            sort_mode: OptionSortMode::Name,
            ascending: true,
            find_value: None,
            include_hidden: true,
            filter: OptionFilter::Owned,
        })
        .await?
        .options
        .into_iter()
        .map(|option| option.launcher_id)
        .collect();
    options.sort();

    Ok(Holdings {
        xch,
        cats,
        nfts,
        dids,
        options,
    })
}

/// Waits for the open wallet to sync to the expected holdings.
async fn wait_for_holdings(app: &mut TestApp, expected: &Holdings) -> Result<()> {
    for _ in 0..300 {
        // Nothing else reads sync events here, so they're drained to keep sync going.
        while app.events.try_recv().is_ok() {}

        if holdings(app).await? == *expected {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    bail!(
        "expected {expected:?}, but the wallet holds {:?}",
        holdings(app).await?
    );
}

/// A BLS wallet holding XCH, a CAT, a DID, an NFT and an option. Returns its fingerprint, its
/// holdings and the addresses it has derived.
async fn alice_with_assets(app: &mut TestApp) -> Result<(u32, Holdings, Vec<String>)> {
    let alice = app.setup_bls(10_000).await?;

    app.issue_cat(IssueCat {
        name: "Watched".to_string(),
        ticker: "WCH".to_string(),
        amount: Amount::u64(1000),
        revocable: false,
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;
    app.wait_for_coins().await;

    let cat_id = app.get_cats(GetCats {}).await?.cats[0].asset_id.clone();

    // Minting the NFT creates a DID too.
    mint_mutable_nft(app).await?;

    let expiration_seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 86_400;

    app.mint_option(MintOption {
        expiration_seconds,
        underlying: OptionAsset {
            asset_id: None,
            amount: Amount::u64(100),
        },
        strike: OptionAsset {
            asset_id: cat_id,
            amount: Amount::u64(10),
        },
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;
    app.wait_for_coins().await;

    let holdings = holdings(app).await?;
    assert_eq!(holdings.cats.len(), 1);
    assert_eq!(holdings.nfts.len(), 1);
    assert_eq!(holdings.dids.len(), 1);
    assert_eq!(holdings.options.len(), 1);

    let addresses = app
        .get_derivations(GetDerivations {
            hardened: false,
            offset: 0,
            limit: 10_000,
        })
        .await?
        .derivations
        .into_iter()
        .map(|derivation| derivation.address)
        .collect();

    Ok((alice, holdings, addresses))
}

/// Like [`alice_with_assets`], but with the NFT moved to an address of its own. Returns Alice's
/// fingerprint, her holdings, the NFT's address and the rest of her addresses.
async fn alice_with_nft_apart(app: &mut TestApp) -> Result<(u32, Holdings, String, Vec<String>)> {
    let (alice, expected, addresses) = alice_with_assets(app).await?;

    // Everything is at Alice's first address, so the NFT is moved to an address of its own.
    let nft_address = addresses[5].clone();
    app.transfer_nfts(TransferNfts {
        nft_ids: expected.nfts.clone(),
        address: nft_address.clone(),
        fee: Amount::u64(0),
        clawback: None,
        auto_submit: true,
        confirmed_handles: Vec::new(),
    })
    .await?;
    app.wait_for_coins().await;

    let nfts = app
        .get_nfts(GetNfts {
            collection_id: None,
            minter_did_id: None,
            owner_did_id: None,
            name: None,
            offset: 0,
            limit: 100,
            sort_mode: NftSortMode::Name,
            include_hidden: true,
        })
        .await?
        .nfts;
    assert_eq!(nfts[0].address, nft_address);
    let expected = holdings(app).await?;

    let others: Vec<String> = addresses
        .iter()
        .filter(|address| **address != nft_address)
        .cloned()
        .collect();

    Ok((alice, expected, nft_address, others))
}

fn import(addresses: Vec<String>) -> ImportWatchWallet {
    ImportWatchWallet {
        name: "Watch".to_string(),
        addresses,
        login: true,
        emoji: None,
    }
}

#[tokio::test]
async fn watched_addresses_show_their_holdings() -> Result<()> {
    let mut app = TestApp::new().await?;

    let (_alice, expected, addresses) = alice_with_assets(&mut app).await?;

    let watch = app
        .import_watch_wallet(import(addresses.clone()))
        .await?
        .fingerprint;

    wait_for_holdings(&mut app, &expected).await?;

    let wallet = app
        .get_wallet(GetWallet { fingerprint: None })
        .await?
        .wallet
        .expect("watch wallet is missing");
    assert_eq!(wallet.fingerprint, watch);
    assert_eq!(wallet.kind, WalletKind::Watch { addresses });

    // A watch wallet has no key, so the keychain only holds Alice's key.
    assert_eq!(app.get_keys(GetKeys {}).await?.keys.len(), 1);

    Ok(())
}

#[tokio::test]
async fn watch_wallets_are_read_only() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(1000).await?;
    let alice_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let watch = app
        .import_watch_wallet(import(vec![alice_address.clone()]))
        .await?
        .fingerprint;

    wait_for_holdings(
        &mut app,
        &Holdings {
            xch: Some(1000),
            cats: Vec::new(),
            nfts: Vec::new(),
            dids: Vec::new(),
            options: Vec::new(),
        },
    )
    .await?;

    // Its first address is where it receives.
    assert_eq!(
        app.get_sync_status(GetSyncStatus {}).await?.receive_address,
        alice_address
    );
    assert_eq!(
        app.get_wallet_address(GetWalletAddress {
            fingerprint: watch,
            network_id: "testnet11".to_string(),
        })
        .await?
        .address,
        alice_address
    );

    let error = app
        .send_xch(SendXch {
            confirmed_handles: Vec::new(),
            address: alice_address.clone(),
            amount: Amount::u64(100),
            fee: Amount::u64(0),
            memos: vec![],
            clawback: None,
            auto_submit: false,
            coin_ids: None,
        })
        .await
        .expect_err("watch wallets can't send");
    assert!(error.to_string().contains("Watch-only"), "{error}");

    let error = app
        .create_did(CreateDid {
            name: "Watch".to_string(),
            fee: Amount::u64(0),
            auto_submit: false,
        })
        .await
        .expect_err("watch wallets can't create DIDs");
    assert!(error.to_string().contains("Watch-only"), "{error}");

    let error = app
        .sign_coin_spends(SignCoinSpends {
            coin_spends: Vec::new(),
            auto_submit: false,
            partial: true,
            aggregated_signature: None,
        })
        .await
        .expect_err("watch wallets can't sign");
    assert!(error.to_string().contains("Watch-only"), "{error}");

    let error = app
        .increase_derivation_index(IncreaseDerivationIndex {
            hardened: None,
            unhardened: None,
            index: 10,
        })
        .await
        .expect_err("watch wallets don't derive");
    assert!(error.to_string().contains("derive"), "{error}");

    // The coins are still Alice's to spend.
    app.login(Login { fingerprint: alice }).await?;
    app.send_xch(SendXch {
        confirmed_handles: Vec::new(),
        address: alice_address,
        amount: Amount::u64(100),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit: false,
        coin_ids: None,
    })
    .await?;

    Ok(())
}

#[tokio::test]
async fn addresses_can_be_added_and_removed() -> Result<()> {
    let mut app = TestApp::new().await?;

    let (_alice, expected, nft_address, others) = alice_with_nft_apart(&mut app).await?;
    let addresses_len = others.len() + 1;

    // Without the NFT's address, the wallet doesn't hold the NFT.
    let watch = app
        .import_watch_wallet(import(others.clone()))
        .await?
        .fingerprint;

    let without_nft = Holdings {
        nfts: Vec::new(),
        ..expected.clone()
    };
    wait_for_holdings(&mut app, &without_nft).await?;

    // Watching it syncs the NFT, without a resync.
    app.add_watch_addresses(AddWatchAddresses {
        fingerprint: watch,
        addresses: vec![nft_address.clone(), others[0].clone()],
    })
    .await?;
    wait_for_holdings(&mut app, &expected).await?;

    let wallet = app
        .get_wallet(GetWallet {
            fingerprint: Some(watch),
        })
        .await?
        .wallet
        .expect("watch wallet is missing");
    let WalletKind::Watch { addresses: watched } = wallet.kind else {
        bail!("expected a watch wallet");
    };
    assert_eq!(watched.len(), addresses_len);
    assert_eq!(watched.last(), Some(&nft_address));

    // Removing it again drops the NFT.
    app.remove_watch_addresses(RemoveWatchAddresses {
        fingerprint: watch,
        addresses: vec![nft_address],
    })
    .await?;
    wait_for_holdings(&mut app, &without_nft).await?;

    // A watch wallet needs at least one address.
    let error = app
        .remove_watch_addresses(RemoveWatchAddresses {
            fingerprint: watch,
            addresses: others,
        })
        .await
        .expect_err("the last address can't be removed");
    assert!(
        error.to_string().contains("at least one address"),
        "{error}"
    );

    Ok(())
}

#[tokio::test]
async fn rejects_invalid_watch_wallets() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(0).await?;
    let alice_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    // Mainnet addresses don't belong on testnet11.
    let mainnet = Address::new(Bytes32::new([1; 32]), "xch".to_string()).encode()?;
    let error = app
        .import_watch_wallet(import(vec![mainnet]))
        .await
        .expect_err("the prefix doesn't match the network");
    assert!(
        error.to_string().contains("Wrong address prefix"),
        "{error}"
    );

    let error = app
        .import_watch_wallet(import(Vec::new()))
        .await
        .expect_err("a watch wallet needs an address");
    assert!(
        error.to_string().contains("at least one address"),
        "{error}"
    );

    let watch = app
        .import_watch_wallet(ImportWatchWallet {
            login: false,
            ..import(vec![alice_address.clone()])
        })
        .await?
        .fingerprint;

    let error = app
        .import_watch_wallet(import(vec![alice_address.clone(), alice_address.clone()]))
        .await
        .expect_err("the same addresses are already watched");
    assert!(error.to_string().contains("already exists"), "{error}");

    let error = app
        .add_watch_addresses(AddWatchAddresses {
            fingerprint: alice,
            addresses: vec![alice_address],
        })
        .await
        .expect_err("BLS wallets don't watch addresses");
    assert!(
        error.to_string().contains("isn't a watch wallet"),
        "{error}"
    );

    // Deleting a watch wallet leaves the keychain alone.
    app.delete_wallet(DeleteWallet {
        fingerprint: watch,
        delete_keys: true,
    })
    .await?;
    assert!(
        app.get_wallet(GetWallet {
            fingerprint: Some(watch),
        })
        .await?
        .wallet
        .is_none()
    );
    assert_eq!(app.get_keys(GetKeys {}).await?.keys.len(), 1);

    Ok(())
}

#[tokio::test]
async fn addresses_added_while_closed_sync_when_opened() -> Result<()> {
    let mut app = TestApp::new().await?;

    let (alice, expected, addresses) = alice_with_assets(&mut app).await?;

    // The watch wallet starts with an address Alice never used, so it holds nothing and none of
    // its coins lead to hers.
    let unused = addresses.last().expect("Alice has addresses").clone();
    let watch = app
        .import_watch_wallet(import(vec![unused]))
        .await?
        .fingerprint;
    app.consume_until(|event| matches!(event, SyncEvent::Subscribed))
        .await;
    wait_for_holdings(
        &mut app,
        &Holdings {
            xch: Some(0),
            cats: Vec::new(),
            nfts: Vec::new(),
            dids: Vec::new(),
            options: Vec::new(),
        },
    )
    .await?;

    // Alice's address is added while another wallet is open. Opening the watch wallet again only
    // syncs from its last peak, but the new address still needs its whole history.
    app.login(Login { fingerprint: alice }).await?;
    app.add_watch_addresses(AddWatchAddresses {
        fingerprint: watch,
        addresses: vec![addresses[0].clone()],
    })
    .await?;

    app.login(Login { fingerprint: watch }).await?;
    assert!(
        app.sage
            .lock()
            .await
            .wallet()?
            .db
            .latest_peak()
            .await?
            .is_some(),
        "the watch wallet should resume from its last peak"
    );
    wait_for_holdings(&mut app, &expected).await?;

    Ok(())
}
