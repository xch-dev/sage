use std::collections::HashMap;

use chia_wallet_sdk::{driver::BURN_PUZZLE_HASH, prelude::*, puzzles::SETTLEMENT_PAYMENT_HASH};
use sage_api::{
    Amount, CoinJson, CoinSpendJson, HandleRecord, SpendBundleJson, TransactionInput,
    TransactionOutput, TransactionSummary,
};
use sage_assets::{Data, base64_data_uri};
use sage_database::{Asset, AssetKind, Database, ParsedNftMetadata};
use sage_wallet::{CoinKind, Transaction, compute_nft_info};

use crate::{Error, HandleSend, Result, Sage};

use super::{parse_coin_id, parse_hash, parse_program, parse_signature};

#[derive(Debug, Default)]
pub struct ConfirmationInfo {
    pub nft_data: HashMap<Bytes32, Data>,
    pub uri_hashes: HashMap<Vec<String>, Bytes32>,
    /// The verified handle recipients, shown in the summary.
    pub handles: Vec<HandleRecord>,
    /// The handle recipients to check again before the transaction is submitted, and to
    /// record once it is.
    pub handle_sends: Vec<HandleSend>,
}

impl Sage {
    pub(crate) async fn summarize(
        &self,
        coin_spends: Vec<CoinSpend>,
        mut cache: ConfirmationInfo,
    ) -> Result<TransactionSummary> {
        let wallet = self.wallet()?;

        let transaction = Transaction::from_coin_spends(coin_spends)?;
        let declared_fee = transaction.fee;
        // A transaction that leaves less than its declared fee can't be valid on its own, so
        // the rest must come from spends that aren't here, like one that creates more than it
        // spends. Any transaction it's part of pays at least the declared fee.
        let fee = transaction
            .actual_fee
            .filter(|actual_fee| *actual_fee >= declared_fee)
            .unwrap_or(declared_fee);

        let input_coins: HashMap<Bytes32, Coin> = transaction
            .inputs
            .iter()
            .map(|input| {
                let coin = input.coin_spend.coin;
                (coin.coin_id(), coin)
            })
            .collect();

        let mut inputs = Vec::with_capacity(transaction.inputs.len());

        for input in transaction.inputs {
            let coin = input.coin_spend.coin;

            let mut p2_puzzle_hash = coin.puzzle_hash;

            let asset = match input.kind {
                CoinKind::Launcher => None,
                CoinKind::Unknown => {
                    if let Some(owner_puzzle_hash) = owner_puzzle_hash(coin, &input_coins)
                        && wallet.db.is_p2_puzzle_hash(owner_puzzle_hash).await?
                    {
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
                        self.cache_nft(
                            &allocator,
                            info.launcher_id,
                            metadata,
                            info.metadata_updater_puzzle_hash,
                            &mut cache,
                        )
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
            fee: Amount::u64(fee),
            declared_fee: Amount::u64(declared_fee),
            inputs,
            handles: cache.handles,
        })
    }
}

/// Settlement coins don't have an owner, but ones we created are still our XCH. This is the
/// puzzle hash of the first ancestor in the transaction that isn't a settlement coin.
fn owner_puzzle_hash(coin: Coin, input_coins: &HashMap<Bytes32, Coin>) -> Option<Bytes32> {
    let mut owner = coin;

    while owner.puzzle_hash == SETTLEMENT_PAYMENT_HASH.into() {
        owner = *input_coins.get(&owner.parent_coin_info)?;
    }

    Some(owner.puzzle_hash)
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
    onchain_metadata: Option<ParsedNftMetadata>,
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
            result.icon_url = Some(base64_data_uri(&thumbnail.icon, "image/png"));
        } else if let Some(db) = &db
            && let Some(icon) = db.icon(data_hash).await?
        {
            result.icon_url = Some(base64_data_uri(&icon.data, "image/png"));
        }
    }

    if let Some(metadata_hash) = onchain_metadata.metadata_hash {
        if let Some(metadata) = cache.nft_data.get(&metadata_hash) {
            let info = compute_nft_info(None, &metadata.blob);
            result.name = info.name;
            result.description = info.description;
            result.is_sensitive_content = info.sensitive_content;
        } else if let Some(db) = &db
            && let Some(metadata) = db.full_file_data(metadata_hash).await?
        {
            let info = compute_nft_info(None, &metadata.data);
            result.name = info.name;
        }
    }

    // The display name from on-chain metadata takes precedence over the file's name
    if onchain_metadata.display_name.is_some() {
        result.name = onchain_metadata.display_name;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_puzzle_hash() {
        let settlement_hash = SETTLEMENT_PAYMENT_HASH.into();
        let wallet_hash = Bytes32::new([1; 32]);

        let wallet_coin = Coin::new(Bytes32::new([2; 32]), wallet_hash, 1);
        let settlement = Coin::new(wallet_coin.coin_id(), settlement_hash, 1);
        let nested = Coin::new(settlement.coin_id(), settlement_hash, 1);
        let external = Coin::new(Bytes32::new([3; 32]), settlement_hash, 1);

        let input_coins: HashMap<Bytes32, Coin> = [wallet_coin, settlement, nested]
            .into_iter()
            .map(|coin| (coin.coin_id(), coin))
            .collect();

        assert_eq!(
            owner_puzzle_hash(wallet_coin, &input_coins),
            Some(wallet_hash)
        );
        assert_eq!(
            owner_puzzle_hash(settlement, &input_coins),
            Some(wallet_hash)
        );
        assert_eq!(owner_puzzle_hash(nested, &input_coins), Some(wallet_hash));
        assert_eq!(owner_puzzle_hash(external, &input_coins), None);
    }
}
