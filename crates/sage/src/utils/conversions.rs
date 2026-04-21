use chia_wallet_sdk::{
    driver::BURN_PUZZLE_HASH,
    prelude::*,
    puzzles::{SETTLEMENT_PAYMENT_HASH, SINGLETON_LAUNCHER_HASH},
};
use sage_api::AddressKind;
use sage_database::{Asset, AssetKind};

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

pub fn address_kind(p2_puzzle_hash: Option<Bytes32>) -> AddressKind {
    let Some(p2_puzzle_hash) = p2_puzzle_hash else {
        return AddressKind::External;
    };

    if p2_puzzle_hash == BURN_PUZZLE_HASH {
        return AddressKind::Burn;
    }
    if p2_puzzle_hash == SINGLETON_LAUNCHER_HASH.into() {
        return AddressKind::Launcher;
    }
    if p2_puzzle_hash == SETTLEMENT_PAYMENT_HASH.into() {
        return AddressKind::Offer;
    }

    AddressKind::Own
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
