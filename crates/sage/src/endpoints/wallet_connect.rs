use std::slice;

use chia::{
    bls::{master_to_wallet_hardened, master_to_wallet_unhardened, sign},
    clvm_utils::ToTreeHash,
    protocol::{Bytes32, Coin, CoinSpend, SpendBundle},
    puzzles::{DeriveSynthetic, Proof},
};
use chia_wallet_sdk::driver::{ClawbackV2, Layer, OptionUnderlying, SpendContext, StandardLayer};
use sage_api::wallet_connect::{
    self, AssetCoinType, FilterUnlockedCoins, FilterUnlockedCoinsResponse, GetAssetCoins,
    GetAssetCoinsResponse, LineageProof, SendTransactionImmediately,
    SendTransactionImmediatelyResponse, SignMessageByAddress, SignMessageByAddressResponse,
    SignMessageWithPublicKey, SignMessageWithPublicKeyResponse, SpendableCoin,
};
use sage_database::{AssetFilter, CoinFilterMode, CoinSortMode, DeserializePrimitive, P2Puzzle};
use sage_wallet::{insert_transaction, submit_to_peers, Status, SyncCommand, Transaction};
use tracing::{debug, info, warn};

use crate::{
    parse_asset_id, parse_coin_id, parse_did_id, parse_hash, parse_nft_id, parse_program,
    parse_public_key, parse_signature, parse_signature_message, Error, Result, Sage,
};

impl Sage {
    pub async fn filter_unlocked_coins(
        &self,
        req: FilterUnlockedCoins,
    ) -> Result<FilterUnlockedCoinsResponse> {
        let wallet = self.wallet()?;
        let mut coin_ids = Vec::new();

        for coin_id in req.coin_ids {
            if wallet
                .db
                .are_coins_spendable(slice::from_ref(&coin_id))
                .await?
            {
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
                    let clawback = ClawbackV2::new(
                        clawback.sender_puzzle_hash,
                        clawback.receiver_puzzle_hash,
                        clawback.seconds,
                        row.coin.amount,
                        true,
                    );
                    clawback.into_1_of_n().construct_puzzle(&mut ctx)?
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
                    amount: row.coin.amount,
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
                        amount: Some(proof.parent_amount),
                    }),
                    Some(Proof::Lineage(proof)) => Some(LineageProof {
                        parent_name: Some(hex::encode(proof.parent_parent_coin_info)),
                        inner_puzzle_hash: Some(hex::encode(proof.parent_inner_puzzle_hash)),
                        amount: Some(proof.parent_amount),
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

        let public_key = parse_public_key(req.public_key)?;
        let Some(info) = wallet.db.derivation(public_key).await? else {
            return Err(Error::InvalidKey);
        };

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
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

        let p2_puzzle_hash = self.parse_address(req.address)?;
        let Some(public_key) = wallet.db.public_key(p2_puzzle_hash).await? else {
            return Err(Error::InvalidKey);
        };

        let Some(info) = wallet.db.derivation(public_key).await? else {
            return Err(Error::InvalidKey);
        };

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
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
            Status::Pending => {
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
                    status: 1,
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
        amount: coin.amount,
    })
}
