use std::collections::HashMap;

use base64::{prelude::BASE64_STANDARD, Engine};
use chia::{
    protocol::{Bytes32, Coin, CoinSpend, SpendBundle},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::utils::Address;
use sage_api::{
    Amount, AssetKind, CoinJson, CoinSpendJson, SpendBundleJson, TransactionInput,
    TransactionOutput, TransactionSummary,
};
use sage_assets::Data;
use sage_database::Database;
use sage_wallet::{compute_nft_info, CoinKind, Transaction};

use crate::{Error, Result, Sage};

use super::{parse_coin_id, parse_hash, parse_program, parse_signature, BURN_PUZZLE_HASH};

#[derive(Debug, Default)]
pub struct ConfirmationInfo {
    pub did_names: HashMap<Bytes32, String>,
    pub nft_data: HashMap<Bytes32, Data>,
}

impl Sage {
    pub(crate) async fn summarize(
        &self,
        coin_spends: Vec<CoinSpend>,
        cache: ConfirmationInfo,
    ) -> Result<TransactionSummary> {
        let wallet = self.wallet()?;

        let transaction = Transaction::from_coin_spends(coin_spends)?;

        let mut inputs = Vec::with_capacity(transaction.inputs.len());

        for input in transaction.inputs {
            let coin = input.coin_spend.coin;

            let (kind, p2_puzzle_hash) = match input.kind {
                CoinKind::Unknown => {
                    let kind = if wallet.db.is_p2_puzzle_hash(coin.puzzle_hash).await? {
                        let xch = wallet.db.token_asset(Bytes32::default()).await?;
                        AssetKind::Token {
                            asset_id: hex::encode(
                                xch.as_ref().map(|xch| xch.asset.hash).unwrap_or_default(),
                            ),
                            name: xch.as_ref().and_then(|xch| xch.asset.name.clone()),
                            ticker: xch.as_ref().and_then(|xch| xch.ticker.clone()),
                            icon_url: xch.as_ref().and_then(|xch| xch.asset.icon_url.clone()),
                            precision: xch.as_ref().map_or(12, |xch| xch.precision),
                            is_xch: true,
                        }
                    } else {
                        AssetKind::Unknown
                    };
                    (kind, coin.puzzle_hash)
                }
                CoinKind::Launcher => (AssetKind::Launcher, coin.puzzle_hash),
                CoinKind::Cat { info } => {
                    let cat = wallet.db.token_asset(info.asset_id).await?;
                    let kind = AssetKind::Token {
                        asset_id: hex::encode(info.asset_id),
                        name: cat.as_ref().and_then(|cat| cat.asset.name.clone()),
                        ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                        icon_url: cat.as_ref().and_then(|cat| cat.asset.icon_url.clone()),
                        precision: cat.as_ref().map_or(3, |cat| cat.precision),
                        is_xch: info.asset_id == Bytes32::default(),
                    };
                    (kind, info.p2_puzzle_hash)
                }
                CoinKind::Did { info } => {
                    let name = if let Some(name) = cache.did_names.get(&info.launcher_id).cloned() {
                        Some(name)
                    } else {
                        wallet
                            .db
                            .asset(info.launcher_id)
                            .await?
                            .and_then(|asset| asset.name)
                    };

                    let kind = AssetKind::Did {
                        asset_id: Address::new(info.launcher_id, "did:chia:".to_string())
                            .encode()?,
                        name,
                        icon_url: None,
                    };

                    (kind, info.p2_puzzle_hash)
                }
                CoinKind::Nft { info, metadata } => {
                    let extracted = extract_nft_data(Some(&wallet.db), metadata, &cache).await?;

                    let kind = AssetKind::Nft {
                        asset_id: Address::new(info.launcher_id, "nft".to_string()).encode()?,
                        icon_url: extracted.icon.map(|icon| BASE64_STANDARD.encode(icon)),
                        name: extracted.name,
                    };

                    (kind, info.p2_puzzle_hash)
                }
            };

            let address = Address::new(p2_puzzle_hash, self.network().prefix()).encode()?;

            let mut outputs = Vec::new();

            for output in input.outputs {
                let p2_puzzle_hash = output
                    .kind
                    .receiver_custody_p2_puzzle_hash()
                    .unwrap_or(output.coin.puzzle_hash);

                let address = Address::new(p2_puzzle_hash, self.network().prefix()).encode()?;

                outputs.push(TransactionOutput {
                    coin_id: hex::encode(output.coin.coin_id()),
                    amount: Amount::u64(output.coin.amount),
                    address,
                    receiving: wallet.db.is_custody_p2_puzzle_hash(p2_puzzle_hash).await?,
                    burning: p2_puzzle_hash.to_bytes() == BURN_PUZZLE_HASH,
                });
            }

            inputs.push(TransactionInput {
                coin_id: hex::encode(coin.coin_id()),
                amount: Amount::u64(coin.amount),
                address,
                kind,
                outputs,
            });
        }

        Ok(TransactionSummary {
            fee: Amount::u64(transaction.fee),
            inputs,
        })
    }
}

#[derive(Debug, Default)]
pub struct ExtractedNftData {
    pub icon: Option<Vec<u8>>,
    pub name: Option<String>,
}

pub async fn extract_nft_data(
    db: Option<&Database>,
    onchain_metadata: Option<NftMetadata>,
    cache: &ConfirmationInfo,
) -> Result<ExtractedNftData> {
    let mut result = ExtractedNftData::default();

    let Some(onchain_metadata) = onchain_metadata else {
        return Ok(result);
    };

    if let Some(data_hash) = onchain_metadata.data_hash {
        if let Some(Data {
            thumbnail: Some(thumbnail),
            ..
        }) = cache.nft_data.get(&data_hash)
        {
            result.icon = Some(thumbnail.icon.clone());
        } else if let Some(db) = &db {
            if let Some(icon) = db.icon(data_hash).await? {
                result.icon = Some(icon.data);
            }
        }
    }

    if let Some(metadata_hash) = onchain_metadata.metadata_hash {
        if let Some(metadata) = cache.nft_data.get(&metadata_hash) {
            let info = compute_nft_info(None, &metadata.blob);
            result.name = info.name;
        } else if let Some(db) = &db {
            if let Some(metadata) = db.full_file_data(metadata_hash).await? {
                let info = compute_nft_info(None, &metadata.data);
                result.name = info.name;
            }
        }
    }

    Ok(result)
}

pub fn json_bundle(spend_bundle: &SpendBundle) -> SpendBundleJson {
    SpendBundleJson {
        coin_spends: spend_bundle.coin_spends.iter().map(json_spend).collect(),
        aggregated_signature: format!(
            "0x{}",
            hex::encode(spend_bundle.aggregated_signature.to_bytes())
        ),
    }
}

pub fn json_spend(coin_spend: &CoinSpend) -> CoinSpendJson {
    CoinSpendJson {
        coin: json_coin(&coin_spend.coin),
        puzzle_reveal: hex::encode(&coin_spend.puzzle_reveal),
        solution: hex::encode(&coin_spend.solution),
    }
}

pub fn json_coin(coin: &Coin) -> CoinJson {
    CoinJson {
        parent_coin_info: format!("0x{}", hex::encode(coin.parent_coin_info)),
        puzzle_hash: format!("0x{}", hex::encode(coin.puzzle_hash)),
        amount: Amount::u64(coin.amount),
    }
}

pub fn rust_bundle(spend_bundle: SpendBundleJson) -> Result<SpendBundle> {
    Ok(SpendBundle {
        coin_spends: spend_bundle
            .coin_spends
            .into_iter()
            .map(rust_spend)
            .collect::<Result<_>>()?,
        aggregated_signature: parse_signature(spend_bundle.aggregated_signature)?,
    })
}

pub fn rust_spend(coin_spend: CoinSpendJson) -> Result<CoinSpend> {
    Ok(CoinSpend {
        coin: rust_coin(coin_spend.coin)?,
        puzzle_reveal: parse_program(coin_spend.puzzle_reveal)?,
        solution: parse_program(coin_spend.solution)?,
    })
}

pub fn rust_coin(coin: CoinJson) -> Result<Coin> {
    Ok(Coin {
        parent_coin_info: parse_coin_id(coin.parent_coin_info)?,
        puzzle_hash: parse_hash(coin.puzzle_hash)?,
        amount: coin
            .amount
            .to_u64()
            .ok_or(Error::InvalidCoinAmount(coin.amount.to_string()))?,
    })
}
