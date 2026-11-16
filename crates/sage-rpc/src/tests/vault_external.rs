use std::time::Duration;

use anyhow::{Result, bail};
use chia_wallet_sdk::{
    clvm_traits::clvm_quote,
    driver::{VaultMessageProof, verify_vault_signed_message},
    prelude::*,
    utils::Address,
};
use sage_api::{
    Amount, CoinJson, CoinSpendJson, GenerateMnemonic, GetDerivations, GetSyncStatus, GetVault,
    GetWalletAddress, IncreaseDerivationIndex, Login, MintVault, SendXch, SetChangeAddress,
    SignCoinSpends, VaultRecord, VaultState,
    wallet_connect::{GetAssetCoins, SignMessageByAddress, SignMessageWithPublicKey},
};
use tokio::time::sleep;

use super::TestApp;

async fn wait_for_vault(
    app: &mut TestApp,
    f: impl Fn(&VaultRecord) -> bool,
) -> Result<VaultRecord> {
    let mut last = None;

    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        let vault = app.get_vault(GetVault {}).await?.vault;

        if f(&vault) {
            return Ok(vault);
        }

        last = Some(vault);
        sleep(Duration::from_millis(100)).await;
    }

    bail!("timed out waiting for the vault, which is {last:?}");
}

async fn wait_for_balance(app: &mut TestApp, balance: u64) -> Result<()> {
    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        let status = app.get_sync_status(GetSyncStatus {}).await?;

        if status.selectable_balance.to_u64() == Some(balance) {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    let status = app.get_sync_status(GetSyncStatus {}).await?;
    bail!("timed out waiting for a balance of {balance}: {status:?}");
}

fn send(address: String, amount: u64, auto_submit: bool) -> SendXch {
    SendXch {
        confirmed_handles: Vec::new(),
        address,
        amount: Amount::u64(amount),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit,
        coin_ids: None,
    }
}

/// A vault minted by alice, which holds `balance` XCH and is open.
struct FundedVault {
    alice_address: String,
    fingerprint: u32,
    vault: VaultRecord,
    address: String,
}

async fn funded_vault(app: &mut TestApp, balance: u64) -> Result<FundedVault> {
    let alice = app.setup_bls(balance + 1000).await?;
    wait_for_balance(app, balance + 1000).await?;
    let alice_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let recovery_mnemonic = app
        .generate_mnemonic(GenerateMnemonic {
            use_24_words: false,
        })
        .await?
        .mnemonic;

    let fingerprint = app
        .mint_vault(MintVault {
            name: "Vault".to_string(),
            emoji: None,
            custody_fingerprint: None,
            recovery_mnemonic: Some(recovery_mnemonic),
            recovery_fingerprint: None,
            recovery_timelock: 12 * 60 * 60,
            fee: Amount::u64(0),
            auto_submit: true,
        })
        .await?
        .fingerprint
        .expect("the mint was submitted");

    wait_for_balance(app, balance + 1000 - 1).await?;

    app.login(Login { fingerprint }).await?;

    wait_for_vault(app, |vault| {
        vault.state == VaultState::Active && vault.config.is_some() && !vault.minting
    })
    .await?;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    app.login(Login { fingerprint: alice }).await?;

    // The simulator confirms transactions at the height of its last block, which the vault wallet
    // has already synced up to, so a new block is made first, as a real chain would.
    app.sim.lock().await.create_block();

    app.send_xch(send(address.clone(), balance, true)).await?;
    wait_for_balance(app, 1000 - 1).await?;

    app.login(Login { fingerprint }).await?;
    wait_for_balance(app, balance).await?;

    let vault = app.get_vault(GetVault {}).await?.vault;

    Ok(FundedVault {
        alice_address,
        fingerprint,
        vault,
        address,
    })
}

fn custody_hash(vault: &VaultRecord) -> Result<Bytes32> {
    let Some(custody_hash) = &vault.custody_hash else {
        bail!("the vault's custody hash isn't known");
    };
    Ok(Bytes32::try_from(hex::decode(custody_hash)?)?)
}

fn launcher_id(vault: &VaultRecord) -> Result<Bytes32> {
    Ok(Address::decode(&vault.launcher_id)?.puzzle_hash)
}

fn coin_spends_json(coin_spends: &[CoinSpend]) -> Vec<CoinSpendJson> {
    coin_spends
        .iter()
        .map(|coin_spend| CoinSpendJson {
            coin: CoinJson {
                parent_coin_info: hex::encode(coin_spend.coin.parent_coin_info),
                puzzle_hash: hex::encode(coin_spend.coin.puzzle_hash),
                amount: Amount::u64(coin_spend.coin.amount),
            },
            puzzle_reveal: hex::encode(&coin_spend.puzzle_reveal),
            solution: hex::encode(&coin_spend.solution),
        })
        .collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    Ok(hex::decode(value.trim_start_matches("0x"))?)
}

fn rust_coin_spends(coin_spends: Vec<CoinSpendJson>) -> Result<Vec<CoinSpend>> {
    coin_spends
        .into_iter()
        .map(|coin_spend| {
            let Some(amount) = coin_spend.coin.amount.to_u64() else {
                bail!("the coin amount doesn't fit in a u64");
            };

            Ok(CoinSpend::new(
                Coin::new(
                    Bytes32::try_from(decode_hex(&coin_spend.coin.parent_coin_info)?)?,
                    Bytes32::try_from(decode_hex(&coin_spend.coin.puzzle_hash)?)?,
                    amount,
                ),
                decode_hex(&coin_spend.puzzle_reveal)?.into(),
                decode_hex(&coin_spend.solution)?.into(),
            ))
        })
        .collect()
}

#[tokio::test]
async fn test_vault_addresses_as_derivations() -> Result<()> {
    let mut app = TestApp::new().await?;
    let FundedVault {
        fingerprint,
        address,
        ..
    } = funded_vault(&mut app, 5000).await?;

    // A vault's addresses are listed by nonce, without public keys. Its first address has received
    // a coin, so the address gap starts after it.
    let derivations = app
        .get_derivations(GetDerivations {
            hardened: false,
            offset: 0,
            limit: 10,
        })
        .await?;
    assert_eq!(derivations.total, 101);
    assert_eq!(derivations.derivations.len(), 10);
    assert_eq!(derivations.derivations[0].address, address);
    assert!(
        derivations
            .derivations
            .iter()
            .enumerate()
            .all(|(index, derivation)| derivation.index as usize == index
                && derivation.public_key.is_none())
    );

    let hardened = app
        .get_derivations(GetDerivations {
            hardened: true,
            offset: 0,
            limit: 10,
        })
        .await?;
    assert_eq!(hardened.total, 0);
    assert!(hardened.derivations.is_empty());

    let status = app.get_sync_status(GetSyncStatus {}).await?;
    assert_eq!(status.unhardened_derivation_index, 101);
    assert_eq!(status.hardened_derivation_index, 0);

    // More addresses can be added, and hardened ones can't.
    app.increase_derivation_index(IncreaseDerivationIndex {
        hardened: None,
        unhardened: None,
        index: 150,
    })
    .await?;
    assert_eq!(
        app.get_sync_status(GetSyncStatus {})
            .await?
            .unhardened_derivation_index,
        150
    );

    let error = app
        .increase_derivation_index(IncreaseDerivationIndex {
            hardened: Some(true),
            unhardened: None,
            index: 10,
        })
        .await
        .expect_err("vaults have no hardened addresses");
    assert!(error.to_string().contains("hardened"), "{error}");

    // A vault's change always goes back to the vault.
    let error = app
        .set_change_address(SetChangeAddress {
            fingerprint,
            change_address: Some(Address::new(Bytes32::new([3; 32]), "txch".to_string()).encode()?),
        })
        .await
        .expect_err("vaults can't use a change address");
    assert!(error.to_string().contains("change address"), "{error}");

    assert_eq!(
        app.get_wallet_address(GetWalletAddress {
            fingerprint,
            network_id: "testnet11".to_string(),
        })
        .await?
        .address,
        address
    );

    Ok(())
}

#[tokio::test]
async fn test_vault_asset_coins_have_puzzle_reveals() -> Result<()> {
    let mut app = TestApp::new().await?;
    funded_vault(&mut app, 5000).await?;

    let coins = app
        .sage
        .lock()
        .await
        .get_asset_coins(GetAssetCoins {
            kind: None,
            asset_id: None,
            included_locked: None,
            offset: None,
            limit: None,
        })
        .await?;

    assert_eq!(coins.len(), 1);

    for coin in coins {
        let puzzle = Program::from(decode_hex(&coin.puzzle)?);
        let mut allocator = Allocator::new();
        let puzzle = puzzle.to_clvm(&mut allocator)?;
        assert_eq!(
            hex::encode(tree_hash(&allocator, puzzle)),
            coin.coin.puzzle_hash
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_vault_signs_messages_by_address() -> Result<()> {
    let mut app = TestApp::new().await?;
    let FundedVault { vault, address, .. } = funded_vault(&mut app, 5000).await?;

    let response = app
        .sage
        .lock()
        .await
        .sign_message_by_address(SignMessageByAddress {
            message: hex::encode("hello"),
            address,
        })
        .await?;

    assert_eq!(response.public_key, vault.launcher_id);

    let proof = VaultMessageProof::decode_string(&response.signature)?;
    let messages = verify_vault_signed_message(
        &proof,
        launcher_id(&vault)?,
        custody_hash(&vault)?,
        TESTNET11_CONSTANTS.genesis_challenge,
        None,
    )?;

    let expected: Bytes32 = ("Chia Signed Message", Bytes::from(b"hello".to_vec()))
        .tree_hash()
        .into();
    assert_eq!(messages, vec![Bytes::from(expected.to_vec())]);

    // An address that isn't the vault's can't be signed for.
    let error = app
        .sage
        .lock()
        .await
        .sign_message_by_address(SignMessageByAddress {
            message: hex::encode("hello"),
            address: Address::new(Bytes32::new([3; 32]), "txch".to_string()).encode()?,
        })
        .await
        .expect_err("the address isn't the vault's");
    assert!(error.to_string().contains("key"), "{error}");

    // A vault's addresses aren't locked to a key.
    let error = app
        .sage
        .lock()
        .await
        .sign_message_with_public_key(SignMessageWithPublicKey {
            message: hex::encode("hello"),
            public_key: hex::encode([1; 48]),
        })
        .await
        .expect_err("vaults can't sign by public key");
    assert!(error.to_string().contains("Vault wallets can't"), "{error}");

    Ok(())
}

#[tokio::test]
async fn test_vault_signs_coin_spends() -> Result<()> {
    let mut app = TestApp::new().await?;
    let FundedVault {
        alice_address,
        vault,
        ..
    } = funded_vault(&mut app, 5000).await?;

    let coin_spends = app
        .send_xch(send(alice_address, 2000, false))
        .await?
        .coin_spends;

    // A dApp's spend that asks the vault's custody key to sign for another coin is refused.
    let Some(custody) = vault.config.as_ref() else {
        bail!("the vault's config isn't known");
    };
    let custody_key = PublicKey::from_bytes(
        &hex::decode(&custody.custody.members[0].key)?
            .try_into()
            .expect("a public key is 48 bytes"),
    )?;

    let mut allocator = Allocator::new();
    let conditions = Conditions::new().agg_sig_me(custody_key, Bytes::from(vec![7; 32]));
    let puzzle = clvm_quote!(conditions).to_clvm(&mut allocator)?;
    let puzzle_hash: Bytes32 = tree_hash(&allocator, puzzle).into();
    let other = CoinSpend::new(
        Coin::new(Bytes32::new([1; 32]), puzzle_hash, 1),
        Program::from_clvm(&allocator, puzzle)?,
        Program::default(),
    );

    let error = app
        .sign_coin_spends(SignCoinSpends {
            coin_spends: coin_spends_json(&[other]),
            auto_submit: false,
            partial: true,
            aggregated_signature: None,
        })
        .await
        .expect_err("the vault's key can't sign for another coin");
    assert!(
        error
            .to_string()
            .contains("Refusing to sign with a vault's keys"),
        "{error}"
    );

    // A second spend of the vault coin, whose delegated spend clear signing didn't check, is
    // refused.
    let mut rust_spends = rust_coin_spends(coin_spends.clone())?;
    let vault_coin_id = Bytes32::try_from(hex::decode(
        vault.coin_id.as_deref().expect("the vault coin is synced"),
    )?)?;
    let vault_spend = rust_spends
        .iter()
        .find(|coin_spend| coin_spend.coin.coin_id() == vault_coin_id)
        .cloned()
        .expect("the vault is spent");
    rust_spends.push(vault_spend);

    let error = app
        .sign_coin_spends(SignCoinSpends {
            coin_spends: coin_spends_json(&rust_spends),
            auto_submit: false,
            partial: false,
            aggregated_signature: None,
        })
        .await
        .expect_err("the vault coin can only be spent once");
    assert!(
        error
            .to_string()
            .contains("Refusing to sign with a vault's keys"),
        "{error}"
    );

    // The transaction itself is signed with the custody key, and can be submitted.
    app.sign_coin_spends(SignCoinSpends {
        coin_spends,
        auto_submit: true,
        partial: false,
        aggregated_signature: None,
    })
    .await?;

    wait_for_balance(&mut app, 3000).await?;

    Ok(())
}
