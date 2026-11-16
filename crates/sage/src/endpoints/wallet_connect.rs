use chia_wallet_sdk::driver::{Clawback as ClawbackV1, ClawbackV2};
use chia_wallet_sdk::{
    chia::{
        bls::{master_to_wallet_hardened, master_to_wallet_unhardened, sign},
        puzzle_types::{DeriveSynthetic, Proof},
    },
    driver::{P2DelegatedConditionsLayer, VaultMessageProof, vault_message_envelope},
    prelude::*,
};
use sage_api::{
    Amount,
    wallet_connect::{
        self, AssetCoinType, FilterUnlockedCoins, FilterUnlockedCoinsResponse, GetAssetCoins,
        GetAssetCoinsResponse, LineageProof, SendTransactionImmediately,
        SendTransactionImmediatelyResponse, SignMessageByAddress, SignMessageByAddressResponse,
        SignMessageWithPublicKey, SignMessageWithPublicKeyResponse, SpendableCoin,
    },
};
use sage_database::{
    AssetFilter, CoinFilterMode, CoinSortMode, ConditionsOrSingleton, DeserializePrimitive,
    P2Puzzle,
};
use sage_wallet::{
    Status, SyncCommand, Transaction, Wallet, WalletError, custody_mips_spend, insert_transaction,
    sign_with_secret_keys, submit_to_peers,
};
use tracing::{debug, info, warn};

use crate::{
    Error, Result, Sage, WalletKindName, encode_vault_id, parse_asset_id, parse_coin_id,
    parse_did_id, parse_hash, parse_nft_id, parse_program, parse_public_key, parse_signature,
    parse_signature_message,
};

impl Sage {
    pub async fn filter_unlocked_coins(
        &self,
        req: FilterUnlockedCoins,
    ) -> Result<FilterUnlockedCoinsResponse> {
        let wallet = self.wallet()?;
        let mut coin_ids = Vec::new();

        for coin_id in req.coin_ids {
            let parsed_coin_id = parse_coin_id(coin_id.clone())?;

            if wallet.db.are_coins_spendable(&[parsed_coin_id]).await? {
                coin_ids.push(coin_id);
            }
        }

        Ok(FilterUnlockedCoinsResponse { coin_ids })
    }

