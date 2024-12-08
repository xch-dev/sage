#![allow(clippy::needless_pass_by_value)]

use chia::{
    bls::{PublicKey, Signature},
    protocol::{Bytes32, Program},
};
use chia_wallet_sdk::decode_address;
use sage_api::Amount;

use crate::{Error, Result};

pub fn parse_asset_id(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?
        .try_into()
        .map_err(|_| Error::InvalidAssetId(input))?;
    Ok(asset_id.into())
}

pub fn parse_genesis_challenge(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?
        .try_into()
        .map_err(|_| Error::InvalidGenesisChallenge(input))?;
    Ok(asset_id.into())
}

pub fn parse_coin_id(input: String) -> Result<Bytes32> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    let asset_id: [u8; 32] = hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidCoinId(input))?;
    Ok(asset_id.into())
}

pub fn parse_did_id(input: String) -> Result<Bytes32> {
    let (launcher_id, prefix) = decode_address(&input)?;

    if prefix != "did:chia:" {
        return Err(Error::InvalidDidId(input));
    }

    Ok(launcher_id.into())
}

pub fn parse_nft_id(input: String) -> Result<Bytes32> {
    let (launcher_id, prefix) = decode_address(&input)?;

    if prefix != "nft" {
        return Err(Error::InvalidNftId(input));
    }

    Ok(launcher_id.into())
}

pub fn parse_collection_id(input: String) -> Result<Bytes32> {
    let (launcher_id, prefix) = decode_address(&input)?;

    if prefix != "col" {
        return Err(Error::InvalidCollectionId(input));
    }

    Ok(launcher_id.into())
}

pub fn parse_offer_id(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?
        .try_into()
        .map_err(|_| Error::InvalidOfferId(input))?;
    Ok(asset_id.into())
}

pub fn parse_cat_amount(input: Amount) -> Result<u64> {
    let Some(amount) = input.to_u64() else {
        return Err(Error::InvalidAmount(input.to_string()));
    };

    Ok(amount)
}

pub fn parse_puzzle_hash(input: String) -> Result<Bytes32> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidPuzzleHash(input))
}

pub fn parse_signature(input: String) -> Result<Signature> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    let signature: [u8; 96] = hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidSignature(input))?;

    Ok(Signature::from_bytes(&signature)?)
}

pub fn parse_public_key(input: String) -> Result<PublicKey> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    let public_key: [u8; 48] = hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidPublicKey(input))?;

    Ok(PublicKey::from_bytes(&public_key)?)
}

pub fn parse_program(input: String) -> Result<Program> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    Ok(hex::decode(stripped)?.into())
}
