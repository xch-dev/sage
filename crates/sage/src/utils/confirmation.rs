use std::collections::HashMap;

use chia::{
    clvm_traits::ToClvm,
    protocol::{Bytes32, Coin, CoinSpend, SpendBundle},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{driver::BURN_PUZZLE_HASH, utils::Address};
use clvmr::Allocator;
use sage_api::{
    Amount, CoinJson, CoinSpendJson, SpendBundleJson, TransactionInput, TransactionOutput,
    TransactionSummary,
};
use sage_assets::{base64_data_uri, Data};
use sage_database::{Asset, AssetKind, Database};
use sage_wallet::{compute_nft_info, CoinKind, Transaction};

use crate::{Error, Result, Sage};

use super::{parse_coin_id, parse_hash, parse_program, parse_signature};

#[derive(Debug, Default)]
pub struct ConfirmationInfo {
    pub nft_data: HashMap<Bytes32, Data>,
}

impl Sage {
    pub(crate) async fn summarize(
        &self,
        coin_spends: Vec<CoinSpend>,
        mut cache: ConfirmationInfo,
    ) -> Result<TransactionSummary> {
        let wallet = self.wallet()?;

        let transaction = Transaction::from_coin_spends(coin_spends)?;

        let mut inputs = Vec::with_capacity(transaction.inputs.len());

        for input in transaction.inputs {
            let coin = input.coin_spend.coin;

            let mut p2_puzzle_hash = coin.puzzle_hash;

            let asset = match input.kind {
                CoinKind::Launcher => None,
                CoinKind::Unknown => {
                    if wallet.db.is_p2_puzzle_hash(coin.puzzle_hash).await? {
                        wallet.db.asset(Bytes32::default()).await?
                    } else {
                        None
                    }
                }
                CoinKind::Cat { info } => {
                    p2_puzzle_hash = info.p2_puzzle_hash;
                    Some(
                        self.cache_cat(info.asset_id, info.hidden_puzzle_hash)
                            .await?,
                    )
                }
                CoinKind::Nft { info, .. } => {
                    let mut allocator = Allocator::new();
                    let metadata = info.metadata.to_clvm(&mut allocator)?;
                    p2_puzzle_hash = info.p2_puzzle_hash;
                    Some(
                        self.cache_nft(&allocator, info.launcher_id, metadata, &mut cache)
                            .await?,
                    )
                }
                CoinKind::Did { info, .. } => {
                    Some(wallet.db.asset(info.launcher_id).await?.unwrap_or(Asset {
                        hash: info.launcher_id,
                        name: None,
                        ticker: None,
                        precision: 1,
                        icon_url: None,
                        description: None,
                        is_sensitive_content: false,
                        is_visible: true,
                        hidden_puzzle_hash: None,
                        kind: AssetKind::Did,
                    }))
                }
                CoinKind::Option { info, .. } => {
                    // TODO: Is this correct? We should probably validate and fill in option info somehow
                    Some(wallet.db.asset(info.launcher_id).await?.unwrap_or(Asset {
                        hash: info.launcher_id,
                        name: None,
                        ticker: None,
                        precision: 1,
                        icon_url: None,
                        description: None,
                        is_sensitive_content: false,
                        is_visible: true,
                        hidden_puzzle_hash: None,
                        kind: AssetKind::Option,
                    }))
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
                    burning: p2_puzzle_hash == BURN_PUZZLE_HASH,
                });
            }

            inputs.push(TransactionInput {
                coin_id: hex::encode(coin.coin_id()),
                amount: Amount::u64(coin.amount),
                address,
                asset: asset.map(|asset| self.encode_asset(asset)).transpose()?,
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
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub is_sensitive_content: bool,
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
            mime_type,
            ..
        }) = cache.nft_data.get(&data_hash)
        {
            result.icon_url = Some(base64_data_uri(&thumbnail.icon, mime_type));
        } else if let Some(db) = &db {
            if let Some(icon) = db.icon(data_hash).await? {
                result.icon_url = Some(base64_data_uri(
                    &icon.data,
                    icon.mime_type.as_deref().unwrap_or("image/png"),
                ));
            }
        }
    }

    if let Some(metadata_hash) = onchain_metadata.metadata_hash {
        if let Some(metadata) = cache.nft_data.get(&metadata_hash) {
            let info = compute_nft_info(None, &metadata.blob);
            result.name = info.name;
            result.description = info.description;
            result.is_sensitive_content = info.sensitive_content;
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