    pub async fn get_asset_coins(&self, req: GetAssetCoins) -> Result<GetAssetCoinsResponse> {
        let include_locked = req.included_locked.unwrap_or(false);
        let wallet = self.wallet()?;

        let mut items = Vec::new();

        let (rows, _count) = wallet
            .db
            .coin_records(
                match (req.kind, req.asset_id) {
                    (None, _) => AssetFilter::Id(Bytes32::default()),
                    (Some(AssetCoinType::Cat), None) => return Err(Error::MissingAssetId),
                    (Some(AssetCoinType::Cat), Some(asset_id)) => {
                        AssetFilter::Id(parse_asset_id(asset_id)?)
                    }
                    (Some(AssetCoinType::Did), None) => AssetFilter::Dids,
                    (Some(AssetCoinType::Did), Some(asset_id)) => {
                        AssetFilter::Id(parse_did_id(asset_id)?)
                    }
                    (Some(AssetCoinType::Nft), None) => AssetFilter::Nfts,
                    (Some(AssetCoinType::Nft), Some(asset_id)) => {
                        AssetFilter::Id(parse_nft_id(asset_id)?)
                    }
                },
                req.limit.unwrap_or(10),
                req.offset.unwrap_or(0),
                CoinSortMode::CreatedHeight,
                true,
                if include_locked {
                    CoinFilterMode::Owned
                } else {
                    CoinFilterMode::Selectable
                },
            )
            .await?;

        for row in rows {
            let mut ctx = SpendContext::new();

            let p2_puzzle = match wallet.db.p2_puzzle(row.p2_puzzle_hash).await? {
                P2Puzzle::PublicKey(key) => StandardLayer::new(key).construct_puzzle(&mut ctx)?,
                P2Puzzle::Clawback(clawback) => {
                    if clawback.version == 1 {
                        let v1 = ClawbackV1::new(
                            clawback.seconds,
                            clawback.sender_puzzle_hash,
                            clawback.receiver_puzzle_hash,
                        );
                        v1.into_1_of_n().construct_puzzle(&mut ctx)?
                    } else {
                        let v2 = ClawbackV2::new(
                            clawback.sender_puzzle_hash,
                            clawback.receiver_puzzle_hash,
                            clawback.seconds,
                            row.coin.amount,
                            // Only clawbacks of assets with an outer puzzle are hinted
                            req.kind.is_some(),
                        );
                        v2.into_1_of_n().construct_puzzle(&mut ctx)?
                    }
                }
                P2Puzzle::Option(underlying) => {
                    let underlying = OptionUnderlying::new(
                        underlying.launcher_id,
                        underlying.creator_puzzle_hash,
                        underlying.seconds,
                        underlying.amount,
                        underlying.strike_type,
                    );
                    underlying.into_1_of_n().construct_puzzle(&mut ctx)?
                }
                P2Puzzle::Arbor(key) => {
                    P2DelegatedConditionsLayer::new(key).construct_puzzle(&mut ctx)?
                }
                P2Puzzle::External => return Err(WalletError::WatchOnly.into()),
                // The MIPS puzzle doesn't depend on the delegated spend or the singleton's
                // details, which are only in the solution.
                P2Puzzle::P2Singleton(p2) => {
                    p2.spend(
                        &mut ctx,
                        Bytes32::default(),
                        1,
                        Spend::new(NodePtr::NIL, NodePtr::NIL),
                    )?
                    .puzzle
                }
                P2Puzzle::P2ConditionsOrSingleton(ConditionsOrSingleton { p2, .. }) => {
                    p2.p2_singleton_spend(
                        &mut ctx,
                        Bytes32::default(),
                        1,
                        Spend::new(NodePtr::NIL, NodePtr::NIL),
                    )?
                    .puzzle
                }
            };

            let (puzzle, proof) = match req.kind {
                None => (p2_puzzle, None),
                Some(AssetCoinType::Cat) => {
                    let Some(cat) = wallet.db.cat_coin(row.coin.coin_id()).await? else {
                        return Err(Error::MissingCatCoin(row.coin.coin_id()));
                    };
                    let puzzle = cat.info.construct_puzzle(&mut ctx, p2_puzzle)?;
                    (puzzle, cat.lineage_proof.map(Proof::Lineage))
                }
                Some(AssetCoinType::Did) => {
                    let Some(did) = wallet.db.did_coin(row.coin.coin_id()).await? else {
                        return Err(Error::MissingDidCoin(row.coin.coin_id()));
                    };
                    let did = did.deserialize(&mut ctx)?;
                    let puzzle = did.info.into_layers(p2_puzzle).construct_puzzle(&mut ctx)?;
                    (puzzle, Some(did.proof))
                }
                Some(AssetCoinType::Nft) => {
                    let Some(nft) = wallet.db.nft_coin(row.coin.coin_id()).await? else {
                        return Err(Error::MissingNftCoin(row.coin.coin_id()));
                    };
                    let nft = nft.deserialize(&mut ctx)?;
                    let puzzle = nft.info.into_layers(p2_puzzle).construct_puzzle(&mut ctx)?;
                    (puzzle, Some(nft.proof))
                }
            };

            items.push(SpendableCoin {
                coin: wallet_connect::Coin {
                    parent_coin_info: hex::encode(row.coin.parent_coin_info),
                    puzzle_hash: hex::encode(row.coin.puzzle_hash),
                    amount: Amount::u64(row.coin.amount),
                },
                coin_name: hex::encode(row.coin.coin_id()),
                puzzle: hex::encode(ctx.serialize(&puzzle)?),
                confirmed_block_index: row.created_height.unwrap_or(0),
                locked: row.mempool_item_hash.is_some() || row.offer_hash.is_some(),
                lineage_proof: match proof {
                    None => None,
                    Some(Proof::Eve(proof)) => Some(LineageProof {
                        parent_name: Some(hex::encode(proof.parent_parent_coin_info)),
                        inner_puzzle_hash: None,
                        amount: Some(Amount::u64(proof.parent_amount)),
                    }),
                    Some(Proof::Lineage(proof)) => Some(LineageProof {
                        parent_name: Some(hex::encode(proof.parent_parent_coin_info)),
                        inner_puzzle_hash: Some(hex::encode(proof.parent_inner_puzzle_hash)),
                        amount: Some(Amount::u64(proof.parent_amount)),
                    }),
                },
            });
        }

        Ok(items)
    }

