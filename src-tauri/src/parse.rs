use chia::protocol::Bytes32;
use chia_wallet_sdk::decode_address;
use sage_api::Amount;

use crate::error::{Error, ErrorKind, Result};

#[allow(clippy::needless_pass_by_value)]
pub fn parse_asset_id(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?.try_into().map_err(|_| Error {
        kind: ErrorKind::Api,
        reason: format!("Invalid asset ID: {input}"),
    })?;
    Ok(asset_id.into())
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_genesis_challenge(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?.try_into().map_err(|_| Error {
        kind: ErrorKind::Api,
        reason: format!("Invalid genesis challenge: {input}"),
    })?;
    Ok(asset_id.into())
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_coin_id(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?.try_into().map_err(|_| Error {
        kind: ErrorKind::Api,
        reason: format!("Invalid coin ID: {input}"),
    })?;
    Ok(asset_id.into())
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_did_id(input: String) -> Result<Bytes32> {
    let (launcher_id, prefix) = decode_address(&input)?;

    if prefix != "did:chia:" {
        return Err(Error {
            kind: ErrorKind::Api,
            reason: format!("Invalid DID ID: {input}"),
        });
    }

    Ok(launcher_id.into())
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_nft_id(input: String) -> Result<Bytes32> {
    let (launcher_id, prefix) = decode_address(&input)?;

    if prefix != "nft" {
        return Err(Error {
            kind: ErrorKind::Api,
            reason: format!("Invalid NFT ID: {input}"),
        });
    }

    Ok(launcher_id.into())
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_collection_id(input: String) -> Result<Bytes32> {
    let (launcher_id, prefix) = decode_address(&input)?;

    if prefix != "col" {
        return Err(Error {
            kind: ErrorKind::Api,
            reason: format!("Invalid collection ID: {input}"),
        });
    }

    Ok(launcher_id.into())
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_cat_amount(input: Amount) -> Result<u64> {
    let Some(amount) = input.to_mojos(3) else {
        return Err(Error {
            kind: ErrorKind::Api,
            reason: format!("Invalid CAT amount: {input}"),
        });
    };

    Ok(amount)
}

#[allow(clippy::needless_pass_by_value)]
pub fn parse_percent(input: Amount) -> Result<u16> {
    let Some(royalty_ten_thousandths) = input.to_ten_thousandths() else {
        return Err(Error {
            kind: ErrorKind::Api,
            reason: format!("Invalid percentage: {input}"),
        });
    };

    Ok(royalty_ten_thousandths)
}
