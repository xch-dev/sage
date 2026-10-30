use chia_wallet_sdk::prelude::*;
use sage_api::AddressKind;
use sage_database::{Asset, AssetKind, NftUpdaterKind};

use crate::{Result, Sage};

impl Sage {
    pub fn encode_asset(&self, asset: Asset) -> Result<sage_api::Asset> {
        Ok(sage_api::Asset {
            asset_id: encode_asset_id(asset.hash, asset.kind)?,
            name: asset.name,
            ticker: asset.ticker,
            precision: asset.precision,
            icon_url: asset.icon_url,
            description: asset.description,
            is_sensitive_content: asset.is_sensitive_content,
            is_visible: asset.is_visible,
            revocation_address: asset
                .hidden_puzzle_hash
                .map(|puzzle_hash| Address::new(puzzle_hash, self.network().prefix()).encode())
                .transpose()?,
            kind: encode_asset_kind(asset.kind),
        })
    }
}

/// Transaction history only ever resolves coins to the wallet's own custody puzzle hashes,
/// so any coin with an address is ours and any coin without one is external.
pub fn address_kind(custody_p2_puzzle_hash: Option<Bytes32>) -> AddressKind {
    if custody_p2_puzzle_hash.is_some() {
        AddressKind::Own
    } else {
        AddressKind::External
    }
}

pub fn encode_asset_id(hash: Bytes32, kind: AssetKind) -> Result<Option<String>> {
    Ok(if hash == Bytes32::default() {
        None
    } else {
        Some(match kind {
            AssetKind::Token => hex::encode(hash),
            AssetKind::Nft => Address::new(hash, "nft".to_string()).encode()?,
            AssetKind::Did => Address::new(hash, "did:chia:".to_string()).encode()?,
            AssetKind::Option => Address::new(hash, "option".to_string()).encode()?,
            AssetKind::Vault => Address::new(hash, "vault".to_string()).encode()?,
        })
    })
}

pub fn encode_asset_kind(kind: AssetKind) -> sage_api::AssetKind {
    match kind {
        AssetKind::Token => sage_api::AssetKind::Token,
        AssetKind::Nft => sage_api::AssetKind::Nft,
        AssetKind::Did => sage_api::AssetKind::Did,
        AssetKind::Option => sage_api::AssetKind::Option,
        AssetKind::Vault => sage_api::AssetKind::Vault,
    }
}

pub fn encode_nft_updater_kind(kind: NftUpdaterKind) -> sage_api::NftUpdaterKind {
    match kind {
        NftUpdaterKind::Default => sage_api::NftUpdaterKind::Default,
        NftUpdaterKind::Any => sage_api::NftUpdaterKind::Any,
        NftUpdaterKind::Unknown => sage_api::NftUpdaterKind::Unknown,
    }
}