    pub async fn sign_message_with_public_key(
        &self,
        req: SignMessageWithPublicKey,
    ) -> Result<SignMessageWithPublicKeyResponse> {
        let wallet = self.wallet()?;

        // A vault's addresses aren't locked to a key, so it signs messages by address instead.
        if wallet.vault_launcher_id().is_some() {
            return Err(Error::WalletKindNotSupported {
                kind: WalletKindName::Vault,
                action: "sign messages with a public key. Sign by the vault's address instead",
            });
        }

        // Checked first, so watch wallets get a clear error rather than an unknown key.
        let master_sk = self.master_secret_key(&wallet)?;

        let public_key = parse_public_key(req.public_key)?;
        let Some(info) = wallet.db.derivation(public_key).await? else {
            return Err(Error::InvalidKey);
        };

        let secret_key = if info.is_hardened {
            master_to_wallet_hardened(&master_sk, info.derivation_index)
        } else {
            master_to_wallet_unhardened(&master_sk, info.derivation_index)
        }
        .derive_synthetic();

        let decoded_message = parse_signature_message(req.message)?;
        let signature = sign(
            &secret_key,
            ("Chia Signed Message", decoded_message).tree_hash(),
        );

        Ok(SignMessageWithPublicKeyResponse {
            signature: hex::encode(signature.to_bytes()),
        })
    }

    pub async fn sign_message_by_address(
        &self,
        req: SignMessageByAddress,
    ) -> Result<SignMessageByAddressResponse> {
        let wallet = self.wallet()?;

        if let Some(launcher_id) = wallet.vault_launcher_id() {
            return self.sign_vault_message(&wallet, launcher_id, req).await;
        }

        // Checked first, so watch wallets get a clear error rather than an unknown key.
        let master_sk = self.master_secret_key(&wallet)?;

        let p2_puzzle_hash = self.parse_address(req.address)?;
        let Some(public_key) = wallet.db.public_key(p2_puzzle_hash).await? else {
            return Err(Error::InvalidKey);
        };

        let Some(info) = wallet.db.derivation(public_key).await? else {
            return Err(Error::InvalidKey);
        };

        let secret_key = if info.is_hardened {
            master_to_wallet_hardened(&master_sk, info.derivation_index)
        } else {
            master_to_wallet_unhardened(&master_sk, info.derivation_index)
        }
        .derive_synthetic();

        let decoded_message = parse_signature_message(req.message)?;
        let signature = sign(
            &secret_key,
            ("Chia Signed Message", decoded_message).tree_hash(),
        );

        Ok(SignMessageByAddressResponse {
            public_key: hex::encode(public_key.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
        })
    }

    /// Signs a message as a vault, for one of the vault's addresses. The vault signs a
    /// [`vault_message_envelope`] that sends the tree hash of `("Chia Signed Message", message)`,
    /// the same message a BLS key signs, and the result is a `vaultsig` [`VaultMessageProof`].
    ///
    /// The proof is checked against the vault's custody hash, so it's only valid while the
    /// vault's custody is unchanged. The response's public key is the vault's `vault1` id, which
    /// a verifier uses to look up the custody hash.
    async fn sign_vault_message(
        &self,
        wallet: &Wallet,
        launcher_id: Bytes32,
        req: SignMessageByAddress,
    ) -> Result<SignMessageByAddressResponse> {
        let p2_puzzle_hash = self.parse_address(req.address)?;

        match wallet.db.p2_puzzle(p2_puzzle_hash).await {
            Ok(P2Puzzle::P2Singleton(p2)) if p2.launcher_id == launcher_id => {}
            _ => return Err(Error::InvalidKey),
        }

        let secret_keys = self.vault_signer_secret_keys(wallet.fingerprint)?;

        if secret_keys.is_empty() {
            return Err(Error::NoVaultSigners);
        }

        let vault = wallet.spendable_vault().await?;
        let decoded_message = parse_signature_message(req.message)?;
        let message: Bytes32 = ("Chia Signed Message", decoded_message).tree_hash().into();

        let mut ctx = SpendContext::new();

        let envelope = vault_message_envelope(&mut ctx, &vault.vault, &[message.to_vec().into()])?;
        let mips_spend = custody_mips_spend(
            &mut ctx,
            &vault.custody,
            vault.recovery_member_hash,
            envelope,
            &wallet.vault_signer_keys(),
        )?;
        vault.vault.spend(&mut ctx, &mips_spend)?;

        let Some(coin_spend) = ctx.take().into_iter().next() else {
            return Err(Error::MissingCoin(vault.vault.coin.coin_id()));
        };

        let spend_bundle = sign_with_secret_keys(
            SpendBundle::new(vec![coin_spend], Signature::default()),
            &AggSigConstants::new(self.network().agg_sig_me()),
            &secret_keys,
            false,
        )?;

        let Some(coin_spend) = spend_bundle.coin_spends.into_iter().next() else {
            return Err(Error::MissingCoin(vault.vault.coin.coin_id()));
        };

        let proof = VaultMessageProof::new(coin_spend, spend_bundle.aggregated_signature);

        Ok(SignMessageByAddressResponse {
            public_key: encode_vault_id(launcher_id)?,
            signature: proof.encode_string()?,
        })
    }

    pub async fn send_transaction_immediately(
        &self,
        req: SendTransactionImmediately,
    ) -> Result<SendTransactionImmediatelyResponse> {
        // TODO: Should this be the normal way of sending transactions?

        let wallet = self.wallet()?;
        let spend_bundle = rust_bundle(req.spend_bundle)?;
        let peers = self.peer_state.lock().await.peers();

        debug!("{spend_bundle:?}");

        let transaction_id = spend_bundle.name();

        match submit_to_peers(&peers, spend_bundle.clone()).await? {
            Status::Pending(status) => {
                let peer = self
                    .peer_state
                    .lock()
                    .await
                    .acquire_peer()
                    .ok_or(Error::NoPeers)?;

                let subscriptions = insert_transaction(
                    &wallet.db,
                    &peer,
                    wallet.genesis_challenge,
                    spend_bundle.name(),
                    Transaction::from_coin_spends(spend_bundle.coin_spends)?,
                    spend_bundle.aggregated_signature,
                )
                .await?;

                self.command_sender
                    .send(SyncCommand::SubscribeCoins {
                        coin_ids: subscriptions,
                    })
                    .await?;

                info!("Successfully submitted and inserted transaction {transaction_id}");

                Ok(SendTransactionImmediatelyResponse {
                    status,
                    error: None,
                })
            }
            Status::Failed(status, error) => {
                warn!(
                    "Transaction {transaction_id} failed with status {status} and error {error:?}"
                );

                Ok(SendTransactionImmediatelyResponse { status, error })
            }
            Status::Unknown => {
                warn!("Transaction {transaction_id} failed with unknown status");

                Ok(SendTransactionImmediatelyResponse {
                    status: 3,
                    error: None,
                })
            }
        }
    }
}

fn rust_bundle(spend_bundle: wallet_connect::SpendBundle) -> Result<SpendBundle> {
    Ok(SpendBundle {
        coin_spends: spend_bundle
            .coin_spends
            .into_iter()
            .map(rust_spend)
            .collect::<Result<_>>()?,
        aggregated_signature: parse_signature(spend_bundle.aggregated_signature)?,
    })
}

fn rust_spend(coin_spend: wallet_connect::CoinSpend) -> Result<CoinSpend> {
    Ok(CoinSpend {
        coin: rust_coin(coin_spend.coin)?,
        puzzle_reveal: parse_program(coin_spend.puzzle_reveal)?,
        solution: parse_program(coin_spend.solution)?,
    })
}

fn rust_coin(coin: wallet_connect::Coin) -> Result<Coin> {
    Ok(Coin {
        parent_coin_info: parse_coin_id(coin.parent_coin_info)?,
        puzzle_hash: parse_hash(coin.puzzle_hash)?,
        amount: coin
            .amount
            .to_u64()
            .ok_or(Error::InvalidCoinAmount(coin.amount.to_string()))?,
    })
}
